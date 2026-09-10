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
pub struct Prepared {before:Selection,identity:Identity}
impl Prepared {pub fn identity(&self)->Identity{self.identity}}
pub struct Volume<I:Io>{io:I,selected:Selection,locked:bool,next:Option<u64>,pending:Option<Identity>}
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
        let current=selection(&mut self.io)?;
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
    pub fn copy_package<F>(&mut self,slot:Slot,mut sink:F)->Result<(usize,[u8;32]),Reject>
        where F:FnMut(usize,usize,&[u8])->Result<(),()>
    {
        self.open()?;
        let result=self.stream_inner(slot,&mut sink);
        if matches!(result,Err(Reject::Io|Reject::Changed)){self.locked=true;}
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
        self.pending=Some(identity);
        Ok(Prepared{before:self.selected,identity})
    }
    /// Storage-only publication. The fixed System/Manager lifecycle protocol
    /// MUST bind transaction/seal identity, finish verified trial health and all
    /// fallible handover preparation first. A caller-supplied record alone cannot
    /// bypass package preparation: this consumes and checks the opaque token.
    /// This function grants no execution or lifecycle authority.
    pub fn publish(&mut self,prepared:Prepared,next:Record)->Result<(),Reject>{
        self.open()?;
        if prepared.before!=self.selected||self.pending!=Some(prepared.identity){return Err(Reject::Policy);}
        // A newer prepare invalidates every outstanding older token, even when
        // it staged byte-identical content. Counter exhaustion does not wrap.
        if prepared.identity.transaction.checked_add(1)!=self.next{return Err(Reject::Policy);}
        let id=prepared.identity;
        if next.active().slot()!=id.slot||next.active().generation()!=id.generation||
            next.active().digest()!=id.digest{return Err(Reject::Policy);}
        self.pending=None;
        let result=self.publish_inner(prepared,next);
        if result.is_err(){self.locked=true;}
        result
    }
    fn publish_inner(&mut self,prepared:Prepared,next:Record)->Result<(),Reject>{
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
}
