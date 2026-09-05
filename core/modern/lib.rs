//! Experimental Modern-v0 protocol logic, not active execution or disk authority.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
#[path = "../crypto/sha256.rs"]
pub mod sha256;
#[path = "../crypto/sha512.rs"]
pub mod sha512;
#[path = "../crypto/ed25519.rs"]
pub mod ed25519;
pub mod manifest;
pub mod journal;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Denied }
#[path = "../../nucleus/platform/pe.rs"]
pub mod pe;
