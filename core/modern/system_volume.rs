//! Fixed experimental System media adapter. No formatting, caller LBA or Data handle.
//! Publication is a storage primitive, NOT a health or execution authorization.
use crate::{journal::{self, Journal, Record, Selection, SelectorIo, SelectorSector, Slot},
    manifest, sha256::{sha256,Sha256}};

pub const SECTORS: u32 = 16_384;
pub const SLOT_SECTORS: u32 = 4097;
pub const MAX_PACKAGE: usize = manifest::SIZE + manifest::MAX_PAYLOAD;
pub const FIRST_RESERVED: u32 = 8196;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject { Capacity, Buffer, Framing, Io, Changed, ReadOnly, Policy, Indeterminate, Sink }
/// Implemented only over the System service's kernel-bound device. Capacity and
/// device identity are independently enforced by the kernel/PIO owner.
pub trait Io {
    fn read(&mut self, sector:u32)->Result<[u8;512],()>;
    fn write(&mut self, sector:u32, bytes:&[u8;512])->Result<(),()>;
    fn flush(&mut self)->Result<(),()>;
}
pub fn slot_start(slot:Slot)->u32 {match slot {Slot::A=>2,Slot::B=>2+SLOT_SECTORS}}
fn sectors(length:usize)->usize {length.div_ceil(512)}
fn package_length(first:&[u8;512])->Result<usize,Reject> {
    manifest::Manifest::parse(&first[..manifest::SIZE]).map_err(|_|Reject::Framing)?;
    let payload=u32::from_le_bytes(first[56..60].try_into().unwrap()) as usize;
    if !(512..=manifest::MAX_PAYLOAD).contains(&payload){return Err(Reject::Framing);}
    Ok(manifest::SIZE+payload)
}
fn block(package:&[u8], index:usize)->[u8;512] {
    let mut out=[0;512];let offset=index*512;
    let count=core::cmp::min(512,package.len()-offset);
    out[..count].copy_from_slice(&package[offset..offset+count]);out
}
fn selection<I:Io>(io:&mut I)->Result<Selection,Reject>{
    let a=io.read(0).map_err(|_|Reject::Io)?;
    let b=io.read(1).map_err(|_|Reject::Io)?;
    journal::select([&a,&b]).map_err(|_|Reject::Framing)
}
struct Selectors<'a,I:Io>(&'a mut I);
impl<I:Io> SelectorIo for Selectors<'_,I> {
    fn read(&mut self,s:SelectorSector)->Result<[u8;512],()> {
        self.0.read(match s {SelectorSector::First=>0,SelectorSector::Second=>1})
    }
    fn write(&mut self,s:SelectorSector,b:&[u8;512])->Result<(),()> {
        self.0.write(match s {SelectorSector::First=>0,SelectorSector::Second=>1},b)
    }
    fn flush(&mut self)->Result<(),()>{self.0.flush()}
}
/// Durable byte identity, NOT a signature, health or capability token.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Identity {
    pub transaction:u64, pub slot:Slot, pub generation:u64, pub digest:[u8;32],
    pub length:usize, pub package_hash:[u8;32],
}
/// Private fields and no Clone: only this volume can produce a prepared state.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Purpose{Install,Fallback,Boot}
pub struct Prepared {before:Selection,identity:Identity,purpose:Purpose}
impl Prepared {pub fn identity(&self)->Identity{self.identity}}
pub struct Volume<I:Io>{io:I,selected:Selection,locked:bool,next:Option<u64>,pending:Option<(Identity,Purpose)>}
impl<I:Io> Volume<I> {
    pub fn mount(mut io:I,capacity:u32)->Result<Self,Reject> {
        if capacity!=SECTORS{return Err(Reject::Capacity);}
        let selected=selection(&mut io)?;
        Ok(Self{io,selected,locked:false,next:Some(1),pending:None})
    }
    pub fn record(&self)->Record{self.selected.record()}
    pub fn is_readonly(&self)->bool{self.locked}
    fn open(&self)->Result<(),Reject>{if self.locked{Err(Reject::ReadOnly)}else{Ok(())}}
    fn observe(&mut self)->Result<(),Reject>{
        let current=selection(&mut self.io).map_err(|e|if e==Reject::Framing{Reject::Changed}else{e})?;
        if current!=self.selected{return Err(Reject::Changed);}
        Ok(())
    }
    /// Caller must ignore all buffer contents on error and authenticate the
    /// returned exact span before interpreting it as a usable package.
    pub fn read_package(&mut self,slot:Slot,out:&mut[u8])->Result<usize,Reject>{
        self.open()?;
        if out.len()!=MAX_PACKAGE{return Err(Reject::Buffer);}
        let result=self.read_inner(slot,out);
        if matches!(result,Err(Reject::Io|Reject::Changed)){self.locked=true;}
        result
    }
    fn read_inner(&mut self,slot:Slot,out:&mut[u8])->Result<usize,Reject>{
        self.stream_inner(slot,&mut |_,offset,bytes|{
            out[offset..offset+bytes.len()].copy_from_slice(bytes);Ok(())
        }).map(|(length,_)|length)
    }
    /// Bounded streaming bridge to System STAGE_COPY: one sector on the stack.
    /// Sink receives total logical length, sequential offset and exact bytes.
    /// It must not activate or accept the partially copied package on failure.
    pub fn read_stream<F>(&mut self,slot:Slot,mut sink:F)->Result<(usize,[u8;32]),Reject>
        where F:FnMut(usize,usize,&[u8])->Result<(),()>
    {
        self.open()?;
        let result=self.stream_inner(slot,&mut sink);
        if matches!(result,Err(Reject::Io|Reject::Changed)){self.locked=true;}
        result
    }
    /// Only this transaction-bound route may feed an install/fallback staging
    /// sink. A generic read is never proof of pending transaction membership.
    pub fn copy_prepared<F>(&mut self,prepared:&Prepared,mut sink:F)->Result<(),Reject>
        where F:FnMut(usize,usize,&[u8])->Result<(),()>
    {
        self.open()?;
        self.matches(prepared)?;
        let result=(||{
            self.observe()?;
            let (length,hash)=self.stream_inner(prepared.identity.slot,&mut sink)?;
            if length!=prepared.identity.length||hash!=prepared.identity.package_hash{return Err(Reject::Changed);}
            self.observe()
        })();
        if result.is_err(){
            self.pending=None;
            // A sink prefix may already exist in the kernel. The caller must
            // explicitly abort/clear it; it must never finish or trial that seal.
            // Lock this transaction owner so no implicit retry can reuse it.
            self.locked=true;
        }
        result
    }
    fn matches(&self,prepared:&Prepared)->Result<(),Reject>{
        if prepared.before!=self.selected||self.pending!=Some((prepared.identity,prepared.purpose))||
            prepared.identity.transaction.checked_add(1)!=self.next{return Err(Reject::Policy);}
        Ok(())
    }
    /// Explicit cancellation only after the native owner removed any matching
    /// staging/trial state. No disk write, counter reset, retry or formatting.
    pub fn cancel(&mut self,prepared:&Prepared)->Result<(),Reject>{
        self.open()?;self.matches(prepared)?;self.pending=None;Ok(())
    }
    /// Bind a boot read to the intact selected record and exact package bytes.
    /// No inactive write or signature decision occurs; content rejection leaves
    /// an intact journal available for its one exact authorized prior fallback.
    pub fn prepare_boot(&mut self)->Result<Prepared,Reject>{
        self.open()?;
        if self.pending.is_some(){return Err(Reject::Policy);}
        let transaction=self.next.ok_or(Reject::Policy)?;
        let active=self.record().active();
        let result=(||{
            self.observe()?;
            let mut declared=None;
            let (length,package_hash)=self.stream_inner(active.slot(),&mut |_,offset,bytes|{
                if offset==0{
                    let m=manifest::Manifest::parse(&bytes[..manifest::SIZE]).map_err(|_|())?;
                    declared=Some((m.generation(),m.digest()));
                }Ok(())
            })?;
            if declared!=Some((active.generation(),active.digest())){return Err(Reject::Policy);}
            self.observe()?;
            let identity=Identity{transaction,slot:active.slot(),generation:active.generation(),
                digest:active.digest(),length,package_hash};
            self.next=transaction.checked_add(1);self.pending=Some((identity,Purpose::Boot));
            Ok(Prepared{before:self.selected,identity,purpose:Purpose::Boot})
        })();
        if matches!(result,Err(Reject::Io|Reject::Changed)){self.locked=true;}
        result
    }
    /// No selector write for successful boot of the already-selected package.
    /// The manager must still verify/health-check it before desktop publication.
    pub fn complete_boot(&mut self,prepared:&Prepared)->Result<(),Reject>{
        self.open()?;self.matches(prepared)?;
        if prepared.purpose!=Purpose::Boot{return Err(Reject::Policy);}
        let result=self.observe();self.pending=None;
        if result.is_err(){self.locked=true;}result
    }
    /// Read the exact committed prior; never overwrite it or lower high-water.
    /// Signature and health still belong to the sealed manager/native protocol.
    pub fn prepare_fallback(&mut self)->Result<Prepared,Reject>{
        self.open()?;
        if self.pending.is_some(){return Err(Reject::Policy);}
        let prior=self.record().previous().ok_or(Reject::Policy)?;
        let transaction=self.next.ok_or(Reject::Policy)?;
        let result=(||{
            self.observe()?;
            let mut declared=None;
            let (length,package_hash)=self.stream_inner(prior.slot(),&mut |_,offset,bytes|{
                if offset==0 {
                    let m=manifest::Manifest::parse(&bytes[..manifest::SIZE]).map_err(|_|())?;
                    declared=Some((m.generation(),m.digest()));
                }Ok(())
            })?;
            if declared!=Some((prior.generation(),prior.digest())){return Err(Reject::Policy);}
            self.observe()?;
            let identity=Identity{transaction,slot:prior.slot(),generation:prior.generation(),
                digest:prior.digest(),length,package_hash};
            self.next=transaction.checked_add(1);self.pending=Some((identity,Purpose::Fallback));
            Ok(Prepared{before:self.selected,identity,purpose:Purpose::Fallback})
        })();
        if result.is_err(){self.pending=None;self.locked=true;}
        result
    }
    fn stream_inner<F>(&mut self,slot:Slot,sink:&mut F)->Result<(usize,[u8;32]),Reject>
        where F:FnMut(usize,usize,&[u8])->Result<(),()>
    {
        let start=slot_start(slot);
        let first=self.io.read(start).map_err(|_|Reject::Io)?;
        let length=package_length(&first)?;
        let mut hash=Sha256::new();
        for index in 0..sectors(length) {
            let data=if index==0{first}else{
                self.io.read(start+index as u32).map_err(|_|Reject::Io)?
            };
            let count=core::cmp::min(512,length-index*512);
            if data[count..].iter().any(|&byte|byte!=0){return Err(Reject::Framing);}
            hash.update(&data[..count]).map_err(|_|Reject::Framing)?;
            sink(length,index*512,&data[..count]).map_err(|_|Reject::Sink)?;
        }
        Ok((length,hash.finalize()))
    }
    /// Raw framing is only an input screen. The manager must subsequently verify
    /// the kernel-sealed readback, never the original mutable source bytes.
    pub fn prepare(&mut self,package:&[u8])->Result<Prepared,Reject>{
        self.open()?;
        if self.pending.is_some(){return Err(Reject::Policy);}
        if !(896..=MAX_PACKAGE).contains(&package.len()){return Err(Reject::Framing);}
        let first:[u8;512]=package[..512].try_into().unwrap();
        if package_length(&first)?!=package.len(){return Err(Reject::Framing);}
        let parsed=manifest::Manifest::parse(&package[..manifest::SIZE]).map_err(|_|Reject::Framing)?;
        let minimum=self.record().minimum_install_generation().map_err(|_|Reject::Policy)?;
        if parsed.generation()<minimum||parsed.digest()==[0;32]{return Err(Reject::Policy);}
        let transaction=self.next.ok_or(Reject::Policy)?;
        let identity=Identity{transaction,slot:self.record().active().slot().other(),
            generation:parsed.generation(),digest:parsed.digest(),length:package.len(),
            package_hash:sha256(package).map_err(|_|Reject::Framing)?};
        self.next=transaction.checked_add(1);
        self.pending=None;
        let result=self.prepare_inner(package,identity);
        if result.is_err(){self.locked=true;}
        result
    }
    fn prepare_inner(&mut self,package:&[u8],identity:Identity)->Result<Prepared,Reject>{
        self.observe()?;
        let start=slot_start(identity.slot);
        let count=sectors(package.len());
        for index in 0..count{
            self.io.write(start+index as u32,&block(package,index)).map_err(|_|Reject::Io)?;
        }
        self.io.flush().map_err(|_|Reject::Io)?;
        for index in 0..count{
            let actual=self.io.read(start+index as u32).map_err(|_|Reject::Io)?;
            if actual!=block(package,index){return Err(Reject::Changed);}
        }
        self.observe()?;
        self.pending=Some((identity,Purpose::Install));
        Ok(Prepared{before:self.selected,identity,purpose:Purpose::Install})
    }
    /// Storage-only publication. The fixed System/Manager lifecycle protocol
    /// MUST bind transaction/seal identity, finish verified trial health and all
    /// fallible handover preparation first. A caller-supplied record alone cannot
    /// bypass package preparation: this checks the borrowed opaque token and consumes
    /// its pending identity before publication I/O. Policy rejection preserves
    /// the token for explicit cancellation; consumed tokens cannot be replayed.
    /// This function grants no execution or lifecycle authority.
    pub fn publish(&mut self,prepared:&Prepared,next:Record)->Result<(),Reject>{
        self.open()?;
        self.matches(prepared)?;
        if prepared.purpose==Purpose::Boot{return Err(Reject::Policy);}
        if prepared.purpose==Purpose::Fallback&&self.record().fallback().map_err(|_|Reject::Policy)?!=next{
            return Err(Reject::Policy);
        }
        let id=prepared.identity;
        if next.active().slot()!=id.slot||next.active().generation()!=id.generation||
            next.active().digest()!=id.digest{return Err(Reject::Policy);}
        self.pending=None;
        let result=self.publish_inner(prepared,next);
        if result.is_err(){self.locked=true;}
        result
    }
    fn publish_inner(&mut self,prepared:&Prepared,next:Record)->Result<(),Reject>{
        self.observe()?;
        let id=prepared.identity;
        let (length,hash)=self.stream_inner(id.slot,&mut |_,_,_|Ok(()))?;
        if length!=id.length||hash!=id.package_hash{
            return Err(Reject::Changed);
        }
        let selected_before=self.selected;
        let mut journal=Journal::mount(Selectors(&mut self.io)).map_err(|_|Reject::Changed)?;
        if journal.selection()!=selected_before{return Err(Reject::Changed);}
        journal.commit(next).map_err(|e|match e{
            journal::PublicationError::Indeterminate=>Reject::Indeterminate,_=>Reject::Changed
        })?;
        // Read the actual selected sector as well as the record; failure after
        // durable ACK is still indeterminate and cannot be retried in-process.
        let selected=selection(&mut self.io).map_err(|_|Reject::Indeterminate)?;
        if selected.record()!=next{return Err(Reject::Indeterminate);}
        self.selected=selected;Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn package_bounds_are_disjoint_and_do_not_touch_reserved_tail(){
        assert_eq!(slot_start(Slot::A),2);
        assert_eq!(slot_start(Slot::A)+SLOT_SECTORS,slot_start(Slot::B));
        assert_eq!(slot_start(Slot::B)+SLOT_SECTORS,FIRST_RESERVED);
        assert!(FIRST_RESERVED<SECTORS);
        assert_eq!(sectors(MAX_PACKAGE),SLOT_SECTORS as usize);
        assert_eq!(MAX_PACKAGE,2_097_536);
    }
    #[test]fn transaction_full_width_exhaustion_cannot_wrap_or_restage(){
        use std::collections::BTreeMap;
        struct Media{blocks:BTreeMap<u32,[u8;512]>,calls:usize}
        impl Io for Media{
            fn read(&mut self,s:u32)->Result<[u8;512],()>{self.calls+=1;Ok(*self.blocks.get(&s).unwrap_or(&[0;512]))}
            fn write(&mut self,s:u32,b:&[u8;512])->Result<(),()>{self.calls+=1;self.blocks.insert(s,*b);Ok(())}
            fn flush(&mut self)->Result<(),()>{self.calls+=1;Ok(())}
        }
        // Structurally valid public fixture record; no signature/health claim.
        let mut record=[0;512];record[..8].copy_from_slice(b"RARSYS00");
        record[10..12].copy_from_slice(&512u16.to_le_bytes());record[14]=255;
        for offset in [16usize,24,32,40]{record[offset..offset+8].copy_from_slice(&1u64.to_le_bytes());}
        record[64..96].fill(1);let hash=sha256(&record[..480]).unwrap();record[480..].copy_from_slice(&hash);
        let mut blocks=BTreeMap::new();blocks.insert(0,record);
        let mut volume=Volume::mount(Media{blocks,calls:0},SECTORS).unwrap();
        let mut package=vec![0;896];package[..8].copy_from_slice(b"RARMODL0");
        package[10..12].copy_from_slice(&384u16.to_le_bytes());package[12]=1;
        package[16..40].copy_from_slice(b"rar.alpha.ed25519.v0\0\0\0\0");
        package[56..60].copy_from_slice(&512u32.to_le_bytes());package[72]=2;package[288..320].fill(2);
        volume.next=Some(u64::MAX);
        let last=volume.prepare(&package).unwrap();assert_eq!(last.identity().transaction,u64::MAX);
        assert_eq!(volume.next,None);volume.copy_prepared(&last,|_,_,_|Ok(())).unwrap();
        let calls=volume.io.calls;assert!(matches!(volume.prepare(&package),Err(Reject::Policy)));
        assert_eq!(volume.io.calls,calls);assert_eq!(volume.next,None);
        let stale=Prepared{before:last.before,identity:Identity{transaction:1,..last.identity},purpose:Purpose::Install};
        assert_eq!(volume.copy_prepared(&stale,|_,_,_|panic!("stale sink")),Err(Reject::Policy));
        assert_eq!(volume.io.calls,calls);
    }

}
