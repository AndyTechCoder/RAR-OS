//! Pure System repair decision. No I/O, native authority, allocation or unsafe code.
//! Complete sealed reads and the immutable expected hash are caller obligations.
#![forbid(unsafe_code)]
use crate::{journal::{Record,LayerId},manifest::{self,VerifiedLayer},sha256::sha256};

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Reject { Root, MissingPrior, ActiveUsable, PriorUsable, Identity, Changed, Exhausted }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Role { Active, Prior }

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

/// Evidence from a complete bounded content read; never construct from a transport
/// failure or a partially filled buffer. No public constructor or caller boolean.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Inspection {
    current:Record, role:Role, referenced:LayerId,
    length:usize, observed_hash:[u8;32], usable:bool,
}
pub fn inspect(current:Record,role:Role,package:&[u8])->Result<Inspection,Reject> {
    if package.len()>manifest::SIZE+manifest::MAX_PAYLOAD{return Err(Reject::Identity);}
    let referenced=match role {
        Role::Active=>current.active(),
        Role::Prior=>current.previous().ok_or(Reject::MissingPrior)?,
    };
    let observed_hash=sha256(package).map_err(|_|Reject::Identity)?;
    let usable=if package.len()<manifest::SIZE {false} else {
        match manifest::verify(&package[..manifest::SIZE],&package[manifest::SIZE..],1) {
            Ok(layer)=>layer.manifest().generation()==referenced.generation() &&
                layer.manifest().digest()==referenced.digest(),
            Err(_)=>false,
        }
    };
    Ok(Inspection{current,role,referenced,length:package.len(),observed_hash,usable})
}

/// A plan does not authorize publication. Native use additionally requires
/// exclusive System ownership, sealed-root binding, trial health and one-shot
/// request/seal/incarnation correlation. Reobserve immediately before mutation.
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
    pub fn next(&self)->Record{self.next}
    pub fn factory_hash(&self)->[u8;32]{self.factory_hash}
    /// Exact fresh observations must still match. This performs no I/O and cannot
    /// establish native provenance/exclusion or consume the owner's transaction.
    pub fn recheck(&self,current:Record,factory:&Factory<'_>,active:Inspection,prior:Option<Inspection>)
        ->Result<(),Reject>
    {
        if current!=self.current||factory.package_hash!=self.factory_hash||
            active!=self.active||prior!=self.prior{return Err(Reject::Changed);}
        Ok(())
    }
}
