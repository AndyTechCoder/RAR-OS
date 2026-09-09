//! Kernel-owned bounded byte staging for one Settings transaction.
//! No allocator, device, signature policy, page mapping or execution authority.
//! The caller must supply the exclusively owned guarded physical staging region.
//! Rust borrowing prevents safe aliases, NOT aliases in other address spaces.
#![forbid(unsafe_code)]

pub const MANIFEST_BYTES:usize=384;
pub const MIN_PACKAGE:usize=MANIFEST_BYTES+512;
pub const MAX_PACKAGE:usize=MANIFEST_BYTES+2*1024*1024;
pub const BUFFER_BYTES:usize=513*4096;
pub const MAX_CHUNK:usize=512;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Bounds,Busy,Stale,Order,Incomplete,Exhausted,State}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Identity {seal:u64,slot:u8,length:usize}
impl Identity {
    pub fn seal(self)->u64{self.seal}
    pub fn slot(self)->usize{self.slot as usize}
    pub fn length(self)->usize{self.length}
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
        if self.phase!=Phase::Empty{return Err(Error::Busy);}
        if !matches!(slot,5|7)||!(MIN_PACKAGE..=MAX_PACKAGE).contains(&length){
            return Err(Error::Bounds);
        }
        let seal=self.next.ok_or(Error::Exhausted)?;
        let id=Identity{seal,slot:slot as u8,length};
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
        if self.phase!=Phase::Sealed{return Err(Error::State);}
        Ok(&self.bytes[..id.length])
    }
    pub fn reserved(&self)->Option<Identity>{self.identity}
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
}
