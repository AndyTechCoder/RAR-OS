#![deny(unsafe_code)]

#[allow(unsafe_code)]
mod descriptor_fs;
mod hash;
mod input_bundle;
mod json;
mod package;
mod transaction;
mod transaction_contracts;

pub use descriptor_fs::{DescriptorDir, HeldSnapshot, snapshot_to_private};
pub use hash::{sha256_hex, sha256_reader};
pub use input_bundle::{InputBundleV1, InputObjectV1, parse_input_bundle_v1};
pub use json::Json;
pub use package::{PackageManifest, PackageRow};
pub use transaction::{ArchiveEntry, ArchivePlan, DebPlan, FrozenTransactionGraph, MemberKind,
    MutationBoundary, OwnedSnapshot, TransactionEffects, TransactionMachine, TransactionPhase,
    plan_deb_ar, plan_tar, validate_closure_inputs};
pub use transaction_contracts::{INPUT_LOCK_FIELDS, InputLockV4, TRANSACTION_GRAPH_FIELDS,
    TransactionGraphV1};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PreauthError { pub code: &'static str }
impl PreauthError { pub(crate) fn new(code: &'static str) -> Self { Self { code } } }
pub type Result<T> = std::result::Result<T, PreauthError>;

pub const MILESTONE_COMPLETENESS: &str = "m1.6-input-delivery-m2-incomplete";
