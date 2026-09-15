//! Service-owned static-peer channel. No ambient device/network authority.
//! All caller identities and ticks must come from kernel envelope/clock adapters.
#![forbid(unsafe_code)]
use crate::network::{self, Endpoint, Grant, MAX_FRAME, MAX_PAYLOAD};
pub const DEPTH: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Denied, Closed, Full, Empty, Budget, Io }
#[derive(Clone, Copy)]
struct Frame { bytes: [u8; MAX_FRAME], len: usize }
impl Frame { const EMPTY: Self = Self { bytes: [0; MAX_FRAME], len: 0 }; }
#[derive(Clone, Copy)]
pub struct Datagram { bytes: [u8; MAX_PAYLOAD], len: usize }
impl Datagram {
    const EMPTY: Self = Self { bytes: [0; MAX_PAYLOAD], len: 0 };
    pub fn bytes(&self) -> &[u8] { &self.bytes[..self.len] }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget { pub packets: u32, pub bytes: u32 }
impl Budget {
    fn consume(&mut self, bytes: usize) -> Result<(), Error> {
        if self.packets == 0 || bytes > self.bytes as usize { return Err(Error::Budget); }
        self.packets -= 1; self.bytes -= bytes as u32; Ok(())
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Stats { pub sent: u32, pub received: u32, pub dropped: u32, pub io_failed: u32 }
/// Non-Clone ownership: a live channel cannot be snapshotted to restore budgets.
/// Construction is trusted service policy, never an application operation.
pub struct Channel {
    principal: u64, incarnation: u64, interface: u32,
    local: Endpoint, peer: Endpoint, expires: u64, clock: u64, closed: bool,
    grant: Grant, tx_budget: Budget, rx_budget: Budget,
    tx: [Frame; DEPTH], rx: [Datagram; DEPTH], tx_len: usize, rx_len: usize,
    packet_id: u16, stats: Stats,
}
impl Channel {
    pub fn new(principal: u64, incarnation: u64, interface: u32,
               local: Endpoint, peer: Endpoint, now: u64, expires: u64,
               tx: Budget, rx: Budget) -> Result<Self, Error> {
        if interface == 0 || !local.valid() || !peer.valid() || local == peer ||
            local.mac == peer.mac || local.ip == peer.ip || tx.packets == 0 ||
            rx.packets == 0 || tx.bytes < 60 || rx.bytes < 60 {
            return Err(Error::Invalid);
        }
        let grant = Grant::new(principal, incarnation, peer, now, expires,
            tx.packets, tx.bytes).map_err(|_|Error::Invalid)?;
        Ok(Self { principal, incarnation, interface, local, peer, expires, clock: now,
            closed: false, grant, tx_budget: tx, rx_budget: rx,
            tx: [Frame::EMPTY;DEPTH], rx: [Datagram::EMPTY;DEPTH],
            tx_len:0, rx_len:0, packet_id:0, stats:Stats::default() })
    }
    fn live(&mut self, now: u64) -> Result<(), Error> {
        if self.closed { return Err(Error::Closed); }
        if now < self.clock || now >= self.expires {
            self.revoke(); return Err(Error::Closed);
        }
        self.clock = now; Ok(())
    }
    fn caller(&mut self, principal:u64, incarnation:u64, now:u64) -> Result<(),Error> {
        // Unauthenticated arguments never advance this service's clock.
        if principal != self.principal || incarnation != self.incarnation {
            return Err(Error::Denied);
        }
        self.live(now)
    }
    /// Linearizes revocation immediately; no queued send/receive survives.
    /// Already delivered bytes cannot be recalled.
    pub fn revoke(&mut self) {
        self.closed = true; self.grant.revoke();
        self.tx.fill(Frame::EMPTY);self.rx.fill(Datagram::EMPTY);
        self.tx_len=0;self.rx_len=0;
    }
    /// Check expiry even when there are no packets or application requests.
    pub fn maintain(&mut self,now:u64)->Result<(),Error>{self.live(now)}
    pub fn stats(&self) -> Stats { self.stats }
    pub fn budgets(&self) -> (Budget,Budget) { (self.tx_budget,self.rx_budget) }
    pub fn pending(&self) -> (usize,usize) { (self.tx_len,self.rx_len) }
    /// Copy and encode the exact immutable frame owned by this service before
    /// reserving its wire budget. Neither endpoints nor interface are caller input.
    pub fn enqueue(&mut self, principal:u64, incarnation:u64, now:u64,
                   payload:&[u8]) -> Result<(),Error> {
        self.caller(principal,incarnation,now)?;
        if payload.len()>MAX_PAYLOAD { return Err(Error::Invalid); }
        if self.tx_len==DEPTH { return Err(Error::Full); }
        let mut frame=Frame::EMPTY;
        frame.len=network::encode(self.local,self.peer,self.packet_id,payload,&mut frame.bytes)
            .map_err(|_|Error::Invalid)?;
        // Check both counters before mutating either. Grant remains a second
        // internal check, not an externally obtainable capability.
        if self.tx_budget.packets==0 || frame.len>self.tx_budget.bytes as usize {
            return Err(Error::Budget);
        }
        self.grant.reserve(principal,incarnation,self.peer,now,payload.len())
            .map_err(|_|Error::Budget)?;
        self.tx_budget.consume(frame.len)?;
        self.packet_id=self.packet_id.wrapping_add(1);
        self.tx[self.tx_len]=frame;self.tx_len+=1;Ok(())
    }
    /// A single synchronous driver attempt while exclusively borrowing Channel.
    /// The trusted adapter must enforce a deadline and must not retain the slice.
    /// Revocation linearizes when processed by this single-owner service loop.
    /// Failure consumes the queued frame and reservation; no invisible retry.
    pub fn transmit(&mut self, interface:u32, now:u64,
                    write:impl FnOnce(&[u8])->Result<(),()>) -> Result<(),Error> {
        if interface!=self.interface { return Err(Error::Denied); }
        self.live(now)?;
        if self.tx_len==0 { return Err(Error::Empty); }
        let frame=self.tx[0];
        self.tx.copy_within(1..self.tx_len,0);self.tx_len-=1;
        self.tx[self.tx_len]=Frame::EMPTY;
        match write(&frame.bytes[..frame.len]) {
            Ok(())=>{self.stats.sent=self.stats.sent.saturating_add(1);Ok(())},
            Err(())=>{self.stats.io_failed=self.stats.io_failed.saturating_add(1);self.revoke();Err(Error::Io)}
        }
    }
    /// Charge ALL ingress, including invalid frames and padding, before parsing.
    /// The driver must itself bound descriptor lengths before constructing slices.
    pub fn ingress(&mut self, interface:u32, now:u64, frame:&[u8]) -> Result<(),Error> {
        if interface!=self.interface { return Err(Error::Denied); }
        self.live(now)?;
        if let Err(error)=self.rx_budget.consume(frame.len()) {
            self.stats.dropped=self.stats.dropped.saturating_add(1);self.revoke();return Err(error);
        }
        let payload=match network::decode(frame,self.local,self.peer) {
            Ok(payload)=>payload,
            Err(_)=>{self.stats.dropped=self.stats.dropped.saturating_add(1);return Err(Error::Invalid)}
        };
        if self.rx_len==DEPTH {
            self.stats.dropped=self.stats.dropped.saturating_add(1);return Err(Error::Full);
        }
        let mut datagram=Datagram::EMPTY;
        datagram.bytes[..payload.len()].copy_from_slice(payload);datagram.len=payload.len();
        self.rx[self.rx_len]=datagram;self.rx_len+=1;
        self.stats.received=self.stats.received.saturating_add(1);Ok(())
    }
    pub fn receive(&mut self, principal:u64, incarnation:u64, now:u64)->Result<Datagram,Error>{
        self.caller(principal,incarnation,now)?;
        if self.rx_len==0{return Err(Error::Empty);}
        let value=self.rx[0];
        self.rx.copy_within(1..self.rx_len,0);self.rx_len-=1;
        self.rx[self.rx_len]=Datagram::EMPTY;Ok(value)
    }
}
impl Drop for Channel {
    fn drop(&mut self) {
        // Logical lifetime cleanup. Physical page scrubbing remains the kernel's
        // responsibility; optimized stores are not a cryptographic erase claim.
        self.revoke();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const A:Endpoint=Endpoint{mac:[2,0,0,0,0,1],ip:[10,42,0,1],port:4000};
    const B:Endpoint=Endpoint{mac:[2,0,0,0,0,2],ip:[10,42,0,2],port:4001};
    fn channel()->Channel{
        Channel::new(6,1<<40,1,A,B,10,100,Budget{packets:8,bytes:4096},
            Budget{packets:8,bytes:4096}).unwrap()
    }
    fn inbound(data:&[u8])->([u8;MAX_FRAME],usize){
        let mut frame=[0;MAX_FRAME];let n=network::encode(B,A,1,data,&mut frame).unwrap();(frame,n)
    }
    #[test] fn service_owned_bytes_and_endpoints_cannot_change_after_enqueue(){
        let mut c=channel();let mut data=*b"hello";
        c.enqueue(6,1<<40,10,&data).unwrap();data.fill(b'x');
        let mut observed=false;
        c.transmit(1,11,|f|{assert_eq!(network::decode(f,B,A),Ok(&b"hello"[..]));observed=true;Ok(())}).unwrap();
        assert!(observed);assert_eq!(c.pending(),(0,0));
        assert_eq!(c.stats().sent,1);assert_eq!(c.budgets().0,Budget{packets:7,bytes:4036});
    }
    #[test] fn incorrect_identity_interface_and_clock_never_reach_driver(){
        let mut c=channel();
        assert_eq!(c.enqueue(7,1<<40,u64::MAX,b"x"),Err(Error::Denied));
        assert_eq!(c.enqueue(6,1,10,b"x"),Err(Error::Denied));
        c.enqueue(6,1<<40,10,b"x").unwrap();
        assert_eq!(c.transmit(2,u64::MAX,|_|panic!()),Err(Error::Denied));
        assert_eq!(c.transmit(1,9,|_|panic!()),Err(Error::Closed));
        assert_eq!(c.transmit(1,11,|_|panic!()),Err(Error::Closed));
        assert_eq!(c.pending(),(0,0));
    }
    #[test] fn expiry_and_revocation_discard_both_queues(){
        for revoke in [true,false]{
            let mut c=channel();let (f,n)=inbound(b"reply");
            c.enqueue(6,1<<40,10,b"x").unwrap();c.ingress(1,10,&f[..n]).unwrap();
            assert_eq!(c.pending(),(1,1));
            if revoke {c.revoke();}
            assert_eq!(c.transmit(1,100,|_|panic!()),Err(Error::Closed));
            assert_eq!(c.receive(6,1<<40,11).err(),Some(Error::Closed));
            assert_eq!(c.pending(),(0,0));
        }
    }
    #[test] fn bounded_queues_preserve_fifo_and_full_is_not_retried(){
        let mut c=channel();
        for x in 0..DEPTH {c.enqueue(6,1<<40,10,&[x as u8]).unwrap();}
        let before=c.budgets();
        assert_eq!(c.enqueue(6,1<<40,10,b"extra"),Err(Error::Full));
        assert_eq!(c.budgets(),before);
        for x in 0..DEPTH {
            c.transmit(1,11,|f|{assert_eq!(network::decode(f,B,A),Ok(&[x as u8][..]));Ok(())}).unwrap();
        }
        assert_eq!(c.transmit(1,11,|_|panic!()),Err(Error::Empty));
    }
    #[test] fn uncertain_io_consumes_frame_without_automatic_replay(){
        let mut c=channel();c.enqueue(6,1<<40,10,b"x").unwrap();c.enqueue(6,1<<40,10,b"second").unwrap();let before=c.budgets();
        assert_eq!(c.transmit(1,11,|_|Err(())),Err(Error::Io));
        assert_eq!(c.transmit(1,12,|_|panic!()),Err(Error::Closed));
        assert_eq!(c.pending(),(0,0));
        assert_eq!(c.budgets(),before);assert_eq!(c.stats().io_failed,1);
    }
    #[test] fn ingress_charges_invalid_padding_and_full_frames_before_parsing(){
        let mut c=channel();
        assert_eq!(c.ingress(1,10,&[0;MAX_FRAME]),Err(Error::Invalid));
        assert_eq!(c.budgets().1,Budget{packets:7,bytes:4096-MAX_FRAME as u32});
        let (f,n)=inbound(b"x");
        for _ in 0..DEPTH {c.ingress(1,10,&f[..n]).unwrap();}
        assert_eq!(c.ingress(1,10,&f[..n]),Err(Error::Full));
        assert_eq!(c.stats().dropped,2);
        for _ in 0..DEPTH {assert_eq!(c.receive(6,1<<40,11).unwrap().bytes(),b"x");}
        assert_eq!(c.receive(6,1<<40,11).err(),Some(Error::Empty));
    }
    #[test] fn receive_is_copied_scoped_and_service_restart_has_no_old_queues(){
        let mut c=channel();let(mut f,n)=inbound(b"secret");
        c.ingress(1,10,&f[..n]).unwrap();f.fill(0);
        assert_eq!(c.receive(7,1<<40,u64::MAX).err(),Some(Error::Denied));
        assert_eq!(c.receive(6,1,11).err(),Some(Error::Denied));
        assert_eq!(c.receive(6,1<<40,11).unwrap().bytes(),b"secret");
        drop(c);
        let mut fresh=Channel::new(6,(1<<40)+1,1,A,B,10,100,
            Budget{packets:1,bytes:60},Budget{packets:1,bytes:60}).unwrap();
        assert_eq!(fresh.enqueue(6,1<<40,10,b"x"),Err(Error::Denied));
        assert_eq!(fresh.pending(),(0,0));
    }
    #[test] fn wire_budget_accounts_headers_and_empty_datagrams(){
        let mut c=Channel::new(6,1,1,A,B,0,100,Budget{packets:100,bytes:60},
            Budget{packets:1,bytes:60}).unwrap();
        c.enqueue(6,1,0,b"").unwrap();
        assert_eq!(c.enqueue(6,1,0,b""),Err(Error::Budget));
        let(f,n)=inbound(b"");
        c.ingress(1,0,&f[..n]).unwrap();
        assert_eq!(c.ingress(1,0,&f[..n]),Err(Error::Budget));
        assert_eq!(c.budgets(),(Budget{packets:99,bytes:0},Budget{packets:0,bytes:0}));
    }
    #[test] fn ingress_budget_failure_is_sticky_even_for_smaller_later_frames(){
        let mut c=Channel::new(6,1,1,A,B,0,100,Budget{packets:1,bytes:60},
            Budget{packets:5,bytes:60}).unwrap();
        let (large,n)=inbound(&[0;100]);
        assert_eq!(c.ingress(1,0,&large[..n]),Err(Error::Budget));
        let(small,n)=inbound(b"");
        assert_eq!(c.ingress(1,0,&small[..n]),Err(Error::Closed));
        assert_eq!(c.pending(),(0,0));
    }
    #[test] fn all_invalid_construction_boundaries_refuse(){
        let good=Budget{packets:1,bytes:60};
        for (principal,incarnation,interface,local,peer,now,expiry,tx,rx) in [
            (0,1,1,A,B,0,10,good,good),(1,0,1,A,B,0,10,good,good),
            (1,1,0,A,B,0,10,good,good),(1,1,1,A,B,10,10,good,good),
            (1,1,1,A,B,11,10,good,good),(1,1,1,A,A,0,10,good,good),
            (1,1,1,A,Endpoint{mac:A.mac,..B},0,10,good,good),
            (1,1,1,A,Endpoint{ip:A.ip,..B},0,10,good,good),
            (1,1,1,Endpoint{port:0,..A},B,0,10,good,good),
            (1,1,1,A,Endpoint{port:0,..B},0,10,good,good),
            (1,1,1,A,B,0,10,Budget{packets:0,..good},good),
            (1,1,1,A,B,0,10,good,Budget{packets:0,..good}),
            (1,1,1,A,B,0,10,Budget{bytes:59,..good},good),
            (1,1,1,A,B,0,10,good,Budget{bytes:59,..good})
        ]{
            assert!(Channel::new(principal,incarnation,interface,local,peer,now,expiry,tx,rx).is_err());
        }
    }
    #[test] fn invalid_configuration_and_oversize_have_no_partial_effect(){
        for interface in [0]{
            assert!(Channel::new(1,1,interface,A,B,0,10,
                Budget{packets:1,bytes:60},Budget{packets:1,bytes:60}).is_err());
        }
        let mut c=channel();let before=c.budgets();
        assert_eq!(c.enqueue(6,1<<40,10,&[0;513]),Err(Error::Invalid));
        assert_eq!(c.budgets(),before);assert_eq!(c.pending(),(0,0));
    }
}
