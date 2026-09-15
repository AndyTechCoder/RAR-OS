//! Independent host-produced signed bytes checked by actual RAR verifier.
//! Never loads or executes PE contents; stdin is strictly bounded.
#[path="../../../core/expansion/lib.rs"] mod app;
pub use app::{sha256,sha512,ed25519,pe,Error};
use app::app_manifest::{verify,Error as E};
use std::io::{self,Read};
fn main(){
    const LENGTH:usize=16+6*1568;
    let mut bytes=Vec::new();
    io::stdin().take((LENGTH+1)as u64).read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes.len(),LENGTH);
    assert_eq!(&bytes[..8],b"RARAPPFX");
    assert_eq!(&bytes[8..16],&6u64.to_le_bytes());
    let mut negatives=0;
    for case in 0..6{
        let record=&bytes[16+case*1568..16+(case+1)*1568];
        let owner=&record[..32];let manifest=&record[32..544];let payload=&record[544..];
        if case>=2{
            let expected=[E::Compatibility,E::Executable,E::Budget,E::Publisher][case-2];
            assert!(matches!(verify(manifest,payload,1,15),Err(e) if e==expected));
            negatives+=1;continue;
        }
        let verified=verify(manifest,payload,1,15).unwrap();
        assert_eq!(verified.document_owner(),owner);
        assert_eq!(verified.manifest().generation(),case as u64+1);
        assert_eq!(verified.manifest().rights(),3);
        assert_eq!(verified.layout().entry,0x401000);
        assert_eq!(verified.payload(),payload);
        assert!(matches!(verify(manifest,payload,3,15),Err(E::Rollback)));
        assert!(matches!(verify(manifest,payload,1,1),Err(E::Budget)));
        negatives+=2;
        for at in 448..512{
            let mut bad=manifest.to_vec();bad[at]^=1;
            assert!(matches!(verify(&bad,payload,1,15),Err(E::Signature)));negatives+=1;
        }
        // Start from a VERIFIED signed fixture, not an already invalid signature.
        for at in 0..448{
            let mut bad=manifest.to_vec();bad[at]^=1;
            assert!(verify(&bad,payload,1,15).is_err());negatives+=1;
        }
        // Recomputing the public digest does not re-sign changed authority.
        for at in 16..128{
            let mut bad=manifest.to_vec();bad[at]^=1;
            let digest=sha256::sha256(&bad[..416]).unwrap();
            bad[416..448].copy_from_slice(&digest);
            assert!(verify(&bad,payload,1,15).is_err());negatives+=1;
        }
        assert!(verify(&manifest[..511],payload,1,15).is_err());
        let mut extra=manifest.to_vec();extra.push(0);
        assert!(verify(&extra,payload,1,15).is_err());
        assert!(verify(manifest,&payload[..payload.len()-1],1,15).is_err());
        let mut extra=payload.to_vec();extra.push(0);
        assert!(verify(manifest,&extra,1,15).is_err());negatives+=4;
        // Hash failure follows valid signature, so every payload byte is checked.
        for at in 0..payload.len(){
            let mut bad=payload.to_vec();bad[at]^=1;
            assert!(matches!(verify(manifest,&bad,1,15),Err(E::Digest)));negatives+=1;
        }
    }
    println!("Expansion app package: two signed positives and {negatives} exact refusals; no execution");
}
