//! Bounded native network-tool client. No automatic retry or ambient authority.
#![forbid(unsafe_code)]
use crate::sdk::{self,Operation,Status};
pub const POLLS:usize=4096;
pub const DEADLINE:u64=2000;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Unavailable,Invalid,Timeout,InputLost}
pub struct Envelope{pub peer:u64,pub incarnation:u64,pub length:usize,pub bytes:[u8;128]}
pub trait Transport{
    fn ticks(&mut self)->Result<u64,()>;
    /// One send attempt. Error never triggers a client retry.
    fn send(&mut self,bytes:&[u8])->Result<(),()>;
    fn poll(&mut self)->Result<Option<Envelope>,()>;
    fn input(&mut self,bytes:[u8;128])->bool;
    fn yield_cpu(&mut self)->Result<(),()>;
}
#[derive(Debug,PartialEq,Eq)]
pub struct Reply{pub status:Status,pub length:usize,pub bytes:[u8;112]}
pub struct Client{wire:sdk::Client,shell:u64}
impl Client{
    pub fn new(network_incarnation:u64,shell_incarnation:u64)->Result<Self,Error>{
        if shell_incarnation==0{return Err(Error::Invalid);}
        Ok(Self{wire:sdk::Client::new(7,network_incarnation).map_err(|_|Error::Invalid)?,shell:shell_incarnation})
    }
    pub fn call<T:Transport>(&mut self,t:&mut T,op:Operation,payload:&[u8])->Result<Reply,Error>{
        let result=self.perform(t,op,payload);
        if result.is_err(){self.wire.retire();} // effects may already have happened
        result
    }
    fn perform<T:Transport>(&mut self,t:&mut T,op:Operation,payload:&[u8])->Result<Reply,Error>{
        let mut bytes=[0;128];
        let n=self.wire.begin(op,payload,&mut bytes).map_err(|_|Error::Unavailable)?;
        let mut last=t.ticks().map_err(|_|Error::Unavailable)?;
        let until=last.checked_add(DEADLINE).ok_or(Error::Unavailable)?;
        t.send(&bytes[..n]).map_err(|_|Error::Unavailable)?;
        for _ in 0..POLLS{
            let now=t.ticks().map_err(|_|Error::Unavailable)?;
            if now<last||now>=until{return Err(Error::Timeout);}last=now;
            if let Some(e)=t.poll().map_err(|_|Error::Unavailable)?{
                if !(1..=128).contains(&e.length){return Err(Error::Invalid);}
                if e.peer==0&&e.incarnation==self.shell&&e.length==128{
                    if !t.input(e.bytes){return Err(Error::InputLost);}
                }else if e.peer==7{
                    let r=match self.wire.accept(e.peer,e.incarnation,&e.bytes[..e.length]){
                        Ok(r)=>r,
                        Err(sdk::Error::Peer)=>{t.yield_cpu().map_err(|_|Error::Unavailable)?;continue;},
                        Err(_)=>return Err(Error::Invalid),
                    };
                    let mut reply=Reply{status:r.status,length:r.payload.len(),bytes:[0;112]};
                    reply.bytes[..r.payload.len()].copy_from_slice(r.payload);return Ok(reply);
                }
            }
            t.yield_cpu().map_err(|_|Error::Unavailable)?;
        }
        Err(Error::Timeout)
    }
}
#[derive(Debug,PartialEq,Eq)]
pub enum Command<'a>{Send(&'a[u8]),Receive,Close,Help,Invalid}
pub fn command(line:&[u8])->Option<Command<'_>>{
    if line.len()>64||!line.iter().all(|b|(32..=126).contains(b)){return None;}
    let line=line.trim_ascii();
    if line.eq_ignore_ascii_case(b"net"){return Some(Command::Help);}
    if line.len()<4||!line[..4].eq_ignore_ascii_case(b"net "){return None;}
    let args=line[4..].trim_ascii_start();
    Some(if args.eq_ignore_ascii_case(b"recv"){Command::Receive}
        else if args.eq_ignore_ascii_case(b"close"){Command::Close}
        else if args.len()>=5&&args[..5].eq_ignore_ascii_case(b"send "){Command::Send(&args[5..])}
        else{Command::Invalid})
}
#[cfg(test)]
mod tests{
    use super::*;
    use std::collections::VecDeque;
    struct Fake{events:VecDeque<Envelope>,now:u64,tick_step:u64,sends:usize,polls:usize,
        yields:usize,input:usize,full:bool,fail:u8}
    impl Fake{fn new()->Self{Self{events:VecDeque::new(),now:0,tick_step:0,sends:0,
        polls:0,yields:0,input:0,full:false,fail:0}}}
    impl Transport for Fake{
        fn ticks(&mut self)->Result<u64,()>{if self.fail==1{return Err(());}
            let n=self.now;self.now=self.now.saturating_add(self.tick_step);Ok(n)}
        fn send(&mut self,b:&[u8])->Result<(),()>{self.sends+=1;assert!(b.starts_with(b"RNET"));
            if self.fail==2{Err(())}else{Ok(())}}
        fn poll(&mut self)->Result<Option<Envelope>,()>{self.polls+=1;
            if self.fail==3{Err(())}else{Ok(self.events.pop_front())}}
        fn input(&mut self,_:[u8;128])->bool{self.input+=1;!self.full}
        fn yield_cpu(&mut self)->Result<(),()>{self.yields+=1;if self.fail==4{Err(())}else{Ok(())}}
    }
    fn response(op:Operation,payload:&[u8])->Envelope{
        let mut b=[0;128];b[..4].copy_from_slice(b"RNET");b[5]=op as u8;b[8]=1;
        b[16..16+payload.len()].copy_from_slice(payload);
        Envelope{peer:7,incarnation:1<<40,length:16+payload.len(),bytes:b}
    }
    #[test]fn authenticated_reply_and_preserved_input(){
        for n in 0..=112{
            let mut f=Fake::new();let mut c=Client::new(1<<40,99).unwrap();
            f.events.push_back(Envelope{peer:0,incarnation:99,length:128,bytes:[0;128]});
            let payload=[b'x';112];f.events.push_back(response(Operation::Receive,&payload[..n]));
            let r=c.call(&mut f,Operation::Receive,b"").unwrap();
            assert_eq!(r.length,n);assert_eq!(&r.bytes[..n],&payload[..n]);
            assert_eq!((f.sends,f.input),(1,1));
        }
    }
    #[test]fn stale_peer_and_unrelated_sender_never_supply_reply(){
        let mut f=Fake::new();let mut c=Client::new(1<<40,99).unwrap();
        for (peer,incarnation)in [(7,1),(6,1<<40),(0,1)]{
            let mut e=response(Operation::Send,b"");e.peer=peer;e.incarnation=incarnation;f.events.push_back(e);
        }
        f.events.push_back(response(Operation::Send,b""));
        assert_eq!(c.call(&mut f,Operation::Send,b"challenge").unwrap().status,Status::Ok);
        assert_eq!((f.sends,f.polls,f.input),(1,4,0));
    }
    #[test]fn timeout_frozen_clock_failures_and_input_overflow_retire_without_retry(){
        for fail in 0..=4{
            let mut f=Fake::new();f.fail=fail;let mut c=Client::new(1<<40,99).unwrap();
            assert!(c.call(&mut f,Operation::Send,b"x").is_err());
            let sends=f.sends;assert!(sends<=1&&f.polls<=POLLS);
            assert!(c.call(&mut f,Operation::Send,b"x").is_err());assert_eq!(f.sends,sends);
        }
        let mut f=Fake::new();f.tick_step=DEADLINE;
        assert_eq!(Client::new(1<<40,99).unwrap().call(&mut f,Operation::Send,b"x"),Err(Error::Timeout));
        assert_eq!(f.polls,0);
        let mut f=Fake::new();f.full=true;
        f.events.push_back(Envelope{peer:0,incarnation:99,length:128,bytes:[0;128]});
        assert_eq!(Client::new(1<<40,99).unwrap().call(&mut f,Operation::Send,b"x"),Err(Error::InputLost));
    }
    #[test]fn malformed_bound_lengths_retire(){
        for n in [0,1,15,129,usize::MAX]{
            let mut f=Fake::new();let mut e=response(Operation::Receive,b"");e.length=n;f.events.push_back(e);
            assert_eq!(Client::new(1<<40,99).unwrap().call(&mut f,Operation::Receive,b""),Err(Error::Invalid));
        }
        let mut f=Fake::new();let mut e=response(Operation::Send,b"");e.bytes[8]=2;f.events.push_back(e);
        assert_eq!(Client::new(1<<40,99).unwrap().call(&mut f,Operation::Send,b"x"),Err(Error::Invalid));
    }
    #[test]fn text_commands_are_bounded_and_explicit(){
        assert_eq!(command(b" net "),Some(Command::Help));
        assert_eq!(command(b"NET RECV"),Some(Command::Receive));
        assert_eq!(command(b"net close"),Some(Command::Close));
        assert_eq!(command(b"net send challenge"),Some(Command::Send(b"challenge")));
        assert_eq!(command(b"net x"),Some(Command::Invalid));
        for bytes in [b"network".as_slice(),b"list",b"net\0",b"", &[b'x';65]]{
            assert_eq!(command(bytes),None);
        }
    }
}
