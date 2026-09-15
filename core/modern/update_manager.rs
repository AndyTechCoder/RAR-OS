//! Manager-side immutable package/selector binding, before native trial acceptance.
#![forbid(unsafe_code)]
use crate::{journal::Record,manifest::{self,VerifiedLayer},sha256::sha256,
    update_wire::{Transfer,Mode,Kind}};
#[derive(Debug,PartialEq,Eq)]
pub enum Reject{Identity,Record,Package,Signature,Transition}
pub struct Verified<'a>{layer:VerifiedLayer<'a>,next:Option<Record>,transfer:Transfer,boot_prior:bool}
impl<'a> Verified<'a>{
    /// Structural availability only, never authority to skip prior verification.
    pub fn boot_prior(&self)->bool{self.boot_prior}
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
        Mode::Repair=>return Err(Reject::Transition),
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
        Mode::Repair=>return Err(Reject::Transition),
        Mode::Boot=>None,
        Mode::Install=>Some(current.install(&layer).map_err(|_|Reject::Transition)?),
        Mode::Fallback=>Some(current.fallback().map_err(|_|Reject::Transition)?),
    };
    if let Some(record)=next{
        if (record.active().slot(),record.active().generation(),record.active().digest())!=
            (id.slot,id.generation,id.digest){return Err(Reject::Identity);}
    }
    let verified=Verified{layer,next,transfer:t,boot_prior:boot_prior(t.mode,current)};verified.expected_ack()?;Ok(verified)
}

fn boot_prior(mode:Mode,current:Record)->bool{
    mode==Mode::Boot&&current.fallback().is_ok()
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Failure{Rejected,Verify,Trial,Channel,Native,Indeterminate}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum BootAction{Active,PriorOnce,Unavailable,Reconcile}
pub fn boot_action(result:Result<(),Failure>,prior_attempted:bool)->BootAction{
    match result{
        Ok(())=>BootAction::Active,
        Err(Failure::Indeterminate)=>BootAction::Reconcile,
        Err(Failure::Rejected|Failure::Verify|Failure::Trial)if !prior_attempted=>BootAction::PriorOnce,
        _=>BootAction::Unavailable,
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum ReleaseAction{Done,Yield,Stop}
pub fn release_action(status:i64,attempt:usize)->ReleaseAction{
    if attempt>=256{return ReleaseAction::Stop;}
    match status{0=>ReleaseAction::Done,-7 if attempt<255=>ReleaseAction::Yield,_=>ReleaseAction::Stop}
}
#[cfg(test)]mod lifecycle_tests{
    use super::*;
    fn record(kind:u8,sequence:u64)->Record{
        let mut b=[0u8;512];b[..8].copy_from_slice(b"RARSYS00");
        b[10..12].copy_from_slice(&512u16.to_le_bytes());b[12]=kind;
        b[13]=if kind==1||kind==3{1}else{0};b[14]=if kind==1{0}else{255};
        let active=if kind==1{2u64}else{1u64};
        let highest=if kind==0{1u64}else{2u64};
        for(offset,value)in [(16,sequence),(24,highest),(32,1),(40,active),
            (48,if kind==1{1}else{0}),(56,if kind==0{0}else{sequence-1})]{
            b[offset..offset+8].copy_from_slice(&value.to_le_bytes());
        }
        b[64..96].fill(active as u8);
        if kind==1{b[96..128].fill(1);}
        if kind!=0{b[128..160].fill(3);}
        let sum=sha256(&b[..480]).unwrap();b[480..].copy_from_slice(&sum);
        Record::decode(&b).unwrap()
    }
    #[test]fn boot_hint_requires_available_nonexhausted_prior_transition(){
        for(kind,sequence,wanted)in [(0,1,false),(1,2,true),(1,u64::MAX,false),(2,3,false),(3,2,false)]{
            let r=record(kind,sequence);
            assert_eq!(boot_prior(Mode::Boot,r),wanted);
            for mode in [Mode::Install,Mode::Fallback,Mode::Repair]{assert!(!boot_prior(mode,r));}
        }
        let mut corrupt=record(1,2).encode();corrupt[96..128].fill(0);
        let sum=sha256(&corrupt[..480]).unwrap();corrupt[480..].copy_from_slice(&sum);
        assert!(Record::decode(&corrupt).is_err());
        let prior=record(1,2).fallback().unwrap();
        assert!(prior.previous().is_none());assert_eq!(prior.highest_committed_generation(),2);
        assert!(!boot_prior(Mode::Boot,prior));
    }
    #[test]fn only_clean_boot_rejection_can_request_exact_prior_once(){
        for prior in [false,true]{
            assert_eq!(boot_action(Ok(()),prior),BootAction::Active);
            assert_eq!(boot_action(Err(Failure::Indeterminate),prior),BootAction::Reconcile);
            for error in [Failure::Native,Failure::Channel]{
                assert_eq!(boot_action(Err(error),prior),BootAction::Unavailable);
            }
            for error in [Failure::Rejected,Failure::Verify,Failure::Trial]{
                assert_eq!(boot_action(Err(error),prior),
                    if prior{BootAction::Unavailable}else{BootAction::PriorOnce});
            }
        }
    }
    #[test]fn release_retries_only_kernel_busy_within_exact_budget(){
        for attempt in 0..256{
            assert_eq!(release_action(0,attempt),ReleaseAction::Done);
            assert_eq!(release_action(-7,attempt),
                if attempt==255{ReleaseAction::Stop}else{ReleaseAction::Yield});
            for status in [-1,-2,-3,-4,-5,-6,-8,1,i64::MIN,i64::MAX]{
                assert_eq!(release_action(status,attempt),ReleaseAction::Stop);
            }
        }
        for status in [0,-7,1,-1]{assert_eq!(release_action(status,256),ReleaseAction::Stop);}
    }
}
