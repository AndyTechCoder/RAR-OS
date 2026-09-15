//! Host-only Rust/C ABI conformance; no native calls or app execution.
#[path="../../../sdk/alpha/rust/lib.rs"] mod sdk;
use std::io::{self,Read};
fn main(){
    const SIZE:usize=6*113*128+513*259;
    let mut bytes=Vec::new();io::stdin().take((SIZE+1)as u64).read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes.len(),SIZE);let payload:Vec<u8>=(0..112).collect();let mut at=0;
    for op in 1..=6{for len in 0..=112{
        let expected=sdk::Message::new(op,0,u32::MAX,&payload[..len]).unwrap();
        assert_eq!(&bytes[at..at+128],expected.encode());
        assert_eq!(sdk::Message::decode(&bytes[at..at+128]),Ok(expected));at+=128;
    }}
    for _ in 0..513{
        let len=u16::from_le_bytes(bytes[at..at+2].try_into().unwrap())as usize;
        assert!(len<=256);assert!(bytes[at+2]<=1);
        let result=sdk::Boot::decode(&bytes[at+3..at+3+len]);
        assert_eq!(result.is_ok(),bytes[at+2]==1);at+=259;
    }
    assert_eq!(at,SIZE);
    println!("Expansion app SDK: 678 canonical messages and 513 bootstrap decisions agree in C/Rust");
}
