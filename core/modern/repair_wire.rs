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

/// Serialized Manager-side inspection progress. This is a protocol guard, not
/// a kernel lease or proof of fresh I/O. The native adapter must authenticate
/// kernel envelopes, validate VIEW10 and hash the actual bytes before offer(),
/// and successfully scrub VIEW11 before accepting the Released frame.
/// All errors poison this transaction; recovery never retries it in place.
pub struct Progress {
    snapshot:Snapshot, record:Record, system_incarnation:u64,
    state:ProgressState, last_seal:u64, observations:[Option<Inspection>;3],
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum ProgressState { Ready(Phase), Awaiting(Phase), Offered(Inspection), Releasing(Inspection), Complete, Halted }
impl Progress {
    pub fn new(snapshot:Snapshot,record:Record,system_incarnation:u64)->Result<Self,Error>{
        if system_incarnation==0||!snapshot.matches_record(record){return Err(Error::Identity);}
        Ok(Self{snapshot,record,system_incarnation,state:ProgressState::Ready(Phase::Active),
            last_seal:0,observations:[None;3]})
    }
    pub fn halt(&mut self){self.state=ProgressState::Halted;}
    pub fn is_halted(&self)->bool{self.state==ProgressState::Halted}
    fn reject<T>(&mut self,error:Error)->Result<T,Error>{self.halt();Err(error)}
    pub fn request(&mut self)->Result<[u8;BYTES],Error>{
        let ProgressState::Ready(phase)=self.state else{return self.reject(Error::Order);};
        let frame=match self.snapshot.control(phase,None,false){
            Ok(frame)=>frame,Err(e)=>return self.reject(e),
        };
        self.state=ProgressState::Awaiting(phase);Ok(frame)
    }
    /// sender/incarnation MUST come from the kernel envelope, never the payload.
    /// The native caller separately validates the kernel view and actual bytes.
    pub fn offer(&mut self,sender:u64,incarnation:u64,raw:&[u8])->Result<Inspection,Error>{
        if sender!=9||incarnation!=self.system_incarnation{return self.reject(Error::Identity);}
        let ProgressState::Awaiting(phase)=self.state else{return self.reject(Error::Order);};
        let offer=match Inspection::parse(raw){Ok(t)=>t,Err(e)=>return self.reject(e)};
        if !offer.binds(self.snapshot,self.record,phase)||offer.seal<=self.last_seal{
            return self.reject(Error::Identity);
        }
        // Leave enough identity space for every remaining required phase.
        // Exhaustion is a terminal refusal, never wraparound or a partial
        // successful exchange that cannot possibly reach completion.
        let mut remaining=0u64;
        let mut next=phase.next(self.record.previous().is_some());
        while let Some(p)=next{
            remaining+=1;next=p.next(self.record.previous().is_some());
        }
        if offer.seal.checked_add(remaining).is_none(){return self.reject(Error::Identity);}
        let index=match phase{
            Phase::Active|Phase::FreshActive=>0,
            Phase::Prior|Phase::FreshPrior=>1,
            Phase::Factory|Phase::FreshFactory=>2,
        };
        if matches!(phase,Phase::FreshActive|Phase::FreshPrior|Phase::FreshFactory){
            let Some(old)=self.observations[index]else{return self.reject(Error::Order);};
            // New seals/phases are mandatory, but all storage identity fields
            // must still match. This comparison is NOT evidence of fresh I/O.
            if (offer.slot,offer.generation,offer.length,offer.digest,offer.stored_hash)!=
                (old.slot,old.generation,old.length,old.digest,old.stored_hash){
                return self.reject(Error::Identity);
            }
        }else{
            if self.observations[index].is_some(){return self.reject(Error::Order);}
            self.observations[index]=Some(offer);
        }
        self.state=ProgressState::Offered(offer);Ok(offer)
    }
    /// Build the release only after native classification and VIEW11 cleanup.
    /// This value does not attest that either operation actually occurred.
    pub fn release_request(&mut self)->Result<[u8;BYTES],Error>{
        let ProgressState::Offered(offer)=self.state else{return self.reject(Error::Order);};
        match self.snapshot.control(offer.phase,Some(offer.seal),false){
            Ok(frame)=>{self.state=ProgressState::Releasing(offer);Ok(frame)},Err(e)=>self.reject(e),
        }
    }
    pub fn released(&mut self,sender:u64,incarnation:u64,raw:&[u8])->Result<(),Error>{
        if sender!=9||incarnation!=self.system_incarnation{return self.reject(Error::Identity);}
        let ProgressState::Releasing(offer)=self.state else{return self.reject(Error::Order);};
        if let Err(e)=self.snapshot.check_control(raw,offer.phase,Some(offer.seal),true){
            return self.reject(e);
        }
        self.last_seal=offer.seal;
        self.state=match offer.phase.next(self.record.previous().is_some()){
            Some(phase)=>ProgressState::Ready(phase),None=>ProgressState::Complete,
        };
        Ok(())
    }
    /// Consumes the ordered protocol progress once. NOT a repair permit: native
    /// complete-read receipts, independently rooted Plan, and freshness remain
    /// required before any write. No reference or reusable token escapes.
    pub fn finish(mut self)->Result<(),Error>{
        if self.state!=ProgressState::Complete{return self.reject(Error::Order);}
        self.halt();Ok(())
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
    fn prior_record(active:Slot)->Record{
        let mut b=record().encode();b[12]=1;b[13]=if active==Slot::A{0}else{1};
        b[14]=if active==Slot::A{1}else{0};
        for at in [16usize,24,40]{b[at..at+8].copy_from_slice(&2u64.to_le_bytes());}
        for at in [48usize,56]{b[at..at+8].copy_from_slice(&1u64.to_le_bytes());}
        b[64..96].fill(2);b[96..128].fill(1);b[128..160].fill(7);
        let hash=sha256(&b[..480]).unwrap();b[480..].copy_from_slice(&hash);
        Record::decode(&b).unwrap()
    }
    const INC:u64=(1<<48)+3;
    fn session(r:Record)->(Snapshot,Progress){
        let s=Snapshot::from_record((1<<40)+7,r).unwrap();
        (s,Progress::new(s,r,INC).unwrap())
    }
    fn observed(r:Record,s:Snapshot,phase:Phase,seal:u64)->Inspection{
        let (slot,generation,digest)=match phase{
            Phase::Active|Phase::FreshActive=>{
                let l=r.active();(l.slot(),l.generation(),l.digest())},
            Phase::Prior|Phase::FreshPrior=>{
                let l=r.previous().unwrap();(l.slot(),l.generation(),l.digest())},
            Phase::Factory|Phase::FreshFactory=>(r.active().slot().other(),1,[7;32]),
        };
        Inspection{phase,request:s.request,seal,sequence:r.sequence(),slot,generation,digest,
            length:1024,stored_hash:[match phase{Phase::Active|Phase::FreshActive=>11,
                Phase::Prior|Phase::FreshPrior=>12,_=>13};32]}
    }
    fn step(p:&mut Progress,s:Snapshot,t:Inspection){
        assert_eq!(p.request(),s.control(t.phase,None,false));
        assert_eq!(p.offer(9,INC,&t.frame().unwrap()),Ok(t));
        assert_eq!(p.release_request(),s.control(t.phase,Some(t.seal),false));
        p.released(9,INC,&s.control(t.phase,Some(t.seal),true).unwrap()).unwrap();
    }
    #[test]fn progress_requires_every_initial_and_fresh_phase_in_both_slot_directions(){
        for r in [record(),prior_record(Slot::A),prior_record(Slot::B)]{
            let(s,mut p)=session(r);let mut phase=Some(Phase::Active);let mut seal=1;
            while let Some(current)=phase{
                step(&mut p,s,observed(r,s,current,seal));
                phase=current.next(r.previous().is_some());seal+=1;
            }
            assert_eq!(p.finish(),Ok(()));
            // All prefixes, including before the final release, are incomplete.
            for prefix in 0..seal-1{
                let(_,mut p)=session(r);let mut phase=Phase::Active;
                for n in 0..prefix{
                    step(&mut p,s,observed(r,s,phase,n+1));
                    phase=phase.next(r.previous().is_some()).unwrap();
                }
                assert_eq!(p.finish(),Err(Error::Order));
            }
        }
    }
    #[test]fn progress_rejects_duplicate_requests_offers_and_early_or_repeated_releases(){
        let r=record();let(s,_)=session(r);let t=observed(r,s,Phase::Active,1);
        let offer=t.frame().unwrap();let ack=s.control(t.phase,Some(t.seal),true).unwrap();
        for attack in 0..7{
            let(_,mut p)=session(r);
            match attack{
                0=>{assert!(p.offer(9,INC,&offer).is_err());},
                1=>{p.request().unwrap();assert!(p.request().is_err());},
                2=>{p.request().unwrap();p.offer(9,INC,&offer).unwrap();
                    assert!(p.offer(9,INC,&offer).is_err());},
                3=>{p.request().unwrap();assert!(p.release_request().is_err());},
                4=>{p.request().unwrap();p.offer(9,INC,&offer).unwrap();
                    assert!(p.released(9,INC,&ack).is_err());},
                5=>{p.request().unwrap();p.offer(9,INC,&offer).unwrap();p.release_request().unwrap();
                    assert!(p.release_request().is_err());},
                _=>{step(&mut p,s,t);assert!(p.released(9,INC,&ack).is_err());},
            }
            assert!(p.is_halted());assert!(p.request().is_err());
            assert!(p.offer(9,INC,&offer).is_err());assert!(p.finish().is_err());
        }
    }
    #[test]fn progress_authenticates_full_peer_identity_and_poison_is_sticky(){
        let r=record();let(s,_)=session(r);let t=observed(r,s,Phase::Active,1);
        for(sender,incarnation)in [(8,INC),(9,3),(9,0),(9,INC+1),(10,INC)]{
            for at_release in [false,true]{
                let(_,mut p)=session(r);p.request().unwrap();
                if at_release{
                    p.offer(9,INC,&t.frame().unwrap()).unwrap();p.release_request().unwrap();
                    assert!(p.released(sender,incarnation,&s.control(t.phase,Some(1),true).unwrap()).is_err());
                }else{assert!(p.offer(sender,incarnation,&t.frame().unwrap()).is_err());}
                assert!(p.is_halted());assert!(p.request().is_err());
            }
        }
        assert!(Progress::new(s,r,0).is_err());
        assert!(Progress::new(Snapshot{record_hash:[9;32],..s},r,INC).is_err());
        let(_,mut p)=session(r);p.halt();assert!(p.finish().is_err());
    }
    #[test]fn progress_rejects_stale_seals_wrong_phase_and_substituted_snapshot(){
        let r=record();let(s,_)=session(r);
        for defect in 0..6{
            let(_,mut p)=session(r);step(&mut p,s,observed(r,s,Phase::Active,10));
            p.request().unwrap();let mut t=observed(r,s,Phase::Factory,11);
            match defect{0=>t.seal=10,1=>t.seal=9,2=>t.phase=Phase::FreshFactory,
                3=>t.request+=1,4=>t.sequence+=1,_=>t.slot=r.active().slot()}
            assert!(p.offer(9,INC,&t.frame().unwrap()).is_err());assert!(p.is_halted());
        }
        // Refuse exhaustion before accepting a phase with no possible successor.
        for r in [record(),prior_record(Slot::A),prior_record(Slot::B)]{
            let(s,_)=session(r);let mut phases=vec![];let mut phase=Some(Phase::Active);
            while let Some(p)=phase{phases.push(p);phase=p.next(r.previous().is_some());}
            for target in 0..phases.len()-1{
                let(_,mut p)=session(r);
                for(i,&phase)in phases[..target].iter().enumerate(){
                    step(&mut p,s,observed(r,s,phase,i as u64+1));
                }
                p.request().unwrap();let remaining=(phases.len()-1-target)as u64;
                let impossible=u64::MAX-remaining+1;
                assert!(p.offer(9,INC,&observed(r,s,phases[target],impossible).frame().unwrap()).is_err());
                assert!(p.is_halted());
            }
            let(_,mut p)=session(r);
            for(i,&phase)in phases.iter().enumerate(){
                let seal=u64::MAX-(phases.len()-1-i)as u64;
                step(&mut p,s,observed(r,s,phase,seal));
            }
            assert_eq!(p.finish(),Ok(())); // MAX is legal only at the final phase.
        }
    }
    #[test]fn progress_compares_all_fresh_storage_identity_fields_and_requires_release(){
        for target in [Phase::FreshActive,Phase::FreshPrior,Phase::FreshFactory]{
            for field in 0..5{
                let r=prior_record(Slot::B);let(s,mut p)=session(r);
                let mut phase=Phase::Active;let mut seal=1;
                while phase!=target{
                    step(&mut p,s,observed(r,s,phase,seal));
                    seal+=1;phase=phase.next(true).unwrap();
                }
                p.request().unwrap();let mut t=observed(r,s,phase,seal);
                match field{0=>t.slot=t.slot.other(),1=>t.generation+=1,2=>t.length+=512,
                    3=>t.digest[0]^=1,_=>t.stored_hash[0]^=1}
                assert!(p.offer(9,INC,&t.frame().unwrap()).is_err());assert!(p.is_halted());
            }
        }
        let r=record();let(s,mut p)=session(r);let mut phase=Phase::Active;let mut seal=1;
        while phase!=Phase::FreshFactory{
            step(&mut p,s,observed(r,s,phase,seal));seal+=1;phase=phase.next(false).unwrap();
        }
        p.request().unwrap();p.offer(9,INC,&observed(r,s,phase,seal).frame().unwrap()).unwrap();
        p.release_request().unwrap();assert_eq!(p.finish(),Err(Error::Order));
    }
    #[test]fn progress_rejects_every_release_byte_mutation_and_offer_truncation(){
        let r=record();let(s,_)=session(r);let t=observed(r,s,Phase::Active,1);
        let offer=t.frame().unwrap();let ack=s.control(t.phase,Some(1),true).unwrap();
        for n in 0..BYTES{
            let(_,mut p)=session(r);p.request().unwrap();
            assert!(p.offer(9,INC,&offer[..n]).is_err());assert!(p.is_halted());
            let(_,mut p)=session(r);p.request().unwrap();p.offer(9,INC,&offer).unwrap();
            p.release_request().unwrap();let mut bad=ack;bad[n]^=1;
            assert!(p.released(9,INC,&bad).is_err());assert!(p.is_halted());
        }
    }

}
