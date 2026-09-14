//! Experimental language-neutral app SDK. No syscall authority.
#![cfg_attr(not(test),no_std)]
#![deny(unsafe_code)]
pub mod wire;
pub use wire::*;
pub mod protocol;
pub mod transport;
#[cfg(all(target_arch="x86_64",any(target_os="uefi",rar_app_object_check)))]
#[allow(unsafe_code)]
pub mod native;
