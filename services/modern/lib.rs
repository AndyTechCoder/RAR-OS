#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
pub mod pio;
#[path = "../../core/crypto/sha256.rs"]
pub mod sha256;
#[path = "../../core/crypto/chacha20poly1305.rs"]
pub mod chacha20poly1305;
pub mod vault;

// Reuse historical request constants/encoding without changing its volatile Store.
#[path = "../platform/model.rs"]
pub mod desktop_wire;
pub mod store;
