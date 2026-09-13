//! Experimental bounded network client protocol. No syscall or ambient authority.
#![forbid(unsafe_code)]
pub const HEADER:usize=16;
pub const MESSAGE:usize=128;
pub const PAYLOAD:usize=MESSAGE-HEADER;
const MAGIC:[u8;4]=*b"RNET";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum Operation{Send=1,Receive=2,Close=3}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
#[repr(u8)]
pub enum Status{Ok=0,Invalid=1,Denied=2,Closed=3,Full=4,Empty=5,Budget=6,Io=7,Oversize=8}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Invalid,Busy,Closed,Exhausted,Peer,Unexpected}
#[derive(Debug,PartialEq,Eq)]
pub struct Response<'a>{pub operation:Operation,pub status:Status,pub payload:&'a[u8]}
/// Encode caller-owned bytes only. Output storage must not be shared with input.
pub fn encode(operation:u8,id:u64,payload:&[u8],out:&mut[u8;MESSAGE])->Option<usize>{
    out.fill(0);
    if !(1..=3).contains(&operation)||id==0||payload.len()>PAYLOAD||
        operation!=1&&!payload.is_empty(){return None;}
    out[..4].copy_from_slice(&MAGIC);out[5]=operation;
    out[8..16].copy_from_slice(&id.to_le_bytes());
    out[HEADER..HEADER+payload.len()].copy_from_slice(payload);Some(HEADER+payload.len())
}
/// One outstanding request per client, no automatic retry or retransmission.
/// Peer identity must come from a real kernel envelope, not message bytes.
/// This value does not grant SEND authority; the kernel capability is separate.
pub struct Client{peer:u64,incarnation:u64,next:u64,pending:Option<(Operation,u64)>,closed:bool}
impl Client{
    pub fn new(peer:u64,incarnation:u64)->Result<Self,Error>{
        if peer!=7||incarnation==0{return Err(Error::Invalid);}
        Ok(Self{peer,incarnation,next:1,pending:None,closed:false})
    }
    pub fn begin(&mut self,operation:Operation,payload:&[u8],out:&mut[u8;MESSAGE])->Result<usize,Error>{
        out.fill(0);
        if self.closed{return Err(Error::Closed);}
        if self.pending.is_some(){return Err(Error::Busy);}
        let next=self.next.checked_add(1).ok_or(Error::Exhausted)?;
        let n=encode(operation as u8,self.next,payload,out).ok_or(Error::Invalid)?;
        self.pending=Some((operation,self.next));self.next=next;Ok(n)
    }
    pub fn accept<'a>(&mut self,peer:u64,incarnation:u64,raw:&'a[u8])->Result<Response<'a>,Error>{
        if peer!=self.peer||incarnation!=self.incarnation{return Err(Error::Peer);}
        if self.closed{return Err(Error::Closed);}
        let (operation,id)=self.pending.ok_or(Error::Unexpected)?;
        if !(HEADER..=MESSAGE).contains(&raw.len())||raw[..4]!=MAGIC||
            raw[4]!=0||raw[5]!=operation as u8||raw[7]!=0||
            raw[8..16]!=id.to_le_bytes(){return Err(Error::Invalid);}
        let status=match raw[6]{0=>Status::Ok,1=>Status::Invalid,2=>Status::Denied,
            3=>Status::Closed,4=>Status::Full,5=>Status::Empty,6=>Status::Budget,
            7=>Status::Io,8=>Status::Oversize,_=>return Err(Error::Invalid)};
        if (operation!=Operation::Receive||status!=Status::Ok)&&raw.len()!=HEADER{
            return Err(Error::Invalid);
        }
        self.pending=None;
        if status==Status::Closed||status==Status::Io||
            operation==Operation::Close&&status==Status::Ok{self.closed=true;}
        Ok(Response{operation,status,payload:&raw[HEADER..]})
    }
    /// Abandoning an uncertain request permanently retires this client.
    /// It does not cancel remote effects or reclaim its budget. Never auto-retry.
    pub fn retire(&mut self){self.closed=true;self.pending=None;}
}
#[cfg(test)]
mod tests{
    use super::*;
    fn reply(op:Operation,id:u64,status:u8,payload:&[u8])->([u8;MESSAGE],usize){
        let mut out=[0;MESSAGE];let n=HEADER+payload.len();
        out[..4].copy_from_slice(b"RNET");out[5]=op as u8;out[6]=status;
        out[8..16].copy_from_slice(&id.to_le_bytes());out[HEADER..n].copy_from_slice(payload);(out,n)
    }
    #[test]fn exact_request_bytes_and_correlated_reply(){
        let mut c=Client::new(7,1<<40).unwrap();let mut out=[0xaa;MESSAGE];
        assert_eq!(c.begin(Operation::Send,b"abc",&mut out),Ok(19));
        assert_eq!(&out[..19],b"RNET\0\x01\0\0\x01\0\0\0\0\0\0\0abc");
        let (r,n)=reply(Operation::Send,1,0,b"");
        assert_eq!(c.accept(7,1<<40,&r[..n]).unwrap().status,Status::Ok);
        assert_eq!(c.accept(7,1<<40,&r[..n]),Err(Error::Unexpected));
        assert_eq!(c.begin(Operation::Receive,b"",&mut out),Ok(16));
        let (r,n)=reply(Operation::Receive,2,0,b"answer");
        assert_eq!(c.accept(7,1<<40,&r[..n]).unwrap().payload,b"answer");
    }
    #[test]fn wrong_peer_and_malformed_reply_preserve_pending_request(){
        let mut c=Client::new(7,u64::MAX).unwrap();let mut out=[0;MESSAGE];
        c.begin(Operation::Receive,b"",&mut out).unwrap();
        let (r,n)=reply(Operation::Receive,1,0,b"");
        for (peer,generation)in [(6,u64::MAX),(7,1),(7,0)]{
            assert_eq!(c.accept(peer,generation,&r[..n]),Err(Error::Peer));
        }
        for size in 0..HEADER{assert_eq!(c.accept(7,u64::MAX,&r[..size]),Err(Error::Invalid));}
        for field in [0,1,2,3,4,5,6,7,8,15]{
            let mut bad=r;bad[field]=255;
            assert_eq!(c.accept(7,u64::MAX,&bad[..n]),Err(Error::Invalid));
        }
        assert_eq!(c.accept(7,u64::MAX,&[0;129]),Err(Error::Invalid));
        assert_eq!(c.begin(Operation::Receive,b"",&mut out),Err(Error::Busy));
        assert_eq!(out,[0;MESSAGE]);
        assert!(c.accept(7,u64::MAX,&r[..n]).is_ok());
    }
    #[test]fn only_successful_receive_may_carry_payload(){
        for op in [Operation::Send,Operation::Receive,Operation::Close]{
            for status in 0..=8{
                let mut c=Client::new(7,1).unwrap();let mut out=[0;MESSAGE];
                c.begin(op,b"",&mut out).unwrap();
                let(r,n)=reply(op,1,status,b"x");
                assert_eq!(c.accept(7,1,&r[..n]).is_ok(),op==Operation::Receive&&status==0);
            }
        }
    }
    #[test]fn closure_uncertainty_and_id_exhaustion_never_replay(){
        for status in [0,3,7]{
            let mut c=Client::new(7,1).unwrap();let mut out=[0;MESSAGE];
            c.begin(Operation::Close,b"",&mut out).unwrap();
            let(r,n)=reply(Operation::Close,1,status,b"");
            c.accept(7,1,&r[..n]).unwrap();
            assert_eq!(c.begin(Operation::Send,b"x",&mut out),Err(Error::Closed));
        }
        let mut c=Client::new(7,1).unwrap();let mut out=[0;MESSAGE];
        c.begin(Operation::Send,b"x",&mut out).unwrap();c.retire();
        assert_eq!(c.begin(Operation::Send,b"x",&mut out),Err(Error::Closed));
        let mut c=Client::new(7,1).unwrap();c.next=u64::MAX;
        assert_eq!(c.begin(Operation::Send,b"x",&mut out),Err(Error::Exhausted));
        assert!(c.pending.is_none());assert_eq!(out,[0;MESSAGE]);
    }
    #[test]fn invalid_begin_has_no_sequence_or_pending_effect(){
        let mut c=Client::new(7,1).unwrap();let mut out=[0xaa;MESSAGE];
        assert_eq!(c.begin(Operation::Receive,b"x",&mut out),Err(Error::Invalid));
        assert_eq!(c.begin(Operation::Send,&[0;113],&mut out),Err(Error::Invalid));
        assert_eq!((c.next,c.pending),(1,None));assert_eq!(out,[0;MESSAGE]);
        for (p,g)in [(0,1),(6,1),(8,1),(7,0)]{assert!(Client::new(p,g).is_err());}
    }
}
