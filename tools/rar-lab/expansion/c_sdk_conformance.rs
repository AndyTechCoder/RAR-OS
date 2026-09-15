//! Host-only C/Rust SDK request conformance consumer, not a guest.
#[path="../../../sdk/expansion/rust/wire.rs"] mod sdk;
use std::io::Read;
fn main(){
    let mut bytes=Vec::new();std::io::stdin().take(44506).read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes.len(),345*129);
    let mut at=0;
    for op in 1..=3u8{
        for n in 0..=if op==1{112}else{0}{
            for id in [1,1u64<<40,u64::MAX]{
                let mut payload=[0u8;112];for(i,b)in payload.iter_mut().enumerate(){*b=i as u8;}
                let mut out=[0xaa;128];let size=sdk::encode(op,id,&payload[..n],&mut out).unwrap();
                assert_eq!(bytes[at]as usize,size);assert_eq!(&bytes[at+1..at+129],&out);
                at+=129;
            }
        }
    }
    assert_eq!(at,bytes.len());println!("Expansion SDK: 345 exact C/Rust wire records agree");
}
