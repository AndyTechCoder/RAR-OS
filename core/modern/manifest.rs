//! Modern-v0 bounded signed layer, not stable RLM/RCI and not execution authority.
use crate::{ed25519, pe, sha256::sha256};

pub const SIZE: usize = 384;
pub const PREIMAGE: usize = 288;
pub const MAX_PAYLOAD: usize = 2 * 1024 * 1024;
pub const LAB_PUBLIC_KEY: [u8; 32] = [
0xd7,0x5a,0x98,0x01,0x82,0xb1,0x0a,0xb7,0xd5,0x4b,0xfe,0xd3,0xc9,0x64,0x07,0x3a,
0x0e,0xe1,0x72,0xf3,0xda,0xa6,0x23,0x25,0xaf,0x02,0x1a,0x68,0xf7,0x07,0x51,0x1a];
pub const HEALTH_NAME: &[u8] = b"RAR-MODERN-SETTINGS-HEALTH-V0\0";
const ALGORITHM: &[u8; 24] = b"rar.alpha.ed25519.v0\0\0\0\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject {
    Framing, Encoding, Algorithm, Publisher, Digest, Signature, Compatibility,
    Budget, Rollback, PayloadLength, PayloadDigest, Executable,
}
fn hash(b: &[u8]) -> Result<[u8; 32], Reject> {
    sha256(b).map_err(|_| Reject::Framing)
}
fn word32(b: &[u8; SIZE], p: usize) -> u32 {
    u32::from_le_bytes(b[p..p+4].try_into().unwrap())
}
fn word64(b: &[u8; SIZE], p: usize) -> u64 {
    u64::from_le_bytes(b[p..p+8].try_into().unwrap())
}

/// Parsed framing is untrusted. No setter or external constructor.
#[derive(Clone, Copy, Debug)]
pub struct Manifest<'a> { bytes: &'a [u8; SIZE] }
impl<'a> Manifest<'a> {
    pub fn parse(raw: &'a [u8]) -> Result<Self, Reject> {
        let b: &[u8; SIZE] = raw.try_into().map_err(|_| Reject::Framing)?;
        if &b[..8] != b"RARMODL0" || b[8..10] != [0,0] ||
            b[10..12] != (SIZE as u16).to_le_bytes() { return Err(Reject::Framing); }
        if word32(b, 12) != 1 || b[244..288].iter().any(|&x| x != 0) {
            return Err(Reject::Encoding);
        }
        if &b[16..40] != ALGORITHM { return Err(Reject::Algorithm); }
        Ok(Self { bytes: b })
    }
    pub fn generation(&self) -> u64 { word64(self.bytes, 72) }
    pub fn digest(&self) -> [u8; 32] { self.bytes[288..320].try_into().unwrap() }
    pub fn payload_digest(&self) -> [u8; 32] { self.bytes[112..144].try_into().unwrap() }
    pub fn signed_message(&self) -> Result<[u8; 51], Reject> {
        // ADR0019 excludes both trailing fields, never zero-fills them.
        let mut result = [0; 51];
        result[..19].copy_from_slice(b"RAR-LAYER-ALPHA-V0\0");
        result[19..].copy_from_slice(&hash(&self.bytes[..PREIMAGE])?);
        Ok(result)
    }
    fn authenticate(&self) -> Result<(), Reject> {
        let b = self.bytes;
        if b[144..176] != hash(&LAB_PUBLIC_KEY)? { return Err(Reject::Publisher); }
        if self.digest() == [0;32] || self.digest() != hash(&b[..PREIMAGE])? { return Err(Reject::Digest); }
        let signature = b[320..384].try_into().unwrap();
        if !ed25519::verify(&LAB_PUBLIC_KEY, &self.signed_message()?, &signature) {
            return Err(Reject::Signature);
        }
        Ok(())
    }
    fn policy(&self, minimum_generation: u64) -> Result<(), Reject> {
        let b = self.bytes;
        if word32(b, 40) != 5 || b[44..46] != [1,0] || b[46..48] != [0,0] ||
            word32(b, 48) != 1 || word32(b, 52) != 0 || word32(b, 228) != 0 ||
            b[80..112] != hash(HEALTH_NAME)? || b[176..196].iter().all(|&x| x == 0) ||
            b[196..228].iter().all(|&x| x == 0) {
            return Err(Reject::Compatibility);
        }
        let memory = word32(b, 60) as usize;
        if memory == 0 || memory > pe::LIMIT || memory % 4096 != 0 ||
            word64(b, 64) != 7 || !(1..=100).contains(&word32(b, 232)) ||
            word32(b, 236) != 0 || word32(b, 240) != 16384 {
            return Err(Reject::Budget);
        }
        if minimum_generation == 0 || self.generation() < minimum_generation {
            return Err(Reject::Rollback);
        }
        Ok(())
    }
}

/// Immutable borrowed bytes were authenticated AND passed fixed PE/W^X bounds.
/// This is not a kernel capability. Caller must retain/seal these exact bytes
/// during mapping, enforce the manifest's reduced trial permissions and health,
/// and reverify from storage after every boot. Public lab root only.
pub struct VerifiedLayer<'a> {
    manifest: Manifest<'a>,
    payload: &'a [u8],
    layout: pe::Layout,
}
impl<'a> VerifiedLayer<'a> {
    pub fn manifest(&self) -> Manifest<'a> { self.manifest }
    pub fn payload(&self) -> &'a [u8] { self.payload }
    pub fn layout(&self) -> &pe::Layout { &self.layout }
}
pub fn verify<'a>(raw: &'a [u8], payload: &'a [u8], minimum_generation: u64)
    -> Result<VerifiedLayer<'a>, Reject>
{
    let manifest = Manifest::parse(raw)?;
    manifest.authenticate()?;
    manifest.policy(minimum_generation)?;
    if !(512..=MAX_PAYLOAD).contains(&payload.len()) ||
        word32(manifest.bytes, 56) as usize != payload.len() {
        return Err(Reject::PayloadLength);
    }
    if hash(payload)? != manifest.payload_digest() { return Err(Reject::PayloadDigest); }
    let layout = pe::parse(payload).map_err(|_| Reject::Executable)?;
    if layout.image_size > word32(manifest.bytes, 60) as usize { return Err(Reject::Budget); }
    Ok(VerifiedLayer { manifest, payload, layout })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn u32put(b: &mut [u8; SIZE], p: usize, v: u32) { b[p..p+4].copy_from_slice(&v.to_le_bytes()); }
    fn fixture() -> [u8; SIZE] {
        let mut b = [0; SIZE];
        b[..8].copy_from_slice(b"RARMODL0");
        b[10..12].copy_from_slice(&(SIZE as u16).to_le_bytes());
        u32put(&mut b, 12, 1);
        b[16..40].copy_from_slice(ALGORITHM);
        u32put(&mut b, 40, 5); b[44] = 1; u32put(&mut b, 48, 1);
        u32put(&mut b, 56, 1024); u32put(&mut b, 60, 8192);
        b[64] = 7; b[72] = 1;
        b[80..112].copy_from_slice(&hash(HEALTH_NAME).unwrap());
        b[144..176].copy_from_slice(&hash(&LAB_PUBLIC_KEY).unwrap());
        b[176..196].fill(1); b[196..228].fill(2);
        u32put(&mut b, 232, 50); u32put(&mut b, 240, 16384);
        let digest = hash(&b[..PREIMAGE]).unwrap();
        b[288..320].copy_from_slice(&digest); b
    }
    #[test]
    fn every_truncation_extension_reserved_and_algorithm_byte_is_rejected() {
        let b = fixture();
        for n in 0..SIZE { assert!(Manifest::parse(&b[..n]).is_err()); }
        assert!(Manifest::parse(&[0; SIZE+1]).is_err());
        for offset in (0..40).chain(244..288) {
            let mut bad = b; bad[offset] ^= 0x80;
            assert!(Manifest::parse(&bad).is_err(), "{offset}");
        }
    }
    #[test]
    fn signature_preimage_excludes_both_entire_fields() {
        let b = fixture();
        let message = Manifest::parse(&b).unwrap().signed_message().unwrap();
        assert_eq!(&message[..19], b"RAR-LAYER-ALPHA-V0\0");
        assert_eq!(&message[19..], &hash(&b[..PREIMAGE]).unwrap());
        for p in PREIMAGE..SIZE {
            let mut changed = b; changed[p] ^= 1;
            assert_eq!(Manifest::parse(&changed).unwrap().signed_message().unwrap(), message);
        }
        for p in 40..244 {
            let mut changed = b; changed[p] ^= 1;
            assert_ne!(Manifest::parse(&changed).unwrap().signed_message().unwrap(), message);
        }
    }
    #[test]
    fn no_unsigned_or_wrong_key_package_returns_verified_bytes() {
        let b = fixture();
        assert!(matches!(verify(&b, &[0;1024], 1), Err(Reject::Signature)));
        let mut changed = b; changed[144] ^= 1; changed[288] ^= 1;
        assert!(matches!(verify(&changed, &[], 1), Err(Reject::Publisher)));
        changed = b; changed[288] ^= 1;
        assert!(matches!(verify(&changed, &[], 1), Err(Reject::Digest)));
        changed = b; changed[72] = 2;
        assert!(matches!(verify(&changed, &[], 1), Err(Reject::Digest)));
    }
    #[test]
    fn bounded_policy_and_rollback_cases() {
        let b = fixture();
        assert_eq!(Manifest::parse(&b).unwrap().policy(1), Ok(()));
        assert_eq!(Manifest::parse(&b).unwrap().policy(2), Err(Reject::Rollback));
        assert_eq!(Manifest::parse(&b).unwrap().policy(0), Err(Reject::Rollback));
        for (offset, value, want) in [
            (40,6,Reject::Compatibility), (44,2,Reject::Compatibility),
            (48,2,Reject::Compatibility), (52,1,Reject::Compatibility),
            (60,4097,Reject::Budget), (60,0,Reject::Budget), (60,131073,Reject::Budget),
            (64,8,Reject::Budget), (232,0,Reject::Budget), (232,101,Reject::Budget),
            (236,1,Reject::Budget), (240,32768,Reject::Budget), (228,1,Reject::Compatibility),
        ] {
            let mut bad = b; u32put(&mut bad, offset, value);
            assert_eq!(Manifest::parse(&bad).unwrap().policy(1), Err(want), "{offset}");
        }
    }
    #[test]
    fn deterministic_bounded_malformed_corpus_never_authenticates() {
        let mut state = 0xd173_b924_5870_acedu64;
        for length in 0..=512 {
            let mut b = [0u8; 512];
            for byte in &mut b[..length] {
                state ^= state << 13; state ^= state >> 7; state ^= state << 17;
                *byte = state as u8;
            }
            assert!(verify(&b[..length], &[], 1).is_err());
        }
    }
}
