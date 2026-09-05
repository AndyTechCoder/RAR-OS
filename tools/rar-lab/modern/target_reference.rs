//! Host-only adapter around the exact RAR crypto modules, not an OS entrypoint.
//! Run only in the separate reference-free cloud compiler/test role.
#![forbid(unsafe_code)]
#[path="../../../core/crypto/sha256.rs"] mod sha256;
#[path="../../../core/crypto/sha512.rs"] mod sha512;
#[path="../../../core/crypto/ed25519.rs"] mod ed25519;
#[path="../../../core/crypto/chacha20poly1305.rs"] mod aead;
use std::io::{Read,Write};
const MAX:usize=4432;
fn operation(raw:&[u8])->Result<(u8,&[u8]),u8> {
    if !(16..=MAX).contains(&raw.len()) || &raw[..8]!=b"RARMCR00" || raw[9..12]!=[0;3] {return Err(64);}
    let n=u32::from_le_bytes(raw[12..16].try_into().map_err(|_|64u8)?)as usize;
    if n!=raw.len()-16{return Err(64);}
    let op=raw[8];let p=&raw[16..];
    match op {
        1|2 if n<=4096=>{},
        3 if (96..=4192).contains(&n)=>{},
        4|5=>{
            let base=if op==4{48}else{64};
            if n<base{return Err(64);}
            let an=u16::from_le_bytes([p[44],p[45]])as usize;
            let dn=u16::from_le_bytes([p[46],p[47]])as usize;
            if an>256||dn>4096||n!=base+an+dn{return Err(64);}
        },
        _=>return Err(64),
    }
    Ok((op,p))
}
fn answer(raw:&[u8])->Result<Vec<u8>,u8> {
    let (op,p)=operation(raw)?;let mut status=0u8;
    let value=match op {
        1=>sha256::sha256(p).map_err(|_|70u8)?.to_vec(),
        2=>sha512::digest(p).to_vec(),
        3=>{
            let key=p[..32].try_into().map_err(|_|64u8)?;
            let signature=p[32..96].try_into().map_err(|_|64u8)?;
            if !ed25519::verify(key,&p[96..],signature){status=1;}Vec::new()
        },
        4|5=>{
            let key=p[..32].try_into().map_err(|_|64u8)?;
            let nonce=p[32..44].try_into().map_err(|_|64u8)?;
            let an=u16::from_le_bytes([p[44],p[45]])as usize;
            let base=if op==4{48}else{64};
            let associated=&p[base..base+an];let mut data=p[base+an..].to_vec();
            if op==4 {
                let tag=aead::seal(key,nonce,associated,&mut data).map_err(|_|70u8)?;
                data.extend_from_slice(&tag);data
            } else {
                let tag=p[48..64].try_into().map_err(|_|64u8)?;
                match aead::open(key,nonce,associated,&mut data,tag) {
                    Ok(())=>data,
                    Err(aead::Error::Authentication)=>{status=1;Vec::new()},
                    Err(aead::Error::Bounds)=>return Err(70),
                }
            }
        },
        _=>return Err(64),
    };
    let mut out=vec![0u8;64];
    out[..8].copy_from_slice(b"RARMCO00");out[8]=op;out[9]=status;out[10]=3;
    out[16..20].copy_from_slice(&(value.len()as u32).to_le_bytes());
    out[24..56].copy_from_slice(&sha256::sha256(raw).map_err(|_|70u8)?);
    out.extend_from_slice(&value);Ok(out)
}
fn main() {
    if std::env::args_os().count()!=1{std::process::exit(64);}
    let mut raw=Vec::with_capacity(MAX+1);
    if std::io::stdin().lock().take((MAX+1)as u64).read_to_end(&mut raw).is_err(){std::process::exit(74);}
    let result=match answer(&raw){Ok(value)=>value,Err(code)=>std::process::exit(code as i32)};
    let mut stdout=std::io::stdout().lock();
    if stdout.write_all(&result).is_err()||stdout.flush().is_err(){std::process::exit(74);}
}
#[cfg(test)]
mod adapter_tests {
    use super::*;
    fn request(op:u8,p:&[u8])->Vec<u8>{
        let mut r=b"RARMCR00".to_vec();r.extend_from_slice(&[op,0,0,0]);
        r.extend_from_slice(&(p.len()as u32).to_le_bytes());r.extend_from_slice(p);r
    }
    #[test] fn exact_hash_result_and_binding() {
        for op in [1,2] {
            let req=request(op,b"abc");let out=answer(&req).unwrap();
            assert_eq!(&out[..8],b"RARMCO00");assert_eq!(&out[8..11],&[op,0,3]);
            assert_eq!(&out[24..56],&sha256::sha256(&req).unwrap());
            let want=if op==1{sha256::sha256(b"abc").unwrap().to_vec()}else{sha512::digest(b"abc").to_vec()};
            assert_eq!(&out[64..],&want);
        }
    }
    #[test] fn malformed_frames_never_run_operation() {
        let req=request(1,b"abc");
        for n in 0..req.len(){assert_eq!(answer(&req[..n]),Err(64));}
        for i in 0..16 {
            let mut bad=req.clone();bad[i]^=128;assert_eq!(answer(&bad),Err(64));
        }
        let mut extra=req;extra.push(0);assert_eq!(answer(&extra),Err(64));
        for op in [0,6,255]{assert_eq!(answer(&request(op,b"")),Err(64));}
        assert_eq!(answer(&request(1,&[0;4097])),Err(64));
        assert_eq!(answer(&request(3,&[0;95])),Err(64));
    }
    #[test] fn aead_roundtrip_and_invalid_tag_emits_no_plaintext() {
        let mut nonce_counter=0u64;
        for an in [0usize,1,255,256] {for dn in [0usize,1,4095,4096] {
            nonce_counter+=1;
            let mut p=vec![7u8;44];p[32..40].copy_from_slice(&nonce_counter.to_le_bytes());p.extend_from_slice(&(an as u16).to_le_bytes());
            p.extend_from_slice(&(dn as u16).to_le_bytes());p.extend_from_slice(&vec![9;an]);
            p.extend_from_slice(&vec![11;dn]);
            let sealed=answer(&request(4,&p)).unwrap();assert_eq!(sealed[9],0);
            let mut open=p[..48].to_vec();open.extend_from_slice(&sealed[64+dn..]);
            open.extend_from_slice(&p[48..48+an]);open.extend_from_slice(&sealed[64..64+dn]);
            let restored=answer(&request(5,&open)).unwrap();
            assert_eq!(restored[9],0);assert_eq!(&restored[64..],&vec![11;dn]);
            open[48]^=1;let bad=answer(&request(5,&open)).unwrap();
            assert_eq!(bad[9],1);assert_eq!(bad.len(),64);assert_eq!(&bad[16..20],&[0;4]);
        }}
    }
}
