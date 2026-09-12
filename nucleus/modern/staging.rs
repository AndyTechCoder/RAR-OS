//! Kernel-owned bounded byte staging for one Settings transaction.
//! No allocator, device, signature policy, page mapping or execution authority.
//! The caller must supply the exclusively owned guarded physical staging region.
//! Rust borrowing prevents safe aliases, NOT aliases in other address spaces.
#![forbid(unsafe_code)]

pub const MANIFEST_BYTES:usize=384;
pub const MIN_PACKAGE:usize=MANIFEST_BYTES+512;
pub const MAX_PACKAGE:usize=MANIFEST_BYTES+2*1024*1024;
/// Inspection includes full storage sectors, including the final zero padding.
pub const MIN_INSPECTION:usize=512;
pub const MAX_INSPECTION:usize=MAX_PACKAGE.div_ceil(512)*512;
pub const BUFFER_BYTES:usize=513*4096;
pub const MAX_CHUNK:usize=512;
pub const PRIVATE_PAGES:usize=9216;
pub const ARENA_PAGES:usize=PRIVATE_PAGES+513+2;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Region {pub lower_guard:u64,pub start:u64,pub end:u64,pub arena_end:u64}
pub fn guard_offset(offset:u64)->bool {
    offset==PRIVATE_PAGES as u64*4096||offset==(ARENA_PAGES as u64-1)*4096
}
/// Exact Modern-only extension. Existing 16 process strides end immediately
/// before lower_guard; neither guard belongs to the backing byte slice.
pub fn region(arena:u64,pages:usize)->Result<Region,Error>{
    if pages!=ARENA_PAGES||arena<0x2000000||arena%4096!=0{return Err(Error::Bounds);}
    let arena_end=arena.checked_add(ARENA_PAGES as u64*4096).ok_or(Error::Bounds)?;
    if arena_end>0x1_0000_0000{return Err(Error::Bounds);}
    let lower_guard=arena+PRIVATE_PAGES as u64*4096;
    let start=lower_guard+4096;let end=start+BUFFER_BYTES as u64;
    if end+4096!=arena_end{return Err(Error::Bounds);}
    Ok(Region{lower_guard,start,end,arena_end})
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Bounds,Busy,Stale,Order,Incomplete,Exhausted,State}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Identity {seal:u64,slot:u8,length:usize,purpose:Purpose}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Purpose {Executable,Inspection}
impl Identity {
    pub fn seal(self)->u64{self.seal}
    pub fn slot(self)->usize{self.slot as usize}
    pub fn length(self)->usize{self.length}
    pub fn inspection(self)->bool{self.purpose==Purpose::Inspection}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Phase {Empty,Copying,Sealed}
/// Lifetime owns every byte, including page-rounded padding. The owner must
/// remove all writable VM aliases before exposing a sealed verifier mapping,
/// and remove all verifier mappings before clear/reuse. This type cannot do so.
pub struct Buffer<'a> {
    bytes:&'a mut [u8],phase:Phase,identity:Option<Identity>,copied:usize,next:Option<u64>,
}
impl<'a> Buffer<'a> {
    /// Construct once per kernel boot session. Do not reconstruct to recover a
    /// failed transaction: that would reset seals while stale requests exist.
    pub fn new(bytes:&'a mut [u8])->Result<Self,Error>{
        if bytes.len()!=BUFFER_BYTES{return Err(Error::Bounds);}
        bytes.fill(0);
        Ok(Self{bytes,phase:Phase::Empty,identity:None,copied:0,next:Some(1)})
    }
    /// Slot comes from the kernel's logically-vacant AND physically-Clean
    /// reservation. Never pass a user-selected slot or a merely dead process.
    /// All rejection checks precede mutation. One full-u64 seal is consumed for
    /// each accepted begin; abort cannot return it to the sequence.
    pub fn begin(&mut self,slot:usize,length:usize)->Result<Identity,Error>{
        self.begin_kind(slot,length,Purpose::Executable)
    }
    /// Read-only storage inspection. This purpose cannot be converted into an
    /// executable package view, even when its length also fits package bounds.
    /// Native integration must separately admit the inspection-only syscall.
    pub(crate) fn begin_inspection(&mut self,slot:usize,length:usize)->Result<Identity,Error>{
        self.begin_kind(slot,length,Purpose::Inspection)
    }
    fn begin_kind(&mut self,slot:usize,length:usize,purpose:Purpose)->Result<Identity,Error>{
        if self.phase!=Phase::Empty{return Err(Error::Busy);}
        let bounded=match purpose{
            Purpose::Executable=>(MIN_PACKAGE..=MAX_PACKAGE).contains(&length),
            Purpose::Inspection=>(MIN_INSPECTION..=MAX_INSPECTION).contains(&length)&&length%512==0,
        };
        if !matches!(slot,5|7)||!bounded{return Err(Error::Bounds);}
        let seal=self.next.ok_or(Error::Exhausted)?;
        let id=Identity{seal,slot:slot as u8,length,purpose};
        self.next=seal.checked_add(1);
        self.identity=Some(id);self.copied=0;self.phase=Phase::Copying;
        Ok(id)
    }
    fn identity(&self,seal:u64)->Result<Identity,Error>{
        self.identity.filter(|id|seal!=0&&id.seal==seal).ok_or(Error::Stale)
    }
    /// Exact sequential copies only, no holes, replay, truncation or overlap.
    /// A rejected request preserves both bytes and progress.
    pub fn append(&mut self,seal:u64,offset:usize,chunk:&[u8])->Result<(),Error>{
        let id=self.identity(seal)?;
        if self.phase!=Phase::Copying{return Err(Error::State);}
        if offset!=self.copied{return Err(Error::Order);}
        let end=offset.checked_add(chunk.len()).ok_or(Error::Bounds)?;
        if chunk.is_empty()||chunk.len()>MAX_CHUNK||end>id.length{return Err(Error::Bounds);}
        self.bytes[offset..end].copy_from_slice(chunk);self.copied=end;Ok(())
    }
    /// Finishes byte copying only. The native adapter must remove writable
    /// aliases and invalidate them before it publishes a manager view/seal.
    /// No manager authority or authenticated metadata is produced here.
    pub fn finish(&mut self,seal:u64)->Result<Identity,Error>{
        let id=self.identity(seal)?;
        if self.phase!=Phase::Copying{return Err(Error::State);}
        if self.copied!=id.length{return Err(Error::Incomplete);}
        if self.bytes[id.length..].iter().any(|&b|b!=0){return Err(Error::State);}
        self.phase=Phase::Sealed;Ok(id)
    }
    /// Shared borrow of exactly the logical package, excluding zero padding.
    /// The lifetime prevents clear/append while this safe Rust view is live.
    pub fn view(&self,seal:u64)->Result<&[u8],Error>{
        let id=self.identity(seal)?;
        if self.phase!=Phase::Sealed||id.purpose!=Purpose::Executable{return Err(Error::State);}
        Ok(&self.bytes[..id.length])
    }
    /// Non-executable inspection bytes. No signature, damage or freshness
    /// authority is inferred from the kernel's successful byte-copy seal.
    pub(crate) fn inspection_view(&self,seal:u64)->Result<&[u8],Error>{
        let id=self.identity(seal)?;
        if self.phase!=Phase::Sealed||id.purpose!=Purpose::Inspection{return Err(Error::State);}
        Ok(&self.bytes[..id.length])
    }
    pub fn reserved(&self)->Option<Identity>{self.identity}
    pub fn copying(&self,seal:u64)->Result<Identity,Error>{
        let id=self.identity(seal)?;if self.phase!=Phase::Copying{return Err(Error::State);}Ok(id)
    }
    /// Native caller removes all external aliases before supplying a non-elidable
    /// eraser. The mutable byte borrow keeps raw scrub pointers derived from this
    /// allocation owner; a failed scrub never publishes Empty or resets seals.
    pub fn clear_with<F:FnOnce(&mut[u8])>(&mut self,seal:u64,erase:F)->Result<(),Error>{
        self.identity(seal)?;erase(self.bytes);
        if self.bytes.iter().any(|&b|b!=0){return Err(Error::State);}
        self.phase=Phase::Empty;self.identity=None;self.copied=0;Ok(())
    }
    /// Only after the native adapter has removed all external mappings.
    /// Clears the full buffer (not just the latest package). This is byte-level
    /// reuse hygiene; architectural TLB/alias retirement is a separate duty.
    pub fn clear(&mut self,seal:u64)->Result<(),Error>{
        self.identity(seal)?;
        self.bytes.fill(0);
        self.phase=Phase::Empty;self.identity=None;self.copied=0;Ok(())
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn fill(b:&mut Buffer<'_>,id:Identity,value:u8){
        let chunk=[value;MAX_CHUNK];let mut offset=0;
        while offset<id.length(){
            let n=MAX_CHUNK.min(id.length()-offset);
            b.append(id.seal(),offset,&chunk[..n]).unwrap();offset+=n;
        }
    }
    #[test] fn native_region_is_guarded_disjoint_and_bounded(){
        assert_eq!(ARENA_PAGES,9731);
        for arena in [0x2000000,0x4000000,0x1_0000_0000-ARENA_PAGES as u64*4096]{
            let r=region(arena,ARENA_PAGES).unwrap();
            assert_eq!(r.lower_guard,arena+0x2400000);
            assert_eq!(r.start%4096,0);assert_eq!(r.end-r.start,BUFFER_BYTES as u64);
            assert!(guard_offset(r.lower_guard-arena));assert!(guard_offset(r.end-arena));
            assert!(!guard_offset(r.start-arena));assert!(!guard_offset(r.end-arena-4096));
            assert_eq!(r.arena_end-r.end,4096);
            assert_eq!((0..ARENA_PAGES).filter(|&p|guard_offset(p as u64*4096)).count(),2);
        }
        for (arena,pages) in [(0,ARENA_PAGES),(0x2000001,ARENA_PAGES),
            (0x2000000,9216),(0x2000000,ARENA_PAGES-1),(0x2000000,ARENA_PAGES+1),
            (u64::MAX-4095,ARENA_PAGES),(0xfffff000,ARENA_PAGES)]{
            assert_eq!(region(arena,pages),Err(Error::Bounds));
        }
    }
    #[test] fn exact_maximum_package_and_padding_survive_seal(){
        assert_eq!(MAX_PACKAGE,2_097_536);assert_eq!(BUFFER_BYTES,2_101_248);
        let mut bytes=vec![0xff;BUFFER_BYTES];
        let mut b=Buffer::new(&mut bytes).unwrap();
        let id=b.begin(7,MAX_PACKAGE).unwrap();fill(&mut b,id,0x42);
        assert_eq!(b.finish(id.seal()),Ok(id));
        assert_eq!(b.view(id.seal()).unwrap().len(),MAX_PACKAGE);
        assert!(b.view(id.seal()).unwrap().iter().all(|&x|x==0x42));
        assert!(b.bytes[MAX_PACKAGE..].iter().all(|&x|x==0));
        assert_eq!(b.reserved(),Some(id));
        assert_eq!(b.append(id.seal(),0,b"x"),Err(Error::State));
        assert_eq!(b.finish(id.seal()),Err(Error::State));
    }
    #[test] fn malformed_copy_never_advances_or_changes_bytes(){
        let mut bytes=vec![0;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        let id=b.begin(5,MIN_PACKAGE).unwrap();
        assert_eq!(b.view(id.seal()),Err(Error::State));
        assert_eq!(b.finish(id.seal()),Err(Error::Incomplete));
        for (seal,offset,data,error) in [
            (0,0,b"x".as_slice(),Error::Stale),
            (id.seal()+1,0,b"x",Error::Stale),
            (id.seal(),1,b"x",Error::Order),
            (id.seal(),usize::MAX,b"x",Error::Order),
            (id.seal(),0,b"",Error::Bounds),
            (id.seal(),0,&[1;MAX_CHUNK+1],Error::Bounds),
        ]{
            assert_eq!(b.append(seal,offset,data),Err(error));
            assert_eq!(b.copied,0);assert!(b.bytes.iter().all(|&x|x==0));
        }
        b.append(id.seal(),0,&[7;512]).unwrap();
        assert_eq!(b.append(id.seal(),0,b"x"),Err(Error::Order));
        assert_eq!(b.append(id.seal(),512,&[8;512]),Err(Error::Bounds));
        assert_eq!(b.copied,512);
        assert!(b.bytes[..512].iter().all(|&x|x==7));
        assert!(b.bytes[512..].iter().all(|&x|x==0));
        assert_eq!(b.reserved(),Some(id));
    }
    #[test] fn begin_is_bounded_one_at_a_time_and_abort_does_not_reuse_seal(){
        for n in [0,BUFFER_BYTES-1,BUFFER_BYTES+1]{
            let mut bytes=vec![5;n];assert!(matches!(Buffer::new(&mut bytes),Err(Error::Bounds)));
            assert!(bytes.iter().all(|&x|x==5));
        }
        let mut bytes=vec![3;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        for slot in [0,4,6,8,15,16,usize::MAX]{
            assert_eq!(b.begin(slot,MIN_PACKAGE),Err(Error::Bounds));
        }
        for n in [0,MIN_PACKAGE-1,MAX_PACKAGE+1,usize::MAX]{
            assert_eq!(b.begin(7,n),Err(Error::Bounds));
        }
        let old=b.begin(7,MAX_PACKAGE).unwrap();fill(&mut b,old,0xab);
        assert_eq!(old.seal(),1);
        assert_eq!(b.begin(5,MIN_PACKAGE),Err(Error::Busy));
        assert_eq!(b.clear(old.seal()+1),Err(Error::Stale));
        b.clear(old.seal()).unwrap();assert!(b.bytes.iter().all(|&x|x==0));
        let new=b.begin(5,MIN_PACKAGE).unwrap();
        assert!(new.seal()>old.seal());assert_eq!(new.slot(),5);
        assert_eq!(b.append(old.seal(),0,b"x"),Err(Error::Stale));
        fill(&mut b,new,9);b.finish(new.seal()).unwrap();
        assert!(b.bytes[MIN_PACKAGE..].iter().all(|&x|x==0));
        b.clear(new.seal()).unwrap();assert!(b.bytes.iter().all(|&x|x==0));
        assert_eq!(b.clear(new.seal()),Err(Error::Stale));
    }
    #[test] fn full_width_seals_exhaust_without_wrap_or_revival(){
        let mut bytes=vec![0;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        b.next=Some((1u64<<32)|1);let id=b.begin(7,MIN_PACKAGE).unwrap();
        assert_eq!(b.append(1,0,b"x"),Err(Error::Stale));
        b.clear(id.seal()).unwrap();b.next=Some(u64::MAX);
        let last=b.begin(5,MIN_PACKAGE).unwrap();assert_eq!(last.seal(),u64::MAX);
        b.clear(last.seal()).unwrap();
        assert_eq!(b.begin(7,MIN_PACKAGE),Err(Error::Exhausted));
        assert_eq!(b.reserved(),None);assert!(b.bytes.iter().all(|&x|x==0));
    }
    #[test]fn native_eraser_must_clear_full_buffer_before_empty(){
        let mut bytes=vec![0;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        let id=b.begin(7,MIN_PACKAGE).unwrap();b.append(id.seal(),0,&[7;512]).unwrap();
        assert_eq!(b.copying(id.seal()),Ok(id));
        assert_eq!(b.clear_with(id.seal(),|_|{}),Err(Error::State));
        assert_eq!(b.reserved(),Some(id));
        b.clear_with(id.seal(),|bytes|bytes.fill(0)).unwrap();
        assert_eq!(b.reserved(),None);
        let next=b.begin(7,MIN_PACKAGE).unwrap();assert!(next.seal()>id.seal());
        fill(&mut b,next,1);b.finish(next.seal()).unwrap();
        assert_eq!(b.copying(next.seal()),Err(Error::State));
        assert_eq!(b.clear_with(id.seal(),|_|panic!("stale eraser called")),Err(Error::Stale));
    }

    #[test]fn inspection_seals_never_become_executable_views(){
        assert_eq!(MAX_INSPECTION,2_097_664);
        assert!(MAX_INSPECTION<BUFFER_BYTES);
        let mut bytes=vec![0;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        for n in [MIN_INSPECTION,1024,MAX_INSPECTION]{
            let id=b.begin_inspection(7,n).unwrap();assert!(id.inspection());
            assert_eq!(b.inspection_view(id.seal()),Err(Error::State));
            fill(&mut b,id,0x5a);b.finish(id.seal()).unwrap();
            assert_eq!(b.inspection_view(id.seal()).unwrap().len(),n);
            assert_eq!(b.view(id.seal()),Err(Error::State));
            assert_eq!(b.begin(5,MIN_PACKAGE),Err(Error::Busy));
            b.clear(id.seal()).unwrap();
            assert_eq!(b.inspection_view(id.seal()),Err(Error::Stale));
            assert!(b.bytes.iter().all(|&byte|byte==0));
        }
        let id=b.begin(5,MIN_PACKAGE).unwrap();assert!(!id.inspection());
        fill(&mut b,id,1);b.finish(id.seal()).unwrap();
        assert_eq!(b.inspection_view(id.seal()),Err(Error::State));
        assert_eq!(b.view(id.seal()).unwrap().len(),MIN_PACKAGE);
    }
    #[test]fn inspection_bounds_reject_partial_sector_and_do_not_consume_seal(){
        let mut bytes=vec![0;BUFFER_BYTES];let mut b=Buffer::new(&mut bytes).unwrap();
        for length in [0,511,513,896,MAX_INSPECTION-1,MAX_INSPECTION+1,usize::MAX]{
            assert_eq!(b.begin_inspection(7,length),Err(Error::Bounds));
            assert_eq!(b.reserved(),None);assert_eq!(b.next,Some(1));
        }
        for slot in [0,4,6,8,usize::MAX]{
            assert_eq!(b.begin_inspection(slot,512),Err(Error::Bounds));
        }
        let id=b.begin_inspection(5,512).unwrap();
        b.append(id.seal(),0,&[1;511]).unwrap();
        assert_eq!(b.finish(id.seal()),Err(Error::Incomplete));
        assert_eq!(b.inspection_view(id.seal()),Err(Error::State));
        assert_eq!(b.view(id.seal()),Err(Error::State));
        b.clear(id.seal()).unwrap();
        let next=b.begin(7,MIN_PACKAGE).unwrap();assert!(next.seal()>id.seal());
        assert_eq!(b.append(id.seal(),0,b"x"),Err(Error::Stale));
    }

}
