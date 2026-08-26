#![forbid(unsafe_code)]

mod manifest;
mod sha256;

pub use manifest::{HandoffManifest, ManifestError, OutputKind, Role};
pub use sha256::{Sha256, sha256};
