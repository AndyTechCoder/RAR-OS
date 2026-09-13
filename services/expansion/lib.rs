//! Candidate Expansion services. No active device or guest network authority.
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
pub mod network;
pub mod channel;
pub mod ne2k;

pub mod service;
#[path="../../sdk/expansion/rust/wire.rs"]
pub mod sdk;
