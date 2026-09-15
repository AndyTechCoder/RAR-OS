//! RAR-owned static-peer UDP/IPv4 codec. No device or ambient network authority.
//! See docs/interfaces/expansion-network-v0.md. This is not peer authentication.
#![forbid(unsafe_code)]

pub const MAX_PAYLOAD: usize = 512;
pub const MAX_FRAME: usize = 14 + 20 + 8 + MAX_PAYLOAD;
pub const MIN_FRAME: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Denied, Expired, Exhausted }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint { pub mac: [u8; 6], pub ip: [u8; 4], pub port: u16 }
impl Endpoint {
    pub fn valid(&self) -> bool {
        self.mac != [0; 6] && self.mac[0] & 1 == 0 &&
        (1..224).contains(&self.ip[0]) && self.ip[0] != 127 &&
        !(self.ip[0] == 169 && self.ip[1] == 254) && self.port != 0
    }
}

fn word(bytes: &[u8], at: usize) -> u16 {
    u16::from_be_bytes([bytes[at], bytes[at + 1]])
}
fn sum(bytes: &[u8]) -> u32 {
    let mut value = 0u32;
    for pair in bytes.chunks(2) {
        value += (u32::from(pair[0]) << 8) |
            u32::from(if pair.len() == 2 { pair[1] } else { 0 });
    }
    value
}
fn complement(mut value: u32) -> u16 {
    while value >> 16 != 0 { value = (value & 0xffff) + (value >> 16); }
    !(value as u16)
}
fn udp_checksum(source: [u8; 4], destination: [u8; 4], udp: &[u8]) -> u16 {
    complement(sum(&source) + sum(&destination) + 17 + udp.len() as u32 + sum(udp))
}
fn put(bytes: &mut [u8], at: usize, value: u16) {
    bytes[at..at + 2].copy_from_slice(&value.to_be_bytes());
}

/// Encode one bounded static-peer datagram. Invalid inputs never mutate output.
pub fn encode(source: Endpoint, destination: Endpoint, id: u16,
              payload: &[u8], output: &mut [u8]) -> Result<usize, Error> {
    if !source.valid() || !destination.valid() || payload.len() > MAX_PAYLOAD {
        return Err(Error::Invalid);
    }
    let packet_bytes = 42 + payload.len();
    let frame_bytes = packet_bytes.max(MIN_FRAME);
    if output.len() < frame_bytes { return Err(Error::Invalid); }
    let frame = &mut output[..frame_bytes];
    frame.fill(0);
    frame[..6].copy_from_slice(&destination.mac);
    frame[6..12].copy_from_slice(&source.mac);
    put(frame, 12, 0x0800);
    frame[14] = 0x45;
    put(frame, 16, (28 + payload.len()) as u16);
    put(frame, 18, id);
    put(frame, 20, 0x4000); // Don't fragment.
    frame[22] = 64;
    frame[23] = 17;
    frame[26..30].copy_from_slice(&source.ip);
    frame[30..34].copy_from_slice(&destination.ip);
    let ip_check = complement(sum(&frame[14..34]));
    put(frame, 24, ip_check);
    put(frame, 34, source.port);
    put(frame, 36, destination.port);
    put(frame, 38, (8 + payload.len()) as u16);
    frame[42..packet_bytes].copy_from_slice(payload);
    let udp_check = udp_checksum(source.ip, destination.ip, &frame[34..packet_bytes]);
    put(frame, 40, if udp_check == 0 { 0xffff } else { udp_check });
    Ok(frame_bytes)
}

/// Receive from exactly the configured peer; returned bytes borrow the input.
pub fn decode<'a>(frame: &'a [u8], local: Endpoint, peer: Endpoint)
                  -> Result<&'a [u8], Error> {
    if !local.valid() || !peer.valid() ||
        !(MIN_FRAME..=MAX_FRAME).contains(&frame.len()) { return Err(Error::Invalid); }
    if frame[..6] != local.mac || frame[6..12] != peer.mac ||
        word(frame, 12) != 0x0800 { return Err(Error::Denied); }
    let ip = &frame[14..];
    if ip[0] != 0x45 || ip[8] == 0 || ip[9] != 17 ||
        word(ip, 6) & !0x4000 != 0 || complement(sum(&ip[..20])) != 0 {
        return Err(Error::Invalid);
    }
    let total = usize::from(word(ip, 2));
    if !(28..=28 + MAX_PAYLOAD).contains(&total) || total > ip.len() {
        return Err(Error::Invalid);
    }
    if ip[12..16] != peer.ip || ip[16..20] != local.ip { return Err(Error::Denied); }
    let udp = &ip[20..total];
    if usize::from(word(udp, 4)) != udp.len() || word(udp, 6) == 0 ||
        udp_checksum(peer.ip, local.ip, udp) != 0 { return Err(Error::Invalid); }
    if word(udp, 0) != peer.port || word(udp, 2) != local.port { return Err(Error::Denied); }
    Ok(&udp[8..])
}

/// Internal broker accounting, not a user-forgeable token or kernel capability.
/// Only trusted policy installs this object; caller identity MUST be taken from
/// authenticated kernel envelope metadata, never from application payload bytes.
pub struct Grant {
    principal: u64, incarnation: u64, peer: Endpoint, expires: u64,
    last_tick: u64, packets: u32, bytes: u32, revoked: bool,
}
impl Grant {
    pub fn new(principal: u64, incarnation: u64, peer: Endpoint, now: u64,
               expires: u64, packets: u32, bytes: u32) -> Result<Self, Error> {
        if principal == 0 || incarnation == 0 || !peer.valid() ||
            expires <= now || packets == 0 { return Err(Error::Invalid); }
        Ok(Self { principal, incarnation, peer, expires, last_tick: now,
            packets, bytes, revoked: false })
    }
    pub fn revoke(&mut self) { self.revoked = true; }
    pub fn remaining(&self) -> (u32, u32) { (self.packets, self.bytes) }
    /// Reserve before I/O; never refund transport failures.
    pub fn reserve(&mut self, principal: u64, incarnation: u64, peer: Endpoint,
                   now: u64, payload_bytes: usize) -> Result<(), Error> {
        if self.revoked || principal != self.principal || incarnation != self.incarnation ||
            peer != self.peer { return Err(Error::Denied); }
        if now < self.last_tick || now >= self.expires {
            // Retain even an expired observation: a later stale clock sample
            // must never resurrect a grant after expiration was observed.
            self.last_tick = self.last_tick.max(now);
            return Err(Error::Expired);
        }
        self.last_tick = now;
        if payload_bytes > MAX_PAYLOAD { return Err(Error::Invalid); }
        if self.packets == 0 || payload_bytes > self.bytes as usize {
            return Err(Error::Exhausted);
        }
        self.packets -= 1;
        self.bytes -= payload_bytes as u32;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const A: Endpoint = Endpoint { mac: [2,0,0,0,0,1], ip: [10,42,0,1], port: 4000 };
    const B: Endpoint = Endpoint { mac: [2,0,0,0,0,2], ip: [10,42,0,2], port: 4001 };
    fn frame(payload: &[u8]) -> ([u8; MAX_FRAME], usize) {
        let mut bytes = [0; MAX_FRAME];
        let n = encode(A, B, 0x1234, payload, &mut bytes).unwrap();
        (bytes, n)
    }
    // Independent bit-at-a-time checksum oracle, not the production helper.
    fn oracle(bytes: &[u8]) -> u16 {
        let mut carry = 0u32;
        for bit in (0..bytes.len() * 8).step_by(16) {
            let mut w = 0u32;
            for i in 0..16 {
                w <<= 1;
                let index = bit + i;
                if index < bytes.len() * 8 { w |= u32::from((bytes[index / 8] >> (7 - index % 8)) & 1); }
            }
            carry += w;
            carry = (carry & 65535) + (carry >> 16);
        }
        !(carry as u16)
    }
    fn ip_fix(f: &mut [u8]) {
        f[24] = 0; f[25] = 0;
        let c = oracle(&f[14..34]); put(f,24,c);
    }
    #[test] fn all_payload_lengths_round_trip_and_independent_checksums() {
        let mut body = [0; MAX_PAYLOAD];
        for (i,b) in body.iter_mut().enumerate() { *b = (i.wrapping_mul(37)) as u8; }
        for size in 0..=MAX_PAYLOAD {
            let (f,n) = frame(&body[..size]);
            assert_eq!(decode(&f[..n],B,A),Ok(&body[..size]));
            assert_eq!(oracle(&f[14..34]),0);
            let mut pseudo = [0; 12 + 8 + MAX_PAYLOAD];
            pseudo[..4].copy_from_slice(&A.ip);
            pseudo[4..8].copy_from_slice(&B.ip);
            pseudo[9] = 17;
            put(&mut pseudo,10,(8+size) as u16);
            pseudo[12..20+size].copy_from_slice(&f[34..42+size]);
            assert_eq!(oracle(&pseudo[..20+size]),0);
            assert!(f[42+size..n].iter().all(|&x|x==0));
        }
    }
    #[test] fn fixed_network_order_known_answer_and_padding_bound() {
        // Manually summed network-order words: IP=1444, UDP=47fe.
        let golden: &[u8] = &[0x02,0x00,0x00,0x00,0x00,0x02,0x02,0x00,0x00,0x00,0x00,0x01,0x08,0x00,0x45,0x00,0x00,0x1f,0x12,0x34,0x40,0x00,0x40,0x11,0x14,0x44,0x0a,0x2a,0x00,0x01,0x0a,0x2a,0x00,0x02,0x0f,0xa0,0x0f,0xa1,0x00,0x0b,0x47,0xfe,0x41,0x42,0x43];
        let (f,n)=frame(b"ABC");
        assert_eq!(&f[..golden.len()],golden);
        assert_eq!(n,60);
        assert_eq!(decode(&f[..n],B,A),Ok(&b"ABC"[..]));
        // Maximum bounded Ethernet tail is ignored, never surfaced as payload.
        let mut padded=f;padded[n..].fill(0xa5);
        assert_eq!(decode(&padded,B,A),Ok(&b"ABC"[..]));
        assert_eq!(complement(sum(&[0,1,0xf2,3,0xf4,0xf5,0xf6,0xf7])),0x220d);
    }
    #[test] fn refusal_is_non_mutating() {
        let mut output = [0xa5; MAX_FRAME];
        assert_eq!(encode(A,B,0,&[0;513],&mut output),Err(Error::Invalid));
        for size in 0..MIN_FRAME {
            assert_eq!(encode(A,B,0,b"x",&mut output[..size]),Err(Error::Invalid));
        }
        let mut bad=A;bad.port=0;
        assert_eq!(encode(bad,B,0,b"x",&mut output),Err(Error::Invalid));
        assert_eq!(output,[0xa5;MAX_FRAME]);
    }
    #[test] fn all_truncations_and_oversize_refused() {
        let (f,n)=frame(&[7;512]);
        for size in 0..n { assert!(decode(&f[..size],B,A).is_err()); }
        assert_eq!(decode(&[0;MAX_FRAME+1],B,A),Err(Error::Invalid));
    }
    #[test] fn every_wire_bit_before_padding_is_checked() {
        let (f,n)=frame(b"payload");
        // ID, DSCP/ECN, DF and nonzero TTL may legitimately vary with a corrected
        // IP checksum, but single corruptions without recomputing it must fail.
        for i in 0..49 { for bit in 0..8 {
            let mut bad=f;bad[i]^=1<<bit;
            assert!(decode(&bad[..n],B,A).is_err(),"byte {i} bit {bit}");
        }}
        let mut padding=f;padding[59]=0xaa;
        assert_eq!(decode(&padding[..n],B,A),Ok(&b"payload"[..]));
    }
    #[test] fn unsupported_protocol_shapes_rejected_after_valid_ip_checksum() {
        let (f,n)=frame(b"payload");
        for (at,value) in [(14,0x46),(14,0x65),(22,0),(23,6),(20,0x20),
                          (20,0x80),(21,1)] {
            let mut bad=f;bad[at]=value;ip_fix(&mut bad);
            assert!(decode(&bad[..n],B,A).is_err());
        }
        for total in [0,19,27,36,541,65535] {
            let mut bad=f;put(&mut bad,16,total);ip_fix(&mut bad);
            assert!(decode(&bad[..n],B,A).is_err());
        }
        let mut bad=f;put(&mut bad,40,0);
        assert_eq!(decode(&bad[..n],B,A),Err(Error::Invalid));
        for length in [0,7,14,16,65535] {
            let mut bad=f;put(&mut bad,38,length);
            assert!(decode(&bad[..n],B,A).is_err());
        }
        let mut allowed=f;put(&mut allowed,20,0);ip_fix(&mut allowed);
        assert!(decode(&allowed[..n],B,A).is_ok());
    }
    #[test] fn checksum_zero_is_transmitted_as_ffff() {
        let mut found=false;
        for number in 0..=u16::MAX {
            let (f,n)=frame(&number.to_be_bytes());
            if word(&f,40)==0xffff {
                assert!(decode(&f[..n],B,A).is_ok());found=true;break;
            }
        }
        assert!(found);
    }
    #[test] fn invalid_endpoint_classes_and_wrong_peer_denied() {
        for ip in [[0,0,0,0],[127,0,0,1],[224,0,0,1],[255,255,255,255],[169,254,1,1]] {
            assert!(!Endpoint{ip,..A}.valid());
        }
        assert!(!Endpoint{mac:[0;6],..A}.valid());
        assert!(!Endpoint{mac:[1,0,0,0,0,1],..A}.valid());
        let (f,n)=frame(b"x");
        for peer in [Endpoint{port:4002,..A},Endpoint{ip:[10,42,0,3],..A},
                     Endpoint{mac:[2,0,0,0,0,3],..A}] {
            assert_eq!(decode(&f[..n],B,peer),Err(Error::Denied));
        }
    }
    #[test] fn grants_are_scoped_revocable_incarnation_and_time_bounded() {
        let mut g=Grant::new(5,1<<40,B,10,20,3,100).unwrap();
        for (p,i,e) in [(6,1<<40,B),(5,0,B),(5,1,B),(5,1<<40,A)] {
            assert_eq!(g.reserve(p,i,e,11,1),Err(Error::Denied));
            assert_eq!(g.remaining(),(3,100));
        }
        assert_eq!(g.reserve(5,1<<40,B,9,1),Err(Error::Expired));
        assert_eq!(g.reserve(5,1<<40,B,11,513),Err(Error::Invalid));
        assert_eq!(g.reserve(5,1<<40,B,11,101),Err(Error::Exhausted));
        assert_eq!(g.reserve(5,1<<40,B,12,40),Ok(()));
        assert_eq!(g.remaining(),(2,60));
        assert_eq!(g.reserve(5,1<<40,B,11,1),Err(Error::Expired));
        assert_eq!(g.reserve(5,1<<40,B,20,1),Err(Error::Expired));
        assert_eq!(g.reserve(5,1<<40,B,13,1),Err(Error::Expired));
        assert_eq!(g.remaining(),(2,60));
        g.revoke();
        assert_eq!(g.reserve(5,1<<40,B,13,1),Err(Error::Denied));
    }
    #[test] fn empty_packets_consume_budget_and_failures_cannot_refund() {
        let mut g=Grant::new(1,1,A,0,u64::MAX,2,0).unwrap();
        assert_eq!(g.reserve(1,1,A,0,0),Ok(()));
        assert_eq!(g.reserve(1,1,A,0,0),Ok(()));
        assert_eq!(g.reserve(1,1,A,0,0),Err(Error::Exhausted));
        assert_eq!(g.remaining(),(0,0));
        assert!(Grant::new(0,1,A,0,1,1,1).is_err());
        assert!(Grant::new(1,0,A,0,1,1,1).is_err());
        assert!(Grant::new(1,1,A,1,1,1,1).is_err());
        assert!(Grant::new(1,1,A,0,1,0,1).is_err());
    }
    #[test] fn deterministic_malformed_corpus_never_panics() {
        let mut state=0x12345678u32;let mut f=[0;MAX_FRAME];
        for attempt in 0..4096usize {
            for b in &mut f { state^=state<<13;state^=state>>17;state^=state<<5;*b=state as u8; }
            let n=attempt%(MAX_FRAME+1);
            assert!(decode(&f[..n],B,A).is_err());
        }
    }
}
