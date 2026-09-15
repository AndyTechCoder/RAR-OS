//! Fixed cloud-only peer fault adapter; never selected by normal Alpha builds.
//! Mutates only bounded frames already authorized for this closed lab peer.
//! No sockets, paths, raw host devices, endpoint selection or new guest grant.
#![forbid(unsafe_code)]
use crate::{network::MAX_FRAME,network_service::Link};
pub struct Peer<L:Link>{inner:L,sent:u8,closed:bool}
impl<L:Link> Peer<L>{
    pub fn new(inner:L)->Self{Self{inner,sent:0,closed:false}}
}
impl<L:Link> Link for Peer<L>{
    fn ticks(&mut self)->Result<u64,()>{if self.closed{Err(())}else{self.inner.ticks()}}
    fn send(&mut self,frame:&[u8])->Result<(),()>{
        if self.closed||self.sent==16||frame.len()!=74{return Err(());}
        let ordinal=self.sent;self.sent+=1;
        let mut bytes=[0u8;MAX_FRAME];bytes[..frame.len()].copy_from_slice(frame);
        match ordinal{
            0=>{bytes[14]=0x65;self.inner.send(&bytes[..74])}, // malformed IP version
            1=>self.inner.send(&bytes[..60]), // truncated IP/UDP relative to header length
            2=>{bytes[40]^=1;self.inner.send(&bytes[..74])}, // invalid UDP checksum
            3=>Ok(()), // deliberate one-shot drop: ACK is NOT delivery evidence
            _=>self.inner.send(&bytes[..74]),
        }
    }
    fn receive(&mut self,out:&mut[u8;MAX_FRAME])->Result<Option<usize>,()>{
        if self.closed{Err(())}else{self.inner.receive(out)}
    }
    fn close(&mut self){if !self.closed{self.closed=true;self.inner.close();}}
}
impl<L:Link> Drop for Peer<L>{fn drop(&mut self){self.close();}}
#[cfg(test)]mod tests{
    use super::*;
    #[derive(Default)]struct Wire{frames:std::vec::Vec<std::vec::Vec<u8>>,closed:usize}
    impl Link for Wire{
        fn ticks(&mut self)->Result<u64,()>{Ok(1)}
        fn send(&mut self,f:&[u8])->Result<(),()>{self.frames.push(f.to_vec());Ok(())}
        fn receive(&mut self,_:&mut[u8;MAX_FRAME])->Result<Option<usize>,()>{Ok(None)}
        fn close(&mut self){self.closed+=1;}
    }
    #[test]fn exact_faults_once_no_retry_or_extra_transmission(){
        let mut p=Peer::new(Wire::default());let frame=[7;74];
        for _ in 0..5{p.send(&frame).unwrap();}
        assert_eq!(p.inner.frames.len(),4);
        assert_eq!(p.inner.frames[0][14],0x65);assert_eq!(p.inner.frames[1].len(),60);
        assert_eq!(p.inner.frames[2][40],6);assert_eq!(p.inner.frames[3],frame);
        assert!(p.send(&[0;75]).is_err());assert_eq!(p.sent,5);
        p.close();p.close();assert_eq!(p.inner.closed,1);
        assert!(p.send(&frame).is_err());assert!(p.ticks().is_err());
    }
}
