//! Experimental language-neutral app SDK. No syscall authority.
#![cfg_attr(not(test),no_std)]
#![forbid(unsafe_code)]
pub mod wire;
pub use wire::*;
