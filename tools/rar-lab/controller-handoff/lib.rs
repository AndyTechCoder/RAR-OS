#![deny(unsafe_code)]

mod attempt;
mod contract;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[allow(unsafe_code)]
mod linux;
mod manifest;
mod sha256;
mod transaction;

pub use contract::{Cancellation, Deadline, DeclaredOutput, PhasePlan, PlanError, ProducerQuiesced, SourceMountKind};
pub use attempt::{
    ACTIVE_HEADER_BYTES, ACTIVE_MAXIMUM_BYTES, AttemptActive, AttemptCause,
    AttemptError, AttemptExpected, AttemptRoot, AttemptState, AttemptTransition,
    EXIT_UNOBSERVED, EXPECTED_ENTRY_BYTES, ExpectedKind, RECOVERY_ENTRY_BYTES,
    RECOVERY_HEADER_BYTES, RECOVERY_MAXIMUM_BYTES, ROOT_RECORD_BYTES,
    RecoveryEntry, RecoveryInventory, RootKind, TRANSITION_BYTES,
};
pub use manifest::{HandoffManifest, MANIFEST_BYTES, ManifestError, OutputKind, Role};
pub use sha256::{Sha256, sha256};
pub use transaction::{BatchReceipt, FileIdentity, FileType, HandoffError, HandoffOps, HandoffResult, HandoffRoots, RootIdentity, SourceRoot, handoff_batch};
