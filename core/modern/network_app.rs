//! Native Terminal network tool: private syscall adapter around the standalone SDK.
use crate::{abi::*,syscall,poll_checked,yield_now,services,sdk};
use crate::network_client::{self,Transport,Envelope,Reply,Error,Command};
use services::apps::{Pending,View};
pub struct App{client:Option<network_client::Client>}
struct Io<'a>{boot:&'a Boot,input:&'a mut Pending}
impl Transport for Io<'_>{
    fn ticks(&mut self)->Result<u64,()>{let n=syscall(TICKS,0,0,0,0);if n<0{Err(())}else{Ok(n as u64)}}
    fn send(&mut self,bytes:&[u8])->Result<(),()>{
        if syscall(SEND,self.boot.caps[5],bytes.as_ptr()as u64,bytes.len()as u64,0)==0{Ok(())}else{Err(())}
    }
    fn poll(&mut self)->Result<Option<Envelope>,()>{
        poll_checked(self.boot.caps[SELF_RECV]).map(|e|e.map(|e|Envelope{
            peer:e.sender,incarnation:e.generation,length:e.length as usize,bytes:e.bytes}))
    }
    fn input(&mut self,bytes:[u8;128])->bool{
        crate::session::Input::push(self.input,bytes)
    }
    fn yield_cpu(&mut self)->Result<(),()>{yield_now();Ok(())}
}
impl App{
    pub fn new(boot:&Boot)->Self{
        // Exact Denied before PIO for absent and unrelated granted handles.
        for handle in [0,boot.caps[SELF_RECV],boot.caps[5]]{
            if syscall(NETWORK,handle,0,7,0)!=-2{crate::fail();}
        }
        Self{client:if boot.version==EXPANSION_VERSION&&boot.role==6{
            network_client::Client::new(boot.peers[7],boot.peers[0]).ok()
        }else{None}}
    }
    pub fn execute(&mut self,boot:&Boot,input:&mut Pending,line:&[u8],view:&mut View)->bool{
        let Some(command)=network_client::command(line) else{return false;};
        view.line(0,b"RAR NETWORK - PUBLIC LAB PEER ONLY");
        let(op,payload)=match command{
            Command::Help=>{view.line(1,b"NET SEND text | NET RECV | NET CLOSE");
                view.line(2,b"UDP: NO ENCRYPTION OR PEER AUTHENTICATION");return true;},
            Command::Invalid=>{view.line(1,b"INVALID NET COMMAND");return true;},
            Command::Send(bytes)=>(sdk::Operation::Send,bytes),
            Command::Receive=>(sdk::Operation::Receive,b"".as_slice()),
            Command::Close=>(sdk::Operation::Close,b"".as_slice()),
        };
        let Some(client)=self.client.as_mut() else{
            view.line(1,b"NETWORK UNAVAILABLE");return true;
        };
        match client.call(&mut Io{boot,input},op,payload){
            Ok(reply)=>render(op,&reply,view),
            Err(error)=>{
                view.line(1,match error{Error::Timeout=>b"NETWORK TIMEOUT - NOT RETRIED",
                    Error::InputLost=>b"INPUT LOST - REENTER COMMAND",
                    Error::Invalid=>b"INVALID NETWORK REPLY",
                    Error::Unavailable=>b"NETWORK UNAVAILABLE"});
                view.line(2,b"EFFECT MAY HAVE OCCURRED; CLIENT RETIRED");
                if error==Error::InputLost{while input.pop().is_some(){}}
            },
        }
        true
    }
}
fn render(op:sdk::Operation,r:&Reply,v:&mut View){
    use sdk::Status::*;
    if r.status!=Ok{
        v.line(1,match r.status{
            Invalid=>b"INVALID REQUEST",Denied=>b"NETWORK REQUEST DENIED",
            Closed=>b"NETWORK CHANNEL CLOSED",Full=>b"NETWORK QUEUE FULL",
            Empty=>b"NO PEER DATAGRAM",Budget=>b"NETWORK BUDGET EXHAUSTED",
            Io=>b"NETWORK DEVICE FAILED",Oversize=>b"DATAGRAM TOO LARGE FOR APP",
            Ok=>b"",
        });return;
    }
    if op==sdk::Operation::Send{
        v.line(1,b"QUEUED ONCE - DELIVERY NOT CONFIRMED");return;
    }
    if op==sdk::Operation::Close{v.line(1,b"NETWORK CHANNEL CLOSED");return;}
    if r.bytes[..r.length].iter().any(|b|!(32..=126).contains(b)){
        v.line(1,b"RECEIVED NON-TEXT DATAGRAM");v.line(2,b"NOT RENDERED AS COMMANDS OR CONTROL TEXT");return;
    }
    v.line(0,b"PEER DATAGRAM - UNAUTHENTICATED");
    v.line(1,&r.bytes[..r.length.min(48)]);
    v.line(2,&r.bytes[r.length.min(48)..r.length.min(96)]);
    v.line(5,&r.bytes[r.length.min(96)..r.length]);
}
