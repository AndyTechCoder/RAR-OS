//! Host-only exact public package verification, never PE/OS execution.
#![forbid(unsafe_code)]
#[path="../../../core/crypto/sha256.rs"] mod sha256;
#[path="../../../core/crypto/sha512.rs"] mod sha512;
#[path="../../../core/crypto/ed25519.rs"] mod ed25519;
#[path="../../../core/modern/manifest.rs"] mod manifest;
#[path="../../../nucleus/platform/pe.rs"] mod pe;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub enum Error{Invalid,Denied}
fn valid(input:&[u8])->bool{
    if !(928..=2_097_568).contains(&input.len()){return false;}
    let public:[u8;32]=input[..32].try_into().unwrap();
    if public==manifest::LAB_PUBLIC_KEY{return false;}
    let raw=&input[32..416];let payload=&input[416..];
    let Ok(parsed)=manifest::Manifest::parse(raw) else{return false;};
    let Ok(message)=parsed.signed_message() else{return false;};
    let signature:[u8;64]=raw[320..384].try_into().unwrap();
    parsed.generation()==2&&ed25519::verify(&public,&message,&signature)&&
        raw[144..176]==sha256::sha256(&public).unwrap()&&
        raw[288..320]==sha256::sha256(&raw[..288]).unwrap()&&
        raw[112..144]==sha256::sha256(payload).unwrap()&&
        u32::from_le_bytes(raw[56..60].try_into().unwrap()) as usize==payload.len()&&
        matches!(manifest::verify(raw,payload,1),Err(manifest::Reject::Publisher))
}
fn main(){
    use std::io::Read;
    let mut input=Vec::new();std::io::stdin().take(2_097_569).read_to_end(&mut input).unwrap();
    assert!(valid(&input),"exact package must have valid unknown-key signature and Publisher refusal");
    let mut bad=input.clone();bad[32+320]^=1;assert!(!valid(&bad));
    bad.copy_from_slice(&input);bad[..32].copy_from_slice(&manifest::LAB_PUBLIC_KEY);assert!(!valid(&bad));
    bad.copy_from_slice(&input);let last=bad.len()-1;bad[last]^=1;assert!(!valid(&bad));
    assert!(!valid(&input[..927]));
    println!("RAR-UNKNOWN-PUBLISHER:EXACT-PACKAGE-VERIFIED");
}
