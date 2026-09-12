//! Private bootstrap repair channel. Decoding is NOT authentication or repair authority.
//! Receivers must authenticate fixed System9/Manager8 and the full incarnation first.
#![forbid(unsafe_code)]
use crate::{journal::{Record,Slot},sha256::sha256};
pub const BYTES:usize=128;
pub const PART:usize=88;
pub const MAX_INSPECTION:usize=2_097_664;
const MAGIC:&[u8;8]=b"RARREP01";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Encoding,Identity,Order,Record}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Phase{Active,Prior,Factory,FreshActive,FreshPrior,FreshFactory}
impl Phase{
    fn byte(self)->u8{match self{Self::Active=>0,Self::Prior=>1,Self::Factory=>2,
        Self::FreshActive=>3,Self::FreshPrior=>4,Self::FreshFactory=>5}}
    fn parse(b:u8)->Result<Self,Error>{match b{0=>Ok(Self::Active),1=>Ok(Self::Prior),
        2=>Ok(Self::Factory),3=>Ok(Self::FreshActive),4=>Ok(Self::FreshPrior),
        5=>Ok(Self::FreshFactory),_=>Err(Error::Encoding)}}
    pub fn next(self,has_prior:bool)->Option<Self>{match self{
        Self::Active=>Some(if has_prior{Self::Prior}else{Self::Factory}),
        Self::Prior=>Some(Self::Factory),Self::Factory=>Some(Self::FreshActive),
        Self::FreshActive=>Some(if has_prior{Self::FreshPrior}else{Self::FreshFactory}),
        Self::FreshPrior=>Some(Self::FreshFactory),Self::FreshFactory=>None}}
}
fn word(b:&[u8;BYTES],at:usize)->u64{u64::from_le_bytes(b[at..at+8].try_into().unwrap())}
fn put(b:&mut[u8;BYTES],at:usize,value:u64){b[at..at+8].copy_from_slice(&value.to_le_bytes());}
fn base(kind:u8,request:u64)->Result<[u8;BYTES],Error>{
    if request==0{return Err(Error::Identity);}
    let mut b=[0;BYTES];b[..8].copy_from_slice(MAGIC);b[8]=kind;put(&mut b,16,request);Ok(b)
}
fn header(raw:&[u8],kind:u8)->Result<&[u8;BYTES],Error>{
    let b:&[u8;BYTES]=raw.try_into().map_err(|_|Error::Encoding)?;
    if &b[..8]!=MAGIC||b[8]!=kind||word(b,16)==0{return Err(Error::Encoding);}Ok(b)
}
/// Begin one bootstrap transaction. No index, host path, LBA or caller seal.
pub fn start(request:u64)->Result<[u8;BYTES],Error>{base(1,request)}
pub fn parse_start(raw:&[u8])->Result<u64,Error>{
    let b=header(raw,1)?;let request=word(b,16);
    if start(request)?!=*b{return Err(Error::Encoding);}Ok(request)
}
/// Untrusted snapshot identity, checked against all512 assembled record bytes.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Snapshot{pub request:u64,pub sequence:u64,pub record_hash:[u8;32]}
impl Snapshot{
    pub fn from_record(request:u64,record:Record)->Result<Self,Error>{
        let value=Self{request,sequence:record.sequence(),
            record_hash:sha256(&record.encode()).map_err(|_|Error::Record)?};
        value.frame()?;Ok(value)
    }
    pub fn frame(self)->Result<[u8;BYTES],Error>{
        if self.sequence==0||self.record_hash==[0;32]{return Err(Error::Identity);}
        let mut b=base(2,self.request)?;put(&mut b,32,self.sequence);
        b[40..72].copy_from_slice(&self.record_hash);Ok(b)
    }
    pub fn parse(raw:&[u8])->Result<Self,Error>{
        let b=header(raw,2)?;let value=Self{request:word(b,16),sequence:word(b,32),
            record_hash:b[40..72].try_into().unwrap()};
        if value.frame()?!=*b{return Err(Error::Encoding);}Ok(value)
    }
    pub fn matches_record(self,record:Record)->bool{Self::from_record(self.request,record)==Ok(self)}
    pub fn part(self,offset:usize,bytes:Option<&[u8]>)->Result<[u8;BYTES],Error>{
        self.frame()?;
        if offset>=512||offset%PART!=0{return Err(Error::Order);}
        let n=(512-offset).min(PART);
        if bytes.is_some_and(|b|b.len()!=n){return Err(Error::Encoding);}
        let mut b=base(if bytes.is_some(){4}else{3},self.request)?;
        put(&mut b,24,self.sequence);put(&mut b,32,offset as u64);
        if let Some(bytes)=bytes{b[40..40+n].copy_from_slice(bytes);}Ok(b)
    }
    pub fn check_get(self,raw:&[u8],offset:usize)->Result<(),Error>{
        if self.part(offset,None)?.as_slice()!=raw{return Err(Error::Identity);}Ok(())
    }
    /// A request or release repeats the snapshot hash; no implicit phase advance.
    pub fn control(self,phase:Phase,seal:Option<u64>,released:bool)->Result<[u8;BYTES],Error>{
        self.frame()?;
        if seal==Some(0)||released&&seal.is_none(){return Err(Error::Identity);}
        let mut b=base(match(seal,released){(None,false)=>6,(Some(_),false)=>7,
            (Some(_),true)=>8,_=>return Err(Error::Encoding)},self.request)?;
        b[9]=phase.byte();put(&mut b,24,seal.unwrap_or(0));put(&mut b,32,self.sequence);
        b[40..72].copy_from_slice(&self.record_hash);Ok(b)
    }
    pub fn check_control(self,raw:&[u8],phase:Phase,seal:Option<u64>,released:bool)->Result<(),Error>{
        if self.control(phase,seal,released)?.as_slice()!=raw{return Err(Error::Identity);}Ok(())
    }
}
/// Exactly six ordered chunks; no arbitrary allocation or cross-session assembly.
pub struct RecordReceiver{snapshot:Snapshot,offset:usize,bytes:[u8;512]}
impl RecordReceiver{
    pub fn new(snapshot:Snapshot)->Result<Self,Error>{
        snapshot.frame()?;Ok(Self{snapshot,offset:0,bytes:[0;512]})
    }
    pub fn offset(&self)->usize{self.offset}
    pub fn push(&mut self,raw:&[u8])->Result<(),Error>{
        if self.offset>=512{return Err(Error::Order);}
        let b=header(raw,4)?;let n=(512-self.offset).min(PART);
        if self.snapshot.part(self.offset,Some(&b[40..40+n]))?!=*b{return Err(Error::Identity);}
        self.bytes[self.offset..self.offset+n].copy_from_slice(&b[40..40+n]);self.offset+=n;Ok(())
    }
    pub fn finish(self)->Result<Record,Error>{
        if self.offset!=512{return Err(Error::Order);}
        if sha256(&self.bytes).map_err(|_|Error::Record)?!=self.snapshot.record_hash{return Err(Error::Identity);}
        let record=Record::decode(&self.bytes).map_err(|_|Error::Record)?;
        if !self.snapshot.matches_record(record){return Err(Error::Identity);}Ok(record)
    }
}
/// Inspection-only metadata: never an executable update Transfer or CompleteRead.
/// Kernel seal/length/view and authenticated request/incarnation must still agree.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Inspection{
    pub phase:Phase,pub request:u64,pub seal:u64,pub sequence:u64,
    pub slot:Slot,pub generation:u64,pub length:usize,pub digest:[u8;32],pub stored_hash:[u8;32],
}
impl Inspection{
    pub fn frame(self)->Result<[u8;BYTES],Error>{
        if self.seal==0||self.sequence==0||self.generation==0||self.digest==[0;32]||
            self.stored_hash==[0;32]||!(512..=MAX_INSPECTION).contains(&self.length)||self.length%512!=0{
            return Err(Error::Identity);
        }
        let mut b=base(5,self.request)?;b[9]=self.phase.byte();
        b[10]=match self.slot{Slot::A=>0,Slot::B=>1};
        put(&mut b,24,self.seal);put(&mut b,32,self.sequence);put(&mut b,40,self.generation);
        put(&mut b,48,self.length as u64);b[56..88].copy_from_slice(&self.digest);
        b[88..120].copy_from_slice(&self.stored_hash);Ok(b)
    }
    pub fn parse(raw:&[u8])->Result<Self,Error>{
        let b=header(raw,5)?;
        let value=Self{phase:Phase::parse(b[9])?,request:word(b,16),seal:word(b,24),
            sequence:word(b,32),generation:word(b,40),
            length:usize::try_from(word(b,48)).map_err(|_|Error::Encoding)?,
            slot:match b[10]{0=>Slot::A,1=>Slot::B,_=>return Err(Error::Encoding)},
            digest:b[56..88].try_into().unwrap(),stored_hash:b[88..120].try_into().unwrap()};
        if value.frame()?!=*b{return Err(Error::Encoding);}Ok(value)
    }
    /// Value binding only. Fresh I/O, authority, root and content decisions are external.
    pub fn binds(self,snapshot:Snapshot,record:Record,phase:Phase)->bool{
        if self.frame().is_err()||!snapshot.matches_record(record)||self.request!=snapshot.request||
            self.sequence!=snapshot.sequence||self.phase!=phase{return false;}
        match phase{
            Phase::Active|Phase::FreshActive=>{
                let layer=record.active();
                (self.slot,self.generation,self.digest)==(layer.slot(),layer.generation(),layer.digest())
            },
            Phase::Prior|Phase::FreshPrior=>record.previous().is_some_and(|layer|
                (self.slot,self.generation,self.digest)==(layer.slot(),layer.generation(),layer.digest())),
            Phase::Factory|Phase::FreshFactory=>self.slot==record.active().slot().other()&&self.generation==1,
        }
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn record()->Record{
        let mut b=[0u8;512];b[..8].copy_from_slice(b"RARSYS00");b[10..12].copy_from_slice(&512u16.to_le_bytes());
        b[14]=255;
        for at in [16usize,24,32,40]{b[at..at+8].copy_from_slice(&1u64.to_le_bytes());}
        b[64..96].fill(1);let hash=sha256(&b[..480]).unwrap();b[480..].copy_from_slice(&hash);
        Record::decode(&b).unwrap()
    }
    fn inspection()->Inspection{Inspection{phase:Phase::Active,request:(1<<40)+7,seal:u64::MAX,
        sequence:1,slot:Slot::A,generation:1,length:512,digest:[1;32],stored_hash:[2;32]}}
    #[test]fn canonical_snapshot_and_start_bind_full_width_identity(){
        let r=record();let s=Snapshot::from_record((1<<40)+7,r).unwrap();
        let b=s.frame().unwrap();assert_eq!(Snapshot::parse(&b),Ok(s));
        let start=start(s.request).unwrap();assert_eq!(parse_start(&start),Ok(s.request));
        for n in 0..BYTES{assert!(Snapshot::parse(&b[..n]).is_err());assert!(parse_start(&start[..n]).is_err());}
        for p in 0..BYTES{
            let mut changed=b;changed[p]^=1;assert_ne!(Snapshot::parse(&changed),Ok(s));
            let mut changed=start;changed[p]^=1;assert_ne!(parse_start(&changed),Ok(s.request));
        }
        assert!(super::start(0).is_err());assert!(Snapshot{sequence:0,..s}.frame().is_err());
        assert!(Snapshot{record_hash:[0;32],..s}.frame().is_err());
        assert!(Snapshot::parse(&[b.as_slice(),&[0]].concat()).is_err());
    }
    #[test]fn record_parts_are_ordered_exact_hashed_and_session_bound(){
        let r=record();let s=Snapshot::from_record(9,r).unwrap();let bytes=r.encode();
        let mut receiver=RecordReceiver::new(s).unwrap();
        assert!(RecordReceiver::new(s).unwrap().finish().is_err());
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);let b=s.part(offset,Some(&bytes[offset..offset+n])).unwrap();
            for p in [0usize,8,9,16,24,32]{
                let mut changed=b;changed[p]^=1;assert!(receiver.push(&changed).is_err());
                assert_eq!(receiver.offset(),offset);
            }
            if offset==440{let mut changed=b;changed[127]=1;assert!(receiver.push(&changed).is_err());}
            s.check_get(&s.part(offset,None).unwrap(),offset).unwrap();
            receiver.push(&b).unwrap();assert!(receiver.push(&b).is_err());
        }
        assert_eq!(receiver.finish(),Ok(r));
        let mut receiver=RecordReceiver::new(Snapshot{record_hash:[3;32],..s}).unwrap();
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);receiver.push(&s.part(offset,Some(&bytes[offset..offset+n])).unwrap()).unwrap();
        }
        assert_eq!(receiver.finish(),Err(Error::Identity));
        assert!(s.part(1,None).is_err());assert!(s.part(512,None).is_err());
        assert!(s.part(0,Some(&[0;87])).is_err());
    }
    #[test]fn inspection_has_distinct_framing_and_exact_sector_bounds(){
        let t=inspection();let b=t.frame().unwrap();assert_eq!(Inspection::parse(&b),Ok(t));
        for n in 0..BYTES{assert!(Inspection::parse(&b[..n]).is_err());}
        for p in 0..BYTES{let mut changed=b;changed[p]^=1;assert_ne!(Inspection::parse(&changed),Ok(t));}
        for length in [0,511,513,896,MAX_INSPECTION-1,MAX_INSPECTION+1,usize::MAX]{
            assert!(Inspection{length,..t}.frame().is_err());
        }
        for length in [512,1024,MAX_INSPECTION]{
            let t=Inspection{length,..t};assert_eq!(Inspection::parse(&t.frame().unwrap()),Ok(t));
        }
        for phase in [Phase::Active,Phase::Prior,Phase::Factory,Phase::FreshActive,Phase::FreshPrior,Phase::FreshFactory]{
            let t=Inspection{phase,..t};assert_eq!(Inspection::parse(&t.frame().unwrap()),Ok(t));
        }
        assert!(crate::update_wire::Transfer::parse(&b,crate::update_wire::Kind::Offer).is_err());
        assert!(Inspection::parse(&[b.as_slice(),&[0]].concat()).is_err());
    }
    #[test]fn binding_is_not_content_authority_and_release_is_exact(){
        let r=record();let t=inspection();let s=Snapshot::from_record(t.request,r).unwrap();
        assert!(t.binds(s,r,Phase::Active));assert!(!t.binds(s,r,Phase::Prior));
        assert!(!Inspection{request:t.request+1,..t}.binds(s,r,Phase::Active));
        assert!(!Inspection{sequence:2,..t}.binds(s,r,Phase::Active));
        assert!(!Inspection{digest:[3;32],..t}.binds(s,r,Phase::Active));
        let factory=Inspection{phase:Phase::Factory,slot:Slot::B,..t};
        assert!(factory.binds(s,r,Phase::Factory)); // signature/root still NOT proved
        assert!(!Inspection{generation:2,..factory}.binds(s,r,Phase::Factory));
        assert!(!Inspection{slot:Slot::A,..factory}.binds(s,r,Phase::Factory));
        for (seal,released)in [(None,false),(Some(t.seal),false),(Some(t.seal),true)]{
            let b=s.control(t.phase,seal,released).unwrap();
            s.check_control(&b,t.phase,seal,released).unwrap();
            for p in 0..BYTES{let mut changed=b;changed[p]^=1;
                assert!(s.check_control(&changed,t.phase,seal,released).is_err());}
        }
        assert!(s.control(t.phase,Some(0),false).is_err());
        assert!(s.control(t.phase,None,true).is_err());
        assert_eq!(Phase::Active.next(false),Some(Phase::Factory));
        assert_eq!(Phase::Active.next(true),Some(Phase::Prior));
        assert_eq!(Phase::FreshActive.next(false),Some(Phase::FreshFactory));
        assert_eq!(Phase::FreshActive.next(true),Some(Phase::FreshPrior));
        assert_eq!(Phase::FreshFactory.next(true),None);
    }
}
