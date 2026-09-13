//! Cloud host-only cross-language codec conformance; never a guest entry.
#[allow(dead_code)]
#[path="../../../services/expansion/network.rs"] mod network;
use std::io::{self, Read};
fn main() {
    let mut raw=Vec::new();
    io::stdin().take(300001).read_to_end(&mut raw).unwrap();
    assert!(raw.len()<300000);
    assert_eq!(&raw[..8],b"RARENET0");
    assert_eq!(u16::from_be_bytes([raw[8],raw[9]]),513);
    let a=network::Endpoint{mac:[2,0,0,0,0,1],ip:[10,42,0,1],port:4000};
    let b=network::Endpoint{mac:[2,0,0,0,0,2],ip:[10,42,0,2],port:4001};
    let mut at=10;
    for size in 0..=512usize {
        assert!(at+4<=raw.len());
        let expected_size=u16::from_be_bytes([raw[at],raw[at+1]]) as usize;
        let length=u16::from_be_bytes([raw[at+2],raw[at+3]]) as usize;
        assert_eq!(expected_size,size);at+=4;
        assert!(length<=network::MAX_FRAME && at+length<=raw.len());
        let expected=&raw[at..at+length];at+=length;
        let body:Vec<u8>=(0..size).map(|n|(n*37) as u8).collect();
        let mut encoded=[0;network::MAX_FRAME];
        let n=network::encode(a,b,0x1234,&body,&mut encoded).unwrap();
        assert_eq!(&encoded[..n],expected,"wire size {size}");
        assert_eq!(network::decode(expected,b,a),Ok(body.as_slice()));
    }
    assert_eq!(at,raw.len());
    println!("Expansion network: 513 independent cross-language wire cases passed; no NIC or guest execution");
}
