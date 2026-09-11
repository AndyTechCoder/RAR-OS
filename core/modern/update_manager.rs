//! Manager-side immutable package/selector binding, before native trial acceptance.
#![forbid(unsafe_code)]
use crate::{journal::Record,manifest::{self,VerifiedLayer},sha256::sha256,
    update_wire::{Transfer,Mode,Kind}};
#[derive(Debug,PartialEq,Eq)]
pub enum Reject{Identity,Record,Package,Signature,Transition}
pub struct Verified<'a>{layer:VerifiedLayer<'a>,next:Option<Record>,transfer:Transfer}
impl<'a> Verified<'a>{
    pub fn layer(&self)->&VerifiedLayer<'a>{&self.layer}
    pub fn next(&self)->Option<Record>{self.next}
    pub fn expected_ack(&self)->Result<[u8;128],Reject>{
        let sequence=self.next.map_or(self.transfer.sequence,|r|r.sequence());
        self.transfer.committed(sequence).and_then(|t|t.frame(Kind::Committed)).map_err(|_|Reject::Identity)
    }
}
/// The slice must be the exact kernel-owned sealed read-only view. This pure
/// function does not attest its provenance, create authority, or publish state.
pub fn verify<'a>(t:Transfer,current:Record,package:&'a[u8])->Result<Verified<'a>,Reject>{
    t.frame(Kind::Offer).map_err(|_|Reject::Identity)?;
    if current.sequence()!=t.sequence{return Err(Reject::Record);}
    let id=t.identity;
    let minimum=match t.mode{
        Mode::Boot=>{
            let active=current.active();
            if (id.slot,id.generation,id.digest)!=(active.slot(),active.generation(),active.digest()){
                return Err(Reject::Identity);
            }
            active.generation()
        },
        Mode::Fallback=>{
            let prior=current.previous().ok_or(Reject::Transition)?;
            if (id.slot,id.generation,id.digest)!=(prior.slot(),prior.generation(),prior.digest()){
                return Err(Reject::Identity);
            }
            prior.generation()
        },
        Mode::Install=>{
            if id.slot!=current.active().slot().other(){return Err(Reject::Identity);}
            current.minimum_install_generation().map_err(|_|Reject::Transition)?
        },
    };
    if package.len()!=id.length||package.len()<manifest::SIZE||
        sha256(package).map_err(|_|Reject::Package)?!=id.package_hash{return Err(Reject::Package);}
    let layer=manifest::verify(&package[..manifest::SIZE],&package[manifest::SIZE..],minimum)
        .map_err(|_|Reject::Signature)?;
    if layer.manifest().generation()!=id.generation||layer.manifest().digest()!=id.digest{
        return Err(Reject::Identity);
    }
    let next=match t.mode{
        Mode::Boot=>None,
        Mode::Install=>Some(current.install(&layer).map_err(|_|Reject::Transition)?),
        Mode::Fallback=>Some(current.fallback().map_err(|_|Reject::Transition)?),
    };
    if let Some(record)=next{
        if (record.active().slot(),record.active().generation(),record.active().digest())!=
            (id.slot,id.generation,id.digest){return Err(Reject::Identity);}
    }
    let verified=Verified{layer,next,transfer:t};verified.expected_ack()?;Ok(verified)
}
