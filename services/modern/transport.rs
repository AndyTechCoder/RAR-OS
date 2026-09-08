//! Private Modern file IPC: bounded correlation within one live service epoch.
//! Not durable transaction IDs. Restart requires kernel revocation/reincarnation.
#![forbid(unsafe_code)]
use crate::{desktop_wire as wire, store::{self,Failure,Store}, vault::Block};
pub const MAGIC:[u8;8]=*b"RARFIO01";
pub const DEADLINE_TICKS:u64=4096;
pub const MESSAGE_BUDGET:u32=65_536;
const READ_ONLY:u8=5;
const UNAVAILABLE:u8=6;
const UNCERTAIN:u8=7;
fn mutation(op:u8)->bool { matches!(op,wire::CREATE|wire::WRITE) }
fn unpack(frame:&[u8])->Option<(u64,[u8;128])> {
    if frame.len()!=128 || frame[120..]!=MAGIC ||
        frame[80..112].iter().any(|b|*b!=0) {return None;}
    let id=u64::from_le_bytes(frame[112..120].try_into().ok()?);
    if id==0 {return None;}
    let mut body=[0;128];body[..80].copy_from_slice(&frame[..80]);Some((id,body))
}
fn pack(body:[u8;128],id:u64)->Option<[u8;128]> {
    if id==0 || body[80..].iter().any(|b|*b!=0) {return None;}
    let mut frame=body;frame[112..120].copy_from_slice(&id.to_le_bytes());
    frame[120..].copy_from_slice(&MAGIC);Some(frame)
}
fn reply_valid(op:u8,body:&[u8;128])->bool {
    let status=body[0];
    if status!=wire::OK {
        let allowed=match status {
            wire::INVALID|UNAVAILABLE=>true,
            wire::NOT_FOUND=>matches!(op,wire::READ|wire::WRITE),
            wire::EXISTS=>op==wire::CREATE,
            wire::QUOTA|READ_ONLY|UNCERTAIN=>mutation(op),
            _=>false,
        };
        return allowed && body[1..].iter().all(|b|*b==0);
    }
    match op {
        wire::CREATE|wire::WRITE=>body.iter().all(|b|*b==0),
        wire::READ=>{
            let size=body[2] as usize;
            body[1]==0 && size<=64 && body[3..16].iter().all(|b|*b==0) &&
                body[16+size..].iter().all(|b|*b==0)
        }
        wire::LIST=>{
            if body[1]>4 || body[2..4]!=[0,0] {return false;}
            let mut at=4;let mut previous:Option<&[u8]>=None;
            for _ in 0..body[1] {
                let size=body[at] as usize;at+=1;
                if size==0 || size>12 || at+size>80 {return false;}
                let name=&body[at..at+size];at+=size;
                if !store::name_ok(name) || previous.is_some_and(|old|old>=name) {return false;}
                previous=Some(name);
            }
            body[at..].iter().all(|b|*b==0)
        }
        _=>false,
    }
}
/// One owner/receiver loop. Never recreate with the same live client grants.
/// After service loss, revoke old endpoints/queues and reincarnate clients
/// before mounting a new Server. The kernel must enforce this before activation.
pub struct Server<B:Block> {store:Store<B>,last:[u64;2]}
impl<B:Block> Server<B> {
    pub fn new(store:Store<B>)->Self {Self{store,last:[0;2]}}
    pub fn into_store(self)->Store<B> {self.store}
    pub fn handle(&mut self,sender:u64,generation:u64,frame:&[u8])->Option<[u8;128]> {
        if !self.store.authorized(sender,generation) {return None;}
        let role=if sender==4 {0}else{1};
        let (id,body)=unpack(frame)?;
        // Exact successors reject stale, duplicate and large-gap traffic.
        // A lost send consumes the client ID; a resulting gap fails closed and
        // requires renewed sessions, never retry of the uncertain operation.
        if self.last[role].checked_add(1)!=Some(id) {return None;}
        self.last[role]=id; // Before any Store/vault work, including INVALID bodies.
        let reply=match self.store.process(sender,generation,&body) {
            Ok(reply)=>reply,
            Err(error)=>{
                let mut out=[0;128];out[0]=match error {Failure::ReadOnly=>READ_ONLY,
                    Failure::Unavailable=>UNAVAILABLE,Failure::Indeterminate=>UNCERTAIN};out
            }
        };
        pack(reply,id)
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum StartError {Invalid,Busy,WritesLocked,Exhausted}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Outcome {Reply([u8;128]),ReadOnly,Unavailable,SaveUncertain}
#[derive(Clone,Copy)]
struct Pending {id:u64,op:u8,deadline:u64,budget:u32}
pub struct Client {generation:u64,next:u64,pending:Option<Pending>,writes_locked:bool}
impl Client {
    /// Expected storage incarnation is trusted bootstrap/grant material.
    pub fn new(generation:u64)->Result<Self,StartError> {
        if generation==0 {return Err(StartError::Invalid);}
        Ok(Self{generation,next:1,pending:None,writes_locked:false})
    }
    pub fn writes_locked(&self)->bool {self.writes_locked}
    /// Allocate once before the sole send attempt. Even failed/ambiguous sends
    /// burn this ID; there is no automatic retry or rollback method.
    pub fn begin(&mut self,now:u64,body:&[u8])->Result<[u8;128],StartError> {
        if self.pending.is_some() {return Err(StartError::Busy);}
        let (op,_,_)=store::decode(body).ok_or(StartError::Invalid)?;
        if self.writes_locked && mutation(op) {return Err(StartError::WritesLocked);}
        if self.next==0 {return Err(StartError::Exhausted);}
        let deadline=now.checked_add(DEADLINE_TICKS).ok_or(StartError::Exhausted)?;
        let id=self.next;self.next=id.checked_add(1).unwrap_or(0);
        let mut value=[0;128];value.copy_from_slice(body);
        let frame=pack(value,id).ok_or(StartError::Invalid)?;
        self.pending=Some(Pending{id,op,deadline,budget:MESSAGE_BUDGET});Ok(frame)
    }
    /// Called at send failure or the fixed runtime deadline. READ/LIST never
    /// resolve mutation uncertainty, and no subsequent reply unlocks writes.
    pub fn expire(&mut self)->Option<Outcome> {
        let pending=self.pending.take()?;
        Some(if mutation(pending.op) {self.writes_locked=true;Outcome::SaveUncertain}
             else {Outcome::Unavailable})
    }
    pub fn tick(&mut self,now:u64)->Option<Outcome> {
        if self.pending.is_some_and(|p|now>=p.deadline) {self.expire()} else {None}
    }
    /// Runtime supplies monotonic kernel ticks; stale/malformed messages consume
    /// the fixed work budget and never extend the absolute deadline.
    pub fn receive(&mut self,now:u64,sender:u64,generation:u64,frame:&[u8])->Option<Outcome> {
        let pending=self.pending?;
        if now>=pending.deadline || pending.budget==0 {return self.expire();}
        self.pending.as_mut()?.budget-=1;
        if sender!=1 || generation!=self.generation {return self.exhausted();}
        let Some((id,body))=unpack(frame) else{return self.exhausted();};
        if id!=pending.id || !reply_valid(pending.op,&body) {return self.exhausted();}
        self.pending=None;
        let outcome=match body[0] {
            READ_ONLY=>{self.writes_locked=true;Outcome::ReadOnly},
            UNAVAILABLE=>{self.writes_locked=true;Outcome::Unavailable},
            UNCERTAIN=>{self.writes_locked=true;Outcome::SaveUncertain},
            _=>Outcome::Reply(body),
        };
        Some(outcome)
    }
    fn exhausted(&mut self)->Option<Outcome> {
        if self.pending.is_some_and(|p|p.budget==0) {self.expire()} else {None}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sha256::sha256,vault::{Error,Sector}};
    use std::{cell::Cell,rc::Rc};
    struct Disk {bytes:Vec<Sector>,calls:Rc<Cell<usize>>,fail:Option<usize>}
    impl Disk {
        fn new()->Self {
            let mut h=[0;512];h[..8].copy_from_slice(b"RARVLT00");
            h[8..12].copy_from_slice(&[0,0,0,2]);h[12..16].copy_from_slice(&1u32.to_le_bytes());
            h[16..20].copy_from_slice(&512u32.to_le_bytes());h[20..24].copy_from_slice(&4u32.to_le_bytes());
            h[24..32].copy_from_slice(&2u64.to_le_bytes());h[32..64].fill(7);h[64..96].fill(9);
            let hash=sha256(&h[..480]).unwrap();h[480..].copy_from_slice(&hash);
            let mut bytes=vec![[0;512];14];bytes[0]=h;bytes[1]=h;
            Self{bytes,calls:Rc::new(Cell::new(0)),fail:None}
        }
        fn hit(&mut self)->Result<(),Error> {
            self.calls.set(self.calls.get()+1);
            if self.fail==Some(self.calls.get()){Err(Error::Io)}else{Ok(())}
        }
    }
    impl Block for Disk {
        fn read(&mut self,lba:u32)->Result<Sector,Error>{
            self.hit()?;self.bytes.get(lba as usize).copied().ok_or(Error::Bounds)
        }
        fn write(&mut self,lba:u32,value:&Sector)->Result<(),Error>{
            self.hit()?;*self.bytes.get_mut(lba as usize).ok_or(Error::Bounds)?=*value;Ok(())
        }
        fn flush(&mut self)->Result<(),Error>{self.hit()}
    }
    fn body(op:u8)->[u8;128]{
        wire::request(op,if op==wire::LIST{b""}else{b"note"},b"").unwrap()
    }
    fn response(status:u8,id:u64)->[u8;128]{
        let mut body=[0;128];body[0]=status;pack(body,id).unwrap()
    }
    #[test] fn full_kernel_incarnations_survive_server_and_client_path() {
        let files=(1u64<<32)|3;let terminal=u64::MAX;let storage=(1u64<<40)|9;
        let mut server=Server::new(Store::mount(Disk::new(),14,files,terminal).unwrap());
        let mut client=Client::new(storage).unwrap();
        let frame=client.begin(0,&body(wire::LIST)).unwrap();
        assert!(server.handle(4,3,&frame).is_none());
        assert!(server.handle(6,u32::MAX as u64,&frame).is_none());
        let reply=server.handle(4,files,&frame).unwrap();
        assert_eq!(client.receive(1,1,9,&reply),None);
        assert_eq!(client.receive(2,1,storage,&reply),Some(Outcome::Reply([0;128])));
        let mut other=Client::new(u64::MAX).unwrap();
        let frame=other.begin(0,&body(wire::LIST)).unwrap();
        let reply=server.handle(6,terminal,&frame).unwrap();
        assert_eq!(other.receive(1,1,u32::MAX as u64,&reply),None);
        assert_eq!(other.receive(2,1,u64::MAX,&reply),Some(Outcome::Reply([0;128])));
    }
    #[test] fn lost_durable_reply_never_reexecutes_or_unlocks_client() {
        let disk=Disk::new();let calls=disk.calls.clone();
        let mut server=Server::new(Store::mount(disk,14,3,7).unwrap());
        let mut client=Client::new(9).unwrap();
        let frame=client.begin(0,&body(wire::CREATE)).unwrap();
        let ack=server.handle(6,7,&frame).unwrap();
        assert_eq!(ack[0],wire::OK);assert_eq!(server.store.revision(),1);
        let before=calls.get();
        assert_eq!(client.expire(),Some(Outcome::SaveUncertain));
        assert!(server.handle(6,7,&frame).is_none());assert_eq!(calls.get(),before);
        assert_eq!(client.receive(1,1,9,&ack),None);
        assert_eq!(client.begin(2,&body(wire::CREATE)),Err(StartError::WritesLocked));
        let read=client.begin(2,&body(wire::READ)).unwrap();
        let answer=server.handle(6,7,&read).unwrap();
        assert!(matches!(client.receive(3,1,9,&answer),Some(Outcome::Reply(_))));
        assert!(client.writes_locked());
        assert_eq!(client.begin(4,&body(wire::WRITE)),Err(StartError::WritesLocked));
    }
    #[test] fn server_envelope_framing_gaps_and_role_sequences() {
        let disk=Disk::new();let calls=disk.calls.clone();
        let mut server=Server::new(Store::mount(disk,14,3,7).unwrap());
        let request=pack(body(wire::CREATE),1).unwrap();let before=calls.get();
        for (sender,generation) in [(0,3),(1,7),(4,7),(6,3),(4,0),(6,1)] {
            assert!(server.handle(sender,generation,&request).is_none());
        }
        for len in 0..128 {assert!(server.handle(4,3,&request[..len]).is_none());}
        for index in [80,111,120,127] {
            let mut bad=request;bad[index]^=1;
            assert!(server.handle(4,3,&bad).is_none());
        }
        for id in [2,u64::MAX] {
            assert!(server.handle(4,3,&pack(body(wire::CREATE),id).unwrap()).is_none());
        }
        let mut zero=request;zero[112..120].fill(0);
        assert!(server.handle(4,3,&zero).is_none());assert_eq!(calls.get(),before);
        assert_eq!(server.handle(4,3,&request).unwrap()[0],wire::OK);
        let other=pack(wire::request(wire::CREATE,b"other",b"").unwrap(),1).unwrap();
        assert_eq!(server.handle(6,7,&other).unwrap()[0],wire::OK);
        assert_eq!(server.store.revision(),2);
        let before=calls.get();
        assert!(server.handle(4,3,&request).is_none());
        assert!(server.handle(6,7,&other).is_none());assert_eq!(calls.get(),before);
        // A valid outer frame with a malformed body burns its ID without I/O.
        let mut malformed=body(wire::CREATE);malformed[3]=1;
        let bad=pack(malformed,2).unwrap();
        assert_eq!(server.handle(4,3,&bad).unwrap()[0],wire::INVALID);
        assert!(server.handle(4,3,&bad).is_none());assert_eq!(calls.get(),before);
    }
    #[test] fn pending_identity_full_canonical_reply_and_busy() {
        let mut client=Client::new(9).unwrap();
        let request=client.begin(10,&body(wire::READ)).unwrap();
        assert_eq!(client.begin(10,&body(wire::READ)),Err(StartError::Busy));
        let ack=response(wire::OK,1);
        for (sender,generation) in [(0,9),(1,1),(6,9)] {
            assert_eq!(client.receive(11,sender,generation,&ack),None);
        }
        for index in [0,1,2,3,15,79,80,111,120,127] {
            let mut bad=ack;bad[index]=255;
            assert_eq!(client.receive(11,1,9,&bad),None);
        }
        for len in 0..128 {assert_eq!(client.receive(11,1,9,&ack[..len]),None);}
        assert_eq!(client.receive(11,1,9,&response(0,2)),None);
        assert_eq!(client.receive(11,1,9,&ack),Some(Outcome::Reply([0;128])));
        assert_eq!(u64::from_le_bytes(request[112..120].try_into().unwrap()),1);
        let next=client.begin(12,&body(wire::READ)).unwrap();
        assert_eq!(u64::from_le_bytes(next[112..120].try_into().unwrap()),2);
        assert_eq!(client.receive(13,1,9,&ack),None);
    }
    #[test] fn deadline_and_stale_flood_do_not_extend_wait() {
        let mut client=Client::new(1).unwrap();
        client.begin(100,&body(wire::WRITE)).unwrap();
        let deadline=100+DEADLINE_TICKS;
        for _ in 0..10 {assert_eq!(client.receive(deadline-1,99,1,&[0;128]),None);}
        assert_eq!(client.tick(deadline),Some(Outcome::SaveUncertain));
        assert!(client.writes_locked());
        let mut client=Client::new(1).unwrap();
        client.begin(0,&body(wire::CREATE)).unwrap();
        for _ in 1..MESSAGE_BUDGET {assert_eq!(client.receive(1,99,1,&[0;128]),None);}
        assert_eq!(client.receive(1,99,1,&[0;128]),Some(Outcome::SaveUncertain));
        assert!(client.writes_locked());
    }
    #[test] fn distinct_storage_failures_are_canonical_and_sticky() {
        for (status,outcome) in [(READ_ONLY,Outcome::ReadOnly),(UNAVAILABLE,Outcome::Unavailable),
                                  (UNCERTAIN,Outcome::SaveUncertain)] {
            let mut client=Client::new(3).unwrap();
            client.begin(0,&body(wire::CREATE)).unwrap();
            let mut bad=response(status,1);bad[10]=1;
            assert_eq!(client.receive(1,1,3,&bad),None);
            assert_eq!(client.receive(1,1,3,&response(status,1)),Some(outcome));
            assert!(client.writes_locked());
            assert_eq!(client.begin(2,&body(wire::CREATE)),Err(StartError::WritesLocked));
            assert!(client.begin(2,&body(wire::LIST)).is_ok());
        }
        let mut disk=Disk::new();disk.fail=Some(24); // commit write after mount14 + operation10
        let mut server=Server::new(Store::mount(disk,14,1,1).unwrap());
        let mut client=Client::new(1).unwrap();
        let request=client.begin(0,&body(wire::CREATE)).unwrap();
        let answer=server.handle(6,1,&request).unwrap();
        assert_eq!(client.receive(1,1,1,&answer),Some(Outcome::SaveUncertain));
    }
    #[test] fn list_order_duplicates_shapes_and_operation_statuses() {
        let mut list=[0;128];list[1]=2;list[4..8].copy_from_slice(&[1,b'a',1,b'b']);
        assert!(reply_valid(wire::LIST,&list));
        for change in [(1,5),(2,1),(4,0),(4,13),(7,b'a'),(7,b'0'),(8,1)] {
            let mut bad=list;bad[change.0]=change.1;assert!(!reply_valid(wire::LIST,&bad));
        }
        for op in [wire::CREATE,wire::WRITE,wire::READ,wire::LIST] {
            for status in 0..=255 {
                let value=unpack(&response(status,1)).unwrap().1;
                let allowed=match status {
                    wire::OK|wire::INVALID|UNAVAILABLE=>true,
                    wire::NOT_FOUND=>matches!(op,wire::READ|wire::WRITE),
                    wire::EXISTS=>op==wire::CREATE,
                    wire::QUOTA|READ_ONLY|UNCERTAIN=>mutation(op),
                    _=>false,
                };
                assert_eq!(reply_valid(op,&value),allowed);
            }
        }
    }
    #[test] fn ids_never_wrap_or_retry_and_clock_overflow_fails() {
        assert!(matches!(Client::new(0),Err(StartError::Invalid)));
        let mut client=Client::new(1).unwrap();client.next=u64::MAX;
        let frame=client.begin(0,&body(wire::READ)).unwrap();
        assert_eq!(u64::from_le_bytes(frame[112..120].try_into().unwrap()),u64::MAX);
        assert_eq!(client.receive(1,1,1,&response(0,u64::MAX)),Some(Outcome::Reply([0;128])));
        assert_eq!(client.begin(2,&body(wire::READ)),Err(StartError::Exhausted));
        let mut client=Client::new(1).unwrap();
        assert_eq!(client.begin(u64::MAX,&body(wire::READ)),Err(StartError::Exhausted));
        assert_eq!(client.begin(0,&[]),Err(StartError::Invalid));
        client.begin(0,&body(wire::READ)).unwrap();
        assert_eq!(client.expire(),Some(Outcome::Unavailable));
        assert_eq!(u64::from_le_bytes(client.begin(1,&body(wire::READ)).unwrap()[112..120]
                   .try_into().unwrap()),2);
    }
}
