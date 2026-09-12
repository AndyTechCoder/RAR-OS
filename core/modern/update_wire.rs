//! Private fixed System/manager update channel. Parsing is not authority:
//! the receiver must first check the kernel-stamped peer and full incarnation.
#![forbid(unsafe_code)]
use crate::{journal::{Record,Slot},system_volume::Identity};
pub const BYTES:usize=128;
pub const PART:usize=88;
const MAGIC:&[u8;8]=b"RARUPD01";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Mode{Boot,Install,Fallback,Repair}
impl Mode{
    fn byte(self)->u8{match self{Self::Boot=>0,Self::Install=>1,Self::Fallback=>2,Self::Repair=>3}}
    fn parse(v:u8)->Option<Self>{match v{0=>Some(Self::Boot),1=>Some(Self::Install),2=>Some(Self::Fallback),3=>Some(Self::Repair),_=>None}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Kind{Offer,RecordGet,RecordPart,PublishPart,PartAck,Commit,Committed,Cancel,Cancelled,Start,Rejected}
impl Kind{
    fn byte(self)->u8{match self{Self::Offer=>1,Self::RecordGet=>2,Self::RecordPart=>3,
        Self::PublishPart=>4,Self::PartAck=>5,Self::Commit=>6,Self::Committed=>7,
        Self::Cancel=>8,Self::Cancelled=>9,Self::Start=>10,Self::Rejected=>11}}
    fn parse(v:u8)->Option<Self>{match v{1=>Some(Self::Offer),2=>Some(Self::RecordGet),3=>Some(Self::RecordPart),
        4=>Some(Self::PublishPart),5=>Some(Self::PartAck),6=>Some(Self::Commit),7=>Some(Self::Committed),
        8=>Some(Self::Cancel),9=>Some(Self::Cancelled),10=>Some(Self::Start),11=>Some(Self::Rejected),_=>None}}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Encoding,Identity,Order,Exhausted,Record}
fn word(b:&[u8;128],p:usize)->u64{u64::from_le_bytes(b[p..p+8].try_into().unwrap())}
fn put(b:&mut[u8;128],p:usize,v:u64){b[p..p+8].copy_from_slice(&v.to_le_bytes());}
fn base(kind:Kind,mode:Mode,request:u64,seal:u64,argument:u64)->Result<[u8;128],Error>{
    if request==0{return Err(Error::Identity);}
    let mut b=[0;128];b[..8].copy_from_slice(MAGIC);b[8]=kind.byte();b[9]=mode.byte();
    put(&mut b,16,request);put(&mut b,24,seal);put(&mut b,32,argument);Ok(b)
}
fn header(b:&[u8])->Result<(&[u8;128],Kind,Mode),Error>{
    let b:&[u8;128]=b.try_into().map_err(|_|Error::Encoding)?;
    if &b[..8]!=MAGIC||b[10..16]!=[0;6]||word(b,16)==0{return Err(Error::Encoding);}
    Ok((b,Kind::parse(b[8]).ok_or(Error::Encoding)?,Mode::parse(b[9]).ok_or(Error::Encoding)?))
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Transfer{pub mode:Mode,pub request:u64,pub seal:u64,pub sequence:u64,pub identity:Identity}
impl Transfer{
    pub fn frame(self,kind:Kind)->Result<[u8;128],Error>{
        if !matches!(kind,Kind::Offer|Kind::Commit|Kind::Committed|Kind::Cancel|Kind::Cancelled)||
            self.seal==0||self.sequence==0||self.identity.transaction==0||self.identity.generation==0||
            self.identity.digest==[0;32]||self.identity.package_hash==[0;32]||
            !(896..=2_097_536).contains(&self.identity.length){return Err(Error::Identity);}
        let mut b=base(kind,self.mode,self.request,self.seal,self.sequence)?;
        put(&mut b,40,self.identity.transaction);put(&mut b,48,self.identity.generation);
        b[56..60].copy_from_slice(&(self.identity.length as u32).to_le_bytes());
        b[60]=match self.identity.slot{Slot::A=>0,Slot::B=>1};
        b[64..96].copy_from_slice(&self.identity.digest);b[96..].copy_from_slice(&self.identity.package_hash);Ok(b)
    }
    pub fn parse(raw:&[u8],wanted:Kind)->Result<Self,Error>{
        let (b,kind,mode)=header(raw)?;
        if kind!=wanted||b[61..64]!=[0;3]{return Err(Error::Encoding);}
        let value=Self{mode,request:word(b,16),seal:word(b,24),sequence:word(b,32),
            identity:Identity{transaction:word(b,40),generation:word(b,48),
                length:u32::from_le_bytes(b[56..60].try_into().unwrap())as usize,
                slot:match b[60]{0=>Slot::A,1=>Slot::B,_=>return Err(Error::Encoding)},
                digest:b[64..96].try_into().unwrap(),package_hash:b[96..].try_into().unwrap()}};
        if value.frame(wanted)?!=*b{return Err(Error::Encoding);}Ok(value)
    }
    pub fn matches(self,raw:&[u8],wanted:Kind)->bool{Self::parse(raw,wanted)==Ok(self)}
    pub fn committed(self,sequence:u64)->Result<Self,Error>{
        let expected=if self.mode==Mode::Boot{self.sequence}else{self.sequence.checked_add(1).ok_or(Error::Exhausted)?};
        if sequence!=expected{return Err(Error::Identity);}Ok(Self{sequence,..self})
    }
    pub fn part(self,kind:Kind,offset:usize,bytes:&[u8])->Result<[u8;128],Error>{
        if self.seal==0||offset>=512||offset%PART!=0||
            !matches!(kind,Kind::RecordGet|Kind::RecordPart|Kind::PublishPart|Kind::PartAck){
            return Err(Error::Order);
        }
        let n=(512-offset).min(PART);
        let has_data=matches!(kind,Kind::RecordPart|Kind::PublishPart);
        if bytes.len()!=if has_data{n}else{0}{return Err(Error::Encoding);}
        let mut b=base(kind,self.mode,self.request,self.seal,offset as u64)?;
        b[40..40+bytes.len()].copy_from_slice(bytes);Ok(b)
    }
    pub fn check_part<'a>(self,raw:&'a[u8],kind:Kind,offset:usize)->Result<&'a[u8],Error>{
        let (b,k,mode)=header(raw)?;
        if k!=kind||mode!=self.mode||word(b,16)!=self.request||word(b,24)!=self.seal||
            word(b,32)!=offset as u64{return Err(Error::Identity);}
        if offset>=512||offset%PART!=0{return Err(Error::Order);}
        let n=if matches!(kind,Kind::RecordPart|Kind::PublishPart){(512-offset).min(PART)}else{0};
        if self.part(kind,offset,&b[40..40+n])?!=*b{return Err(Error::Encoding);}
        Ok(&b[40..40+n])
    }
}
/// Manager-origin operation: a new full-width request, no caller seal/slot/LBA.
/// Install selects a bounded immutable laboratory input index, not a host path.
pub fn request(kind:Kind,mode:Mode,id:u64,index:u64)->Result<[u8;128],Error>{
    if !matches!(kind,Kind::Start|Kind::Rejected)||index>7||(mode!=Mode::Install&&index!=0){
        return Err(Error::Encoding);
    }
    base(kind,mode,id,0,index)
}
pub fn parse_request(raw:&[u8],wanted:Kind)->Result<(Mode,u64,u64),Error>{
    let (b,k,mode)=header(raw)?;let id=word(b,16);let index=word(b,32);
    if k!=wanted||request(wanted,mode,id,index)?!=*b{return Err(Error::Encoding);}
    Ok((mode,id,index))
}
/// Separate monotonic request identity; storage preparation has its own counter.
pub struct Requests{next:Option<u64>}
impl Requests{
    pub const fn new()->Self{Self{next:Some(1)}}
    pub fn next(&self)->Result<u64,Error>{self.next.ok_or(Error::Exhausted)}
    pub fn accept(&mut self,id:u64)->Result<(),Error>{
        if self.next()!=Ok(id){return Err(Error::Order);}
        self.next=id.checked_add(1);Ok(())
    }
}
/// Six exact sequential records fragments. A malformed/mismatched fragment
/// cannot advance assembly or be combined with a later transaction.
pub struct RecordReceiver{transfer:Transfer,kind:Kind,offset:usize,bytes:[u8;512]}
impl RecordReceiver{
    pub fn new(transfer:Transfer,kind:Kind)->Result<Self,Error>{
        transfer.frame(Kind::Offer)?;
        if !matches!(kind,Kind::RecordPart|Kind::PublishPart){return Err(Error::Encoding);}
        Ok(Self{transfer,kind,offset:0,bytes:[0;512]})
    }
    pub fn offset(&self)->usize{self.offset}
    pub fn push(&mut self,frame:&[u8])->Result<(),Error>{
        let part=self.transfer.check_part(frame,self.kind,self.offset)?;
        self.bytes[self.offset..self.offset+part.len()].copy_from_slice(part);self.offset+=part.len();Ok(())
    }
    pub fn finish(self)->Result<Record,Error>{
        if self.offset!=512{return Err(Error::Order);}
        Record::decode(&self.bytes).map_err(|_|Error::Record)
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn transfer()->Transfer{Transfer{mode:Mode::Install,request:(1u64<<40)+1,seal:u64::MAX,
        sequence:4,identity:Identity{transaction:(1u64<<42)+2,slot:Slot::B,generation:7,
            length:896,digest:[1;32],package_hash:[2;32]}}}
    #[test]fn full_transaction_identity_and_exact_ack_are_not_booleans(){
        let t=transfer();let f=t.frame(Kind::Offer).unwrap();
        assert_eq!(Transfer::parse(&f,Kind::Offer),Ok(t));
        for n in 0..128{assert!(Transfer::parse(&f[..n],Kind::Offer).is_err());}
        assert!(!t.matches(&f,Kind::Committed));
        for p in 0..128{let mut bad=f;bad[p]^=1;assert!(!t.matches(&bad,Kind::Offer));}
        let next=t.committed(5).unwrap();assert!(next.matches(&next.frame(Kind::Committed).unwrap(),Kind::Committed));
        assert!(t.committed(4).is_err());assert!(t.committed(6).is_err());
        let exhausted=Transfer{sequence:u64::MAX,..t};assert!(exhausted.committed(1).is_err());
        let boot=Transfer{mode:Mode::Boot,..t};assert!(boot.committed(4).is_ok());
    }
    #[test]fn fragments_are_sequential_exact_correlated_and_zero_padded(){
        let t=transfer();let mut r=RecordReceiver::new(t,Kind::RecordPart).unwrap();
        let bytes=[3u8;512];
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);let f=t.part(Kind::RecordPart,offset,&bytes[offset..offset+n]).unwrap();
            for p in [0usize,8,9,10,16,24,32]{let mut bad=f;bad[p]^=1;
                assert!(r.push(&bad).is_err());assert_eq!(r.offset(),offset);
            }
            if offset==440{let mut bad=f;bad[127]=1;assert!(r.push(&bad).is_err());}
            r.push(&f).unwrap();assert!(r.push(&f).is_err());
        }
        assert_eq!(r.offset(),512);assert_eq!(r.finish(),Err(Error::Record));
        assert!(t.part(Kind::RecordPart,512,&[]).is_err());assert!(t.part(Kind::RecordPart,1,&[0;88]).is_err());
        assert!(t.part(Kind::RecordGet,0,&[0]).is_err());
    }
    #[test]fn request_counter_refuses_replay_reorder_and_full_width_wrap(){
        let mut r=Requests::new();assert_eq!(r.next(),Ok(1));
        assert!(r.accept(0).is_err());assert!(r.accept(2).is_err());r.accept(1).unwrap();
        assert!(r.accept(1).is_err());assert_eq!(r.next(),Ok(2));
        r.next=Some(u64::MAX);r.accept(u64::MAX).unwrap();
        assert_eq!(r.next(),Err(Error::Exhausted));assert!(r.accept(1).is_err());
        for mode in [Mode::Boot,Mode::Install,Mode::Fallback]{
            let frame=request(Kind::Start,mode,1,0).unwrap();
            assert_eq!(parse_request(&frame,Kind::Start),Ok((mode,1,0)));
            let mut bad=frame;bad[24]=1;assert!(parse_request(&bad,Kind::Start).is_err());
        }
        assert!(request(Kind::Start,Mode::Boot,1,1).is_err());
        assert!(request(Kind::Start,Mode::Install,1,8).is_err());
    }
}
