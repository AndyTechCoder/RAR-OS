//! Pure System repair decision. No I/O, native authority, allocation or unsafe code.
//! Complete sealed reads and the immutable expected hash are caller obligations.
#![forbid(unsafe_code)]
use crate::{journal::{Record,LayerId},manifest::{self,VerifiedLayer},sha256::sha256};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Reject { Root, MissingPrior, ActiveUsable, PriorUsable, Identity, Changed, Exhausted }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Role { Active, Prior }
const MAX_STORED:usize=(manifest::SIZE+manifest::MAX_PAYLOAD).div_ceil(512)*512;

/// Exact immutable boot-bound factory, not just any correctly signed generation1.
/// The caller must independently bind expected_hash to immutable boot provenance.
pub struct Factory<'a> { layer:VerifiedLayer<'a>, package_hash:[u8;32] }
impl<'a> Factory<'a> {
    pub(crate) fn verify(package:&'a[u8],expected_hash:[u8;32])->Result<Self,Reject> {
        if !(manifest::SIZE+512..=manifest::SIZE+manifest::MAX_PAYLOAD).contains(&package.len()) ||
            expected_hash==[0;32] || sha256(package).map_err(|_|Reject::Root)?!=expected_hash {
            return Err(Reject::Root);
        }
        let layer=manifest::verify(&package[..manifest::SIZE],&package[manifest::SIZE..],1)
            .map_err(|_|Reject::Root)?;
        if layer.manifest().generation()!=1{return Err(Reject::Root);}
        Ok(Self{layer,package_hash:expected_hash})
    }
    pub fn package_hash(&self)->[u8;32]{self.package_hash}
    pub fn manifest_digest(&self)->[u8;32]{self.layer.manifest().digest()}
}

/// A complete sealed read issued only inside the native child module below.
/// No public or crate-private raw-slice constructor exists. The child obtains
/// an opaque lease from the authenticated runtime and consumes classification
/// before releasing it; tests alone have an explicit mock issuer.
pub struct CompleteRead<'a>{bytes:&'a[u8]}
#[cfg(test)]
impl<'a> CompleteRead<'a>{
    /// Mock of a bridge that issues a receipt only after successful exact I/O.
    /// Test-only: this is not the production provenance or transport boundary.
    pub(crate) fn test_completed(expected:usize,result:Result<&'a[u8],()>)->Result<Self,Reject>{
        let bytes=result.map_err(|_|Reject::Identity)?;
        if !(512..=MAX_STORED).contains(&expected)||expected%512!=0||bytes.len()!=expected{
            return Err(Reject::Identity);
        }
        Ok(Self{bytes})
    }
}

/// Evidence from an opaque complete sector read. No caller-supplied damage
/// boolean. Full storage bytes (including padding) bind observation equality.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Inspection {
    current:Record, role:Role, referenced:LayerId,
    length:usize, observed_hash:[u8;32], usable:bool,
}
pub(crate) fn inspect(current:Record,role:Role,read:CompleteRead<'_>)->Result<Inspection,Reject> {
    let stored=read.bytes;
    if !(512..=MAX_STORED).contains(&stored.len())||stored.len()%512!=0{
        return Err(Reject::Identity);
    }
    let referenced=match role {
        Role::Active=>current.active(),
        Role::Prior=>current.previous().ok_or(Reject::MissingPrior)?,
    };
    // Independently reconstruct the reader's exact complete-read shape. Wrong
    // shape is missing transport evidence, never proof that media is damaged.
    let logical=manifest::Manifest::parse(&stored[..manifest::SIZE]).ok().and_then(|_|{
        let payload=u32::from_le_bytes(stored[56..60].try_into().unwrap())as usize;
        (512..=manifest::MAX_PAYLOAD).contains(&payload).then_some(manifest::SIZE+payload)
    });
    let usable=match logical{
        None=>{
            if stored.len()!=512{return Err(Reject::Identity);}
            false
        },
        Some(length)=>{
            if stored.len()!=length.div_ceil(512)*512{return Err(Reject::Identity);}
            if stored[length..].iter().any(|&byte|byte!=0){false}else{
                match manifest::verify(&stored[..manifest::SIZE],&stored[manifest::SIZE..length],1){
                    Ok(layer)=>layer.manifest().generation()==referenced.generation()&&
                        layer.manifest().digest()==referenced.digest(),
                    Err(_)=>false,
                }
            }
        },
    };
    let observed_hash=sha256(stored).map_err(|_|Reject::Identity)?;
    Ok(Inspection{current,role,referenced,length:stored.len(),observed_hash,usable})
}

/// A plan does not authorize publication. Native use additionally requires
/// exclusive System ownership, sealed-root binding, trial health and one-shot
/// request/seal/incarnation correlation. Native freshness is not established here.
pub struct Plan {
    current:Record, active:Inspection, prior:Option<Inspection>,
    factory_hash:[u8;32], next:Record,
}
impl Plan {
    pub(crate) fn new(current:Record,factory:&Factory<'_>,active:Inspection,prior:Option<Inspection>)
        ->Result<Self,Reject>
    {
        if active.current!=current||active.role!=Role::Active||active.referenced!=current.active(){
            return Err(Reject::Identity);
        }
        if active.usable{return Err(Reject::ActiveUsable);}
        match (current.previous(),prior) {
            (None,None)=>(),
            (Some(expected),Some(p))=>{
                if p.current!=current||p.role!=Role::Prior||p.referenced!=expected{return Err(Reject::Identity);}
                if p.usable{return Err(Reject::PriorUsable);}
            },
            _=>return Err(Reject::MissingPrior),
        }
        let next=current.repair_factory(&factory.layer).map_err(|_|Reject::Exhausted)?;
        Ok(Self{current,active,prior,factory_hash:factory.package_hash,next})
    }
    #[cfg(any(test,rar_signed_updates))]
    pub(crate) fn verify_readback(&self,t:crate::update_wire::Transfer,current:Record,package:&[u8])
        ->Result<Record,Reject>
    {
        use crate::update_wire::{Mode,Kind};
        t.frame(Kind::Offer).map_err(|_|Reject::Identity)?;
        let id=t.identity;let active=self.next.active();
        if t.mode!=Mode::Repair||current!=self.current||t.sequence!=current.sequence()||
            id.length!=package.len()||id.package_hash!=self.factory_hash||
            (id.slot,id.generation,id.digest)!=(active.slot(),active.generation(),active.digest()){
            return Err(Reject::Identity);
        }
        let factory=Factory::verify(package,self.factory_hash)?;
        if factory.manifest_digest()!=active.digest(){return Err(Reject::Root);}
        Ok(self.next)
    }
    pub fn next(&self)->Record{self.next}
    pub fn factory_hash(&self)->[u8;32]{self.factory_hash}
    /// Equality check only, NOT evidence of freshness. The native bridge must
    /// independently issue fresh one-shot reads before calling and consuming its
    /// pending transaction. Copying old Inspection values cannot prove that gate.
    pub fn matches_observations(&self,current:Record,factory:&Factory<'_>,active:Inspection,prior:Option<Inspection>)
        ->Result<(),Reject>
    {
        if current!=self.current||factory.package_hash!=self.factory_hash||
            active!=self.active||prior!=self.prior{return Err(Reject::Changed);}
        Ok(())
    }
}

#[cfg(all(rar_signed_updates,not(test)))]
#[path="../../services/modern/repair_bridge.rs"]
pub(crate) mod native;
