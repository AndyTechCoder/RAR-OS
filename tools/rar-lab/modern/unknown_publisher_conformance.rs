//! Host-only public fixture verification, never a PE/OS execution.
#![forbid(unsafe_code)]
#[path="../../../core/crypto/sha256.rs"] mod sha256;
#[path="../../../core/crypto/sha512.rs"] mod sha512;
#[path="../../../core/crypto/ed25519.rs"] mod ed25519;
#[path="../../../core/modern/manifest.rs"] mod manifest;
#[path="../../../nucleus/platform/pe.rs"] mod pe;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Invalid,Denied}
fn main(){
    use std::io::Read;
    let mut input=Vec::new();std::io::stdin().take(1441).read_to_end(&mut input).unwrap();
    assert_eq!(input.len(),1440);
    let public:[u8;32]=input[..32].try_into().unwrap();
    let raw=&input[32..416];let payload=&input[416..];
    let parsed=manifest::Manifest::parse(raw).unwrap();
    let message=parsed.signed_message().unwrap();
    let signature:[u8;64]=raw[320..384].try_into().unwrap();
    assert!(ed25519::verify(&public,&message,&signature),"negative signature must be valid under its own key");
    assert_eq!(&raw[144..176],&sha256::sha256(&public).unwrap());
    assert_eq!(parsed.generation(),2);
    assert!(matches!(manifest::verify(raw,payload,1),Err(manifest::Reject::Publisher)));
    println!("Unknown publisher: actual RAR signature verification succeeds with the non-enrolled key; layer policy rejects Publisher.");
}
