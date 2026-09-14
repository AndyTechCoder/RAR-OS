//! Cloud host byte verification only. Never executes an ELF/PE input.
#[path="../../../core/expansion/lib.rs"] mod app;
pub use app::{sha256,sha512,ed25519,pe,Error};
use std::io::{self,Read};
fn main() {
    let args:Vec<String>=std::env::args().skip(1).collect();
    let packaged=args==["--package-counter"];
    assert!(args.is_empty()||packaged);
    let mut bytes=Vec::new();
    io::stdin().take((pe::LIMIT+513) as u64).read_to_end(&mut bytes).unwrap();
    let payload=if packaged {
        assert!((1024..=pe::LIMIT+512).contains(&bytes.len()));
        let (manifest,payload)=bytes.split_at(512);
        let verified=app::app_manifest::verify(manifest,payload,1,1).expect("actual RAR app signature verifier");
        assert_eq!(verified.manifest().application_id(),*b"rar.counter.v000");
        assert_eq!(verified.manifest().generation(),1);
        assert_eq!(verified.manifest().rights(),1);
        assert_eq!(verified.manifest().stack_bytes(),65536);
        assert_eq!(verified.layout().image_size,verified.manifest().image_budget());
        assert!(app::app_manifest::verify(manifest,payload,2,1).is_err());
        assert!(app::app_manifest::verify(manifest,payload,1,0).is_err());
        let mut bad=payload.to_vec();bad[0]^=1;
        assert!(app::app_manifest::verify(manifest,&bad,1,1).is_err());
        let mut bad=manifest.to_vec();bad[448]^=1;
        assert!(app::app_manifest::verify(&bad,payload,1,1).is_err());
        println!("Native Counter: exact public-lab signature/identity/UI-only rights verified; rollback/tamper refused");
        payload
    } else {bytes.as_slice()};
    assert!((512..=pe::LIMIT).contains(&payload.len()));
    let layout=pe::parse(payload).expect("actual kernel PE parser");
    assert!(layout.entry>=pe::BASE+4096 && layout.entry<pe::BASE+pe::LIMIT as u64);
    assert!(layout.image_size<=pe::LIMIT && layout.count<=3);
    assert!(layout.header_size>=512);
    assert!(layout.sections[..layout.count].iter().any(|s|s.executable));
    for section in &layout.sections[..layout.count] {
        assert!(!(section.writable && section.executable));
        assert!(section.file_offset+section.file_size<=payload.len());
        assert!(section.virtual_offset+section.memory_size<=layout.image_size);
    }
    println!("Native C PE bytes accepted by unchanged kernel parser: {} bytes, {} sections",payload.len(),layout.count);
}
