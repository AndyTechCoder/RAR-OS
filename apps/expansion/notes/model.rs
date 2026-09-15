//! Independent Notes app state. Pure tests do not execute target entry.
#![forbid(unsafe_code)]
use crate::sdk::{wire::{Boot,Message,READ_DOCUMENT,WRITE_DOCUMENT},protocol,transport::Received};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Status{Ready,Loading,Saving,Saved,Edited,Full,Busy,Unavailable,Uncertain,Retained}
#[derive(Clone,Copy)]
struct Pending{operation:u8,sequence:u32,deadline:u64,revision:u64}
pub struct Notes{
    boot:Boot,text:[u8;64],len:usize,revision:u64,sequence:u32,input_sequence:u32,
    pending:Option<Pending>,pub status:Status,
}
impl Notes{
    pub fn new(boot:Boot)->Result<Self,()>{
        boot.encode().map_err(|_|())?;
        if boot.principal!=10||boot.rights!=3{return Err(());}
        Ok(Self{boot,text:[0;64],len:0,revision:0,sequence:0,input_sequence:0,pending:None,status:Status::Ready})
    }
    pub fn request(&mut self,write:bool,now:u64)->Result<Message,()>{
        if self.pending.is_some(){self.status=Status::Busy;return Err(());}
        let sequence=self.sequence.checked_add(1).ok_or(())?;
        let deadline=now.checked_add(100).ok_or(())?;
        let operation=if write{WRITE_DOCUMENT}else{READ_DOCUMENT};
        let payload=if write{&self.text[..self.len]}else{&[]};
        let message=Message::new(operation,0,sequence,payload).map_err(|_|())?;
        self.sequence=sequence;self.pending=Some(Pending{operation,sequence,deadline,revision:self.revision});
        self.status=if write{Status::Saving}else{Status::Loading};Ok(message)
    }
    /// Called only when the one request SEND failed before acceptance. Never
    /// retry accepted or uncertain writes. A future user command is explicit.
    pub fn send_failed(&mut self){self.pending=None;self.status=Status::Unavailable;}
    pub fn tick(&mut self,now:u64)->bool{
        if self.pending.is_some_and(|p|now>=p.deadline){
            self.pending=None;self.status=Status::Uncertain;return true;
        }false
    }
    pub fn receive(&mut self,received:Received,now:u64)->Result<Option<Message>,()>{
        self.tick(now);
        if received.from_peer(&self.boot,0){
            let key=protocol::key_of(&received.message).map_err(|_|())?;
            if received.message.sequence()<=self.input_sequence{return Err(());}
            self.input_sequence=received.message.sequence();
            return match key{
                13=>self.request(true,now).map(Some),
                27=>self.request(false,now).map(Some),
                8=>{
                    if self.len>0{
                        let revision=self.revision.checked_add(1).ok_or(())?;
                        self.len-=1;self.text[self.len]=0;self.revision=revision;self.status=Status::Edited;
                    }Ok(None)
                },
                32..=126=>{
                    if self.len==64{self.status=Status::Full;return Ok(None);}
                    let revision=self.revision.checked_add(1).ok_or(())?;
                    self.text[self.len]=key;self.len+=1;self.revision=revision;self.status=Status::Edited;Ok(None)
                },
                _=>Err(()),
            };
        }
        if !received.from_peer(&self.boot,1){return Err(());}
        let p=self.pending.ok_or(())?;
        let m=received.message;
        if m.operation()!=p.operation||m.sequence()!=p.sequence{return Err(());}
        protocol::document_reply(&m).map_err(|_|())?;
        if m.status()!=protocol::OK{
            self.pending=None;
            self.status=if m.status()==protocol::INDETERMINATE{Status::Uncertain}else{Status::Unavailable};
            return Ok(None);
        }
        if p.operation==READ_DOCUMENT{
            if !m.payload().iter().all(|b|(32..=126).contains(b)){
                self.pending=None;self.status=Status::Unavailable;return Ok(None);
            }
            if self.revision==p.revision{
                self.text=[0;64];self.len=m.payload().len();self.text[..self.len].copy_from_slice(m.payload());
                self.status=Status::Ready;
            }else{self.status=Status::Retained;}
        }else{self.status=if self.revision==p.revision{Status::Saved}else{Status::Edited};}
        self.pending=None;Ok(None)
    }
    pub fn line(&self,row:usize)->&[u8]{
        match row{
            0=>b"RAR NOTES / INDEPENDENT RUST APP",
            1=>match self.status{
                Status::Ready=>b"READY",Status::Loading=>b"LOADING",Status::Saving=>b"SAVING",
                Status::Saved=>b"SAVED",Status::Edited=>b"UNSAVED EDITS",Status::Full=>b"DOCUMENT FULL (64 BYTES)",
                Status::Busy=>b"OPERATION ALREADY PENDING",Status::Unavailable=>b"STORAGE UNAVAILABLE",
                Status::Uncertain=>b"SAVE/READ UNCERTAIN - NO AUTOMATIC RETRY",
                Status::Retained=>b"LOAD DID NOT REPLACE YOUR NEW EDITS",
            },
            2=>&self.text[..self.len.min(48)],
            3=>if self.len>48{&self.text[48..self.len]}else{b""},
            4=>b"ENTER: SAVE   ESC: LOAD   BACKSPACE: EDIT",
            _=>b"PRIVATE DOCUMENT / NO DISK OR NETWORK ACCESS",
        }
    }
}
#[cfg(test)]mod tests{
    use super::*;
    fn boot()->Boot{
        Boot{application:[1;16],incarnation:1<<40,principal:10,rights:3,entry:0x401000,
            caps:[0x1_0000_0001,0x1_0000_0002,0x1_0000_0003,0,0],
            peer_principals:[3,1,0,0],peer_incarnations:[0x1_0000_0003,0x1_0000_0001,0,0]}
    }
    fn key(n:&mut Notes,sequence:u32,k:u8)->Result<Option<Message>,()>{
        n.receive(Received{principal:3,incarnation:boot().peer_incarnations[0],
            message:protocol::input(sequence,k).unwrap()},1)
    }
    fn reply(n:&mut Notes,request:&Message,data:&[u8],now:u64)->Result<Option<Message>,()>{
        n.receive(Received{principal:1,incarnation:boot().peer_incarnations[1],
            message:Message::new(request.operation(),0,request.sequence(),data).unwrap()},now)
    }
    #[test]fn edits_save_and_load_are_correlated_and_do_not_lose_new_input(){
        let mut n=Notes::new(boot()).unwrap();let load=n.request(false,0).unwrap();
        key(&mut n,1,b'x').unwrap();reply(&mut n,&load,b"old disk",2).unwrap();
        assert_eq!(n.line(2),b"x");assert_eq!(n.status,Status::Retained);
        let write=key(&mut n,2,13).unwrap().unwrap();assert_eq!(write.payload(),b"x");
        key(&mut n,3,b'y').unwrap();reply(&mut n,&write,b"",3).unwrap();
        assert_eq!(n.line(2),b"xy");assert_eq!(n.status,Status::Edited);
        let write=n.request(true,4).unwrap();reply(&mut n,&write,b"",5).unwrap();
        assert_eq!(n.status,Status::Saved);
        let read=n.request(false,6).unwrap();reply(&mut n,&read,b"disk",7).unwrap();
        assert_eq!(n.line(2),b"disk");
    }
    #[test]fn forged_stale_duplicate_and_late_inputs_cannot_change_document(){
        let mut n=Notes::new(boot()).unwrap();let read=n.request(false,0).unwrap();
        for (principal,incarnation)in [(3,3),(1,1),(8,boot().peer_incarnations[1]),(1,0)]{
            assert!(n.receive(Received{principal,incarnation,
                message:Message::new(READ_DOCUMENT,0,read.sequence(),b"bad").unwrap()},1).is_err());
        }
        assert_eq!(n.line(2),b"");assert!(n.tick(100));assert_eq!(n.status,Status::Uncertain);
        assert!(reply(&mut n,&read,b"late",101).is_err());assert_eq!(n.line(2),b"");
        key(&mut n,1,b'a').unwrap();assert!(key(&mut n,1,b'b').is_err());assert_eq!(n.line(2),b"a");
    }
    #[test]fn bounded_text_read_refusal_and_no_implicit_write_retry(){
        let mut n=Notes::new(boot()).unwrap();
        for i in 1..=65{key(&mut n,i,b'a').unwrap();}
        assert_eq!(n.len,64);assert_eq!(n.status,Status::Full);
        let write=n.request(true,2).unwrap();assert!(n.request(true,3).is_err());
        assert!(n.tick(102));assert!(reply(&mut n,&write,b"",103).is_err());
        assert!(n.pending.is_none());
        let read=n.request(false,104).unwrap();reply(&mut n,&read,b"\0",105).unwrap();
        assert_eq!(n.len,64);assert_eq!(n.status,Status::Unavailable);
        n.sequence=u32::MAX;assert!(n.request(true,106).is_err());
    }
}
