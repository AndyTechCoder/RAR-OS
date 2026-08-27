#![deny(unsafe_code)]

mod contract;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(unsafe_code)]
mod linux;
mod manifest;
mod sha256;
mod transaction;

pub use contract::{Cancellation, Deadline, DeclaredOutput, PhasePlan, PlanError, ProducerQuiesced, SourceMountKind};
pub use manifest::{HandoffManifest, MANIFEST_BYTES, ManifestError, OutputKind, Role};
pub use sha256::{Sha256, sha256};
pub use transaction::{BatchReceipt, FileIdentity, FileType, HandoffError, HandoffOps, HandoffResult, HandoffRoots, RootIdentity, SourceRoot, handoff_batch};
