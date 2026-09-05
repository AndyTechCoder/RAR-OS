//! Experimental RAR-owned cryptographic building blocks.
//! No production trust, signing service or target integration is enabled here.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
pub mod sha512;
pub mod ed25519;
pub mod sha256;
pub mod chacha20poly1305;
