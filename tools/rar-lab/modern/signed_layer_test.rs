//! Host-only codec conformance tests; fixture PE bytes are never executed.
#![forbid(unsafe_code)]
#[path = "../../../core/crypto/sha256.rs"] mod sha256;
#[path = "../../../core/crypto/sha512.rs"] mod sha512;
#[path = "../../../core/crypto/ed25519.rs"] mod ed25519;
#[path = "../../../core/modern/manifest.rs"] mod manifest;
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Error { Invalid, Denied }
#[path = "../../../nucleus/platform/pe.rs"] mod pe;

const FIXTURE: &[u8; 4 * (384 + 1024)] =
    include_bytes!(env!("RAR_LAB_SIGNED_CODEC_FIXTURE"));
fn case(index: usize) -> (&'static [u8], &'static [u8]) {
    let start = index * (384 + 1024);
    (&FIXTURE[start..start+384], &FIXTURE[start+384..start+384+1024])
}
#[test]
fn separately_generated_signed_package_reaches_verified_layer() {
    let (raw, payload) = case(0);
    let verified = manifest::verify(raw, payload, 1).unwrap();
    assert_eq!(verified.manifest().generation(), 1);
    assert_eq!(verified.payload(), payload);
    assert_eq!(verified.layout().entry, 0x401000);
    assert_eq!(verified.layout().image_size, 8192);
    assert_eq!(verified.manifest().payload_digest(), sha256::sha256(payload).unwrap());
}
#[test]
fn authenticated_compatibility_budget_and_executable_rules_still_apply() {
    let (raw, payload) = case(0);
    assert!(matches!(manifest::verify(raw, payload, 2), Err(manifest::Reject::Rollback)));
    let (raw, payload) = case(1);
    assert!(matches!(manifest::verify(raw, payload, 1), Err(manifest::Reject::Compatibility)));
    let (raw, payload) = case(2);
    assert_eq!(manifest::verify(raw, payload, 2).unwrap().manifest().generation(), 2);
    let (raw, payload) = case(3);
    assert!(matches!(manifest::verify(raw, payload, 1), Err(manifest::Reject::Executable)));
}
#[test]
fn manifest_bytes_and_representative_payload_tampering_do_not_authenticate() {
    let (raw, payload) = case(0);
    for offset in 0..384 {
        let mut changed = raw.to_vec();
        changed[offset] ^= 1;
        assert!(manifest::verify(&changed, payload, 1).is_err(), "manifest byte {offset}");
    }
    for offset in [0,1,60,64,112,328,364,511,512,513,1023] {
        let mut changed = payload.to_vec();
        changed[offset] ^= 1;
        assert!(matches!(manifest::verify(raw, &changed, 1), Err(manifest::Reject::PayloadDigest)),
            "payload byte {offset}");
    }
}
