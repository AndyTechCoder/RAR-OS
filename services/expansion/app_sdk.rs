//! Shared safe app codecs for service dispatch, without native syscall adapters.
#![forbid(unsafe_code)]
#[path="../../sdk/alpha/rust/wire.rs"]pub mod wire;
#[path="../../sdk/alpha/rust/protocol.rs"]pub mod protocol;
