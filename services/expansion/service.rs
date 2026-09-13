//! Candidate bounded network service composition, not a native activation.
#![forbid(unsafe_code)]
use crate::{channel::{self,Budget,Channel},network::{Endpoint,MAX_FRAME},ne2k};
pub const HEADER:usize=16;
pub const MESSAGE:usize=128;
pub const APP_PAYLOAD:usize=MESSAGE-HEADER;
const MAGIC:[u8;4]=*b"RNET";
/// Trusted driver/clock adapter only. Implementations must not retain slices,
/// block indefinitely, choose host resources, or perform ambient networking.
pub trait Link {
    fn ticks(&mut self)->Result<u64,()>;
    fn send(&mut self,frame:&[u8])->Result<(),()>;
    fn receive(&mut self,out:&mut[u8;MAX_FRAME])->Result<Option<usize>,()>;
    fn close(&mut self);
}
impl<I:ne2k::Io> Link for ne2k::Device<I>{
    fn ticks(&mut self)->Result<u64,()>{self.clock().map_err(|_|())}
    fn send(&mut self,f:&[u8])->Result<(),()>{self.transmit(f).map_err(|_|())}
    fn receive(&mut self,b:&mut[u8;MAX_FRAME])->Result<Option<usize>,()>{
        ne2k::Device::receive(self,b).map_err(|_|())
    }
    fn close(&mut self){ne2k::Device::close(self);}
}
#[derive(Clone,Copy)]
pub struct Policy{
    pub principal:u64,pub incarnation:u64,pub interface:u32,
    pub local:Endpoint,pub peer:Endpoint,pub issued:u64,pub expires:u64,
    pub tx:Budget,pub rx:Budget,
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum Status{Ok=0,Invalid=1,Denied=2,Closed=3,Full=4,Empty=5,Budget=6,Io=7,Oversize=8}
fn status(e:channel::Error)->Status{match e{
    channel::Error::Invalid=>Status::Invalid,channel::Error::Denied=>Status::Denied,
    channel::Error::Closed=>Status::Closed,channel::Error::Full=>Status::Full,
    channel::Error::Empty=>Status::Empty,channel::Error::Budget=>Status::Budget,
    channel::Error::Io=>Status::Io,
}}
pub struct Reply{bytes:[u8;MESSAGE],length:usize}
impl Reply{
    fn new(operation:u8,id:u64,status:Status)->Self{
        let mut r=Self{bytes:[0;MESSAGE],length:HEADER};r.bytes[..4].copy_from_slice(&MAGIC);
        r.bytes[5]=operation;r.bytes[6]=status as u8;r.bytes[8..16].copy_from_slice(&id.to_le_bytes());r
    }
    pub fn bytes(&self)->&[u8]{&self.bytes[..self.length]}
    pub fn status(&self)->Status{match self.bytes[6]{0=>Status::Ok,1=>Status::Invalid,
        2=>Status::Denied,3=>Status::Closed,4=>Status::Full,5=>Status::Empty,
        6=>Status::Budget,7=>Status::Io,_=>Status::Oversize}}
}
/// Bind identity from the real kernel envelope, never request bytes.
pub struct Service<L:Link>{
    link:L,channel:Channel,principal:u64,incarnation:u64,interface:u32,closed:bool,
}
impl<L:Link> Service<L>{
    /// Policy construction is trusted owner policy, not an application request.
    pub fn new(mut link:L,p:Policy)->Result<Self,Status>{
        let channel=match Channel::new(p.principal,p.incarnation,p.interface,p.local,p.peer,
            p.issued,p.expires,p.tx,p.rx){
            Ok(c)=>c,Err(e)=>{link.close();return Err(status(e));}
        };
        Ok(Self{link,channel,principal:p.principal,incarnation:p.incarnation,
            interface:p.interface,closed:false})
    }
    pub fn close(&mut self){
        if !self.closed{self.closed=true;self.channel.revoke();self.link.close();}
    }
    fn time(&mut self)->Result<u64,Status>{
        if self.closed{return Err(Status::Closed);}
        let now=match self.link.ticks(){Ok(n)=>n,Err(())=>{self.close();return Err(Status::Io);}};
        if let Err(e)=self.channel.maintain(now){self.close();return Err(status(e));}Ok(now)
    }
    /// At most one ingress and one egress attempt per call. Malformed ingress
    /// consumes its budget but does not starve a queued transmit in this turn.
    pub fn poll(&mut self)->Result<(),Status>{
        let now=self.time()?;let mut frame=[0u8;MAX_FRAME];
        match self.link.receive(&mut frame){
            Err(())=>{self.close();return Err(Status::Io);},
            Ok(Some(n))=>{
                if !(60..=MAX_FRAME).contains(&n){self.close();return Err(Status::Io);}
                match self.channel.ingress(self.interface,now,&frame[..n]){
                    Ok(())|Err(channel::Error::Invalid)|Err(channel::Error::Full)=>{},
                    Err(e)=>{self.close();return Err(status(e));}
                }
            },Ok(None)=>{},
        }
        // Driver polling can yield: expiry is checked again before transmit.
        let now=self.time()?;let link=&mut self.link;
        match self.channel.transmit(self.interface,now,|f|link.send(f)){
            Ok(())|Err(channel::Error::Empty)=>Ok(()),
            Err(e)=>{self.close();Err(status(e))}
        }
    }
    /// One exact kernel envelope, <=128 bytes. Never grants capabilities.
    /// SEND acknowledgement means queued, not transmitted/delivered.
    pub fn request(&mut self,principal:u64,incarnation:u64,bytes:&[u8])->Reply{
        if principal!=self.principal||incarnation!=self.incarnation{
            return Reply::new(0,0,Status::Denied);
        }
        let (op,id)=match parse(bytes){Some(v)=>v,None=>return Reply::new(0,0,Status::Invalid)};
        if self.closed{return Reply::new(op,id,Status::Closed);}
        let now=match self.time(){Ok(n)=>n,Err(e)=>return Reply::new(op,id,e)};
        if op==3{self.close();return Reply::new(op,id,Status::Ok);}
        if op==1{
            let result=self.channel.enqueue(principal,incarnation,now,&bytes[HEADER..]);
            return Reply::new(op,id,result.map(|_|Status::Ok).unwrap_or_else(status));
        }
        match self.channel.receive(principal,incarnation,now){
            Err(e)=>Reply::new(op,id,status(e)),
            Ok(data)=>{
                if data.bytes().len()>APP_PAYLOAD{return Reply::new(op,id,Status::Oversize);}
                let mut r=Reply::new(op,id,Status::Ok);r.length+=data.bytes().len();
                r.bytes[HEADER..r.length].copy_from_slice(data.bytes());r
            }
        }
    }
}
impl<L:Link> Drop for Service<L>{fn drop(&mut self){self.close();}}
fn parse(bytes:&[u8])->Option<(u8,u64)>{
    if !(HEADER..=MESSAGE).contains(&bytes.len())||bytes[..4]!=MAGIC||
        bytes[4]!=0||bytes[6]!=0||bytes[7]!=0{return None;}
    let op=bytes[5];if !(1..=3).contains(&op)||op!=1&&bytes.len()!=HEADER{return None;}
    let id=u64::from_le_bytes(bytes[8..16].try_into().ok()?);if id==0{return None;}Some((op,id))
}
/// Experimental Rust SDK encoder. Zeroes the entire destination on failure.
pub fn request_bytes(operation:u8,id:u64,payload:&[u8],out:&mut[u8;MESSAGE])->Option<usize>{
    out.fill(0);
    if !(1..=3).contains(&operation)||id==0||payload.len()>APP_PAYLOAD||
        operation!=1&&!payload.is_empty(){return None;}
    out[..4].copy_from_slice(&MAGIC);out[5]=operation;out[8..16].copy_from_slice(&id.to_le_bytes());
    out[HEADER..HEADER+payload.len()].copy_from_slice(payload);Some(HEADER+payload.len())
}

#[cfg(test)]
mod tests{
    use super::*;
    use crate::network;
    const A:Endpoint=Endpoint{mac:[2,0,0,0,0,1],ip:[10,42,0,1],port:4000};
    const B:Endpoint=Endpoint{mac:[2,0,0,0,0,2],ip:[10,42,0,2],port:4001};
    #[derive(Default)]
    struct Wire{now:u64,after_receive:Option<u64>,fail:u8,reads:usize,writes:usize,
        clocks:usize,closed:usize,closed_observer:std::rc::Rc<std::cell::Cell<usize>>,
        bad_length:Option<usize>,incoming:Option<std::vec::Vec<u8>>,outgoing:std::vec::Vec<u8>}
    impl Link for Wire{
        fn ticks(&mut self)->Result<u64,()>{self.clocks+=1;if self.fail==1{Err(())}else{Ok(self.now)}}
        fn send(&mut self,b:&[u8])->Result<(),()>{self.writes+=1;
            if self.fail==2{return Err(());}self.outgoing=b.to_vec();Ok(())}
        fn receive(&mut self,out:&mut[u8;MAX_FRAME])->Result<Option<usize>,()>{
            self.reads+=1;if let Some(n)=self.after_receive{self.now=n;}
            if self.fail==3{return Err(());}if let Some(n)=self.bad_length{return Ok(Some(n));}
            if self.fail==4{return Ok(Some(MAX_FRAME+1));}
            if let Some(b)=self.incoming.take(){out[..b.len()].copy_from_slice(&b);Ok(Some(b.len()))}
            else{Ok(None)}
        }
        fn close(&mut self){self.closed+=1;self.closed_observer.set(self.closed);}
    }
    fn policy()->Policy{Policy{principal:6,incarnation:1<<40,interface:1,local:A,peer:B,
        issued:0,expires:100,tx:Budget{packets:8,bytes:4096},rx:Budget{packets:8,bytes:4096}}}
    fn service()->Service<Wire>{Service::new(Wire::default(),policy()).unwrap()}
    fn req(op:u8,data:&[u8])->std::vec::Vec<u8>{
        let mut out=[0;MESSAGE];let n=request_bytes(op,u64::MAX,data,&mut out).unwrap();out[..n].to_vec()
    }
    fn incoming(data:&[u8])->std::vec::Vec<u8>{
        let mut out=[0;MAX_FRAME];let n=network::encode(B,A,0,data,&mut out).unwrap();out[..n].to_vec()
    }
    fn request(s:&mut Service<Wire>,op:u8,data:&[u8])->Reply{s.request(6,1<<40,&req(op,data))}
    fn assert_closed(s:&mut Service<Wire>){
        let before=(s.link.reads,s.link.writes,s.link.clocks,s.link.closed);
        assert_eq!(s.poll(),Err(Status::Closed));assert_eq!(request(s,1,b"x").status(),Status::Closed);
        s.close();assert_eq!((s.link.reads,s.link.writes,s.link.clocks,s.link.closed),before);
        assert_eq!(s.channel.pending(),(0,0));assert_eq!(s.link.closed,1);
    }
    #[test]fn service_copies_application_request_and_actual_driver_frame(){
        let mut s=service();let mut r=req(1,b"challenge");
        assert_eq!(s.request(6,1<<40,&r).status(),Status::Ok);r.fill(0);
        assert_eq!(s.link.writes,0);s.poll().unwrap();
        assert_eq!(network::decode(&s.link.outgoing,B,A),Ok(&b"challenge"[..]));
        s.link.incoming=Some(incoming(b"response"));s.poll().unwrap();
        let reply=request(&mut s,2,b"");
        assert_eq!(reply.status(),Status::Ok);assert_eq!(&reply.bytes()[HEADER..],b"response");
        assert_eq!(&reply.bytes()[8..16],&u64::MAX.to_le_bytes());
    }
    #[test]fn untrusted_identity_and_malformed_messages_have_no_io_or_budget_effect(){
        let mut s=service();let r=req(1,b"x");
        for (p,i)in [(7,1<<40),(6,1),(0,0)]{
            assert_eq!(s.request(p,i,&r).status(),Status::Denied);
        }
        for n in 0..HEADER{assert_eq!(s.request(6,1<<40,&r[..n]).status(),Status::Invalid);}
        for field in [0,1,2,3,4,5,6,7]{
            let mut bad=r.clone();bad[field]=255;assert_eq!(s.request(6,1<<40,&bad).status(),Status::Invalid);
        }
        let mut bad=r.clone();bad[8..16].fill(0);
        assert_eq!(s.request(6,1<<40,&bad).status(),Status::Invalid);
        for op in [2,3]{let mut bad=r.clone();bad[5]=op;
            assert_eq!(s.request(6,1<<40,&bad).status(),Status::Invalid);}
        assert_eq!(s.request(6,1<<40,&[0;129]).status(),Status::Invalid);
        assert_eq!((s.link.clocks,s.link.reads,s.link.writes),(0,0,0));
        assert_eq!(s.channel.budgets(),(policy().tx,policy().rx));
    }
    #[test]fn all_application_lengths_round_trip_and_oversize_is_not_truncated(){
        for n in 0..=APP_PAYLOAD{
            let mut s=service();let data=std::vec![0x37;n];
            assert_eq!(request(&mut s,1,&data).status(),Status::Ok);s.poll().unwrap();
            assert_eq!(network::decode(&s.link.outgoing,B,A),Ok(&data[..]));
            s.link.incoming=Some(incoming(&data));s.poll().unwrap();
            let r=request(&mut s,2,b"");assert_eq!(r.status(),Status::Ok);
            assert_eq!(&r.bytes()[HEADER..],&data);
        }
        let mut s=service();s.link.incoming=Some(incoming(&[0x77;APP_PAYLOAD+1]));s.poll().unwrap();
        let r=request(&mut s,2,b"");assert_eq!(r.status(),Status::Oversize);assert_eq!(r.bytes().len(),HEADER);
        assert_eq!(request(&mut s,2,b"").status(),Status::Empty);
    }
    #[test]fn expiry_between_receive_and_send_prevents_transmission(){
        for time in [100,u64::MAX]{
            let mut s=service();request(&mut s,1,b"queued");s.link.after_receive=Some(time);
            assert_eq!(s.poll(),Err(Status::Closed));assert_eq!(s.link.writes,0);assert_closed(&mut s);
        }
        let mut s=service();s.link.now=10;request(&mut s,1,b"queued");s.link.after_receive=Some(9);
        assert_eq!(s.poll(),Err(Status::Closed));assert_eq!(s.link.writes,0);assert_closed(&mut s);
    }
    #[test]fn every_link_failure_revokes_both_queues_and_never_retries(){
        for fault in 1..=4{
            let mut s=service();s.link.incoming=Some(incoming(b"queued reply"));s.poll().unwrap();
            request(&mut s,1,b"one");request(&mut s,1,b"two");
            assert_eq!(s.channel.pending(),(2,1));
            s.link.fail=fault;assert_eq!(s.poll(),Err(Status::Io));assert_closed(&mut s);
        }
    }
    #[test]fn malformed_ingress_does_not_starve_one_queued_send(){
        let mut s=service();request(&mut s,1,b"out");s.link.incoming=Some(std::vec![0;60]);
        s.poll().unwrap();assert_eq!((s.link.reads,s.link.writes),(1,1));
        assert_eq!(s.channel.stats().dropped,1);
    }
    #[test]fn receive_budget_and_client_close_are_sticky(){
        let mut p=policy();p.rx=Budget{packets:1,bytes:60};let mut s=Service::new(Wire::default(),p).unwrap();
        s.link.incoming=Some(incoming(&[1;100]));assert_eq!(s.poll(),Err(Status::Budget));assert_closed(&mut s);
        let mut s=service();request(&mut s,1,b"queued");
        assert_eq!(request(&mut s,3,b"").status(),Status::Ok);assert_closed(&mut s);
    }
    #[test]fn encoder_rejects_invalid_shape_and_erases_destination(){
        for (op,id,n)in [(0,1,0),(4,1,0),(1,0,0),(1,1,113),(2,1,1),(3,1,1)]{
            let mut out=[0xaa;MESSAGE];
            assert_eq!(request_bytes(op,id,&std::vec![0;n],&mut out),None);assert_eq!(out,[0;MESSAGE]);
        }
    }

    #[test]fn every_invalid_driver_length_closes_before_slice_construction(){
        for n in [0,1,59,MAX_FRAME+1,usize::MAX]{
            let mut s=service();s.link.bad_length=Some(n);
            assert_eq!(s.poll(),Err(Status::Io));assert_closed(&mut s);
        }
    }
    #[test]fn invalid_policy_closes_supplied_link_exactly_once(){
        for field in 0..5{
            let link=Wire::default();let closed=link.closed_observer.clone();
            let mut p=policy();match field{
                0=>p.principal=0,1=>p.incarnation=0,2=>p.interface=0,
                3=>p.expires=p.issued,_=>p.tx.packets=0,
            }
            assert!(matches!(Service::new(link,p),Err(Status::Invalid)));
            assert_eq!(closed.get(),1);
        }
    }
}
