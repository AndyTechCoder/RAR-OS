use crate::{OutputKind, Role};

pub const COPY_BUFFER_BYTES: usize = 64 * 1024;
pub const LAUNCH_AGGREGATE_MAXIMUM: u64 = 67_108_864;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SourceMountKind {
    BuildArtifact,
    BuildTranscript,
    Reference,
    Launch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeclaredOutput {
    pub phase: u16,
    pub role: Role,
    pub output_kind: OutputKind,
    pub output_ordinal: u16,
    pub source_mount: SourceMountKind,
    pub basename: String,
    pub maximum_bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PhasePlan {
    outputs: Vec<DeclaredOutput>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PlanError {
    Empty,
    TooManyOutputs,
    Basename,
    DuplicateBasename,
    AggregateMaximum,
}

fn canonical_basename(value: &str) -> bool {
    let bytes = value.as_bytes();
    (1..=64).contains(&bytes.len())
        && value != "."
        && value != ".."
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-')
        })
}

impl PhasePlan {
    pub fn build_one() -> Self { Self::build(2) }

    pub fn build_two() -> Self { Self::build(3) }

    fn build(phase: u16) -> Self {
        Self {
            outputs: vec![
                DeclaredOutput {
                    phase,
                    role: Role::Build,
                    output_kind: OutputKind::Artifact,
                    output_ordinal: 1,
                    source_mount: SourceMountKind::BuildArtifact,
                    basename: "rar-os-alpha.img".into(),
                    maximum_bytes: 67_108_864,
                },
                DeclaredOutput {
                    phase,
                    role: Role::Build,
                    output_kind: OutputKind::Transcript,
                    output_ordinal: 2,
                    source_mount: SourceMountKind::BuildTranscript,
                    basename: "comparison.bin".into(),
                    maximum_bytes: 1_048_576,
                },
            ],
        }
    }

    pub fn reference() -> Self {
        Self {
            outputs: vec![DeclaredOutput {
                phase: 5,
                role: Role::Reference,
                output_kind: OutputKind::ComparisonEvidence,
                output_ordinal: 1,
                source_mount: SourceMountKind::Reference,
                basename: "comparison-evidence.bin".into(),
                maximum_bytes: 1_048_576,
            }],
        }
    }

    pub fn launch(ordered_allowlist: &[String]) -> Result<Self, PlanError> {
        if ordered_allowlist.is_empty() { return Err(PlanError::Empty); }
        if ordered_allowlist.len() > 999 { return Err(PlanError::TooManyOutputs); }
        let mut seen = std::collections::BTreeSet::new();
        let mut outputs = Vec::with_capacity(ordered_allowlist.len());
        for (index, basename) in ordered_allowlist.iter().enumerate() {
            if !canonical_basename(basename) { return Err(PlanError::Basename); }
            if !seen.insert(basename.as_str()) { return Err(PlanError::DuplicateBasename); }
            outputs.push(DeclaredOutput {
                phase: 7,
                role: Role::Launch,
                output_kind: OutputKind::LaunchEvidence,
                output_ordinal: u16::try_from(index + 1).map_err(|_| PlanError::TooManyOutputs)?,
                source_mount: SourceMountKind::Launch,
                basename: basename.clone(),
                maximum_bytes: LAUNCH_AGGREGATE_MAXIMUM,
            });
        }
        Ok(Self { outputs })
    }

    pub fn outputs(&self) -> &[DeclaredOutput] { &self.outputs }

    pub fn validate_aggregate(&self, sizes: &[u64]) -> Result<(), PlanError> {
        if sizes.len() != self.outputs.len() { return Err(PlanError::AggregateMaximum); }
        if self.outputs.first().map(|output| output.role) != Some(Role::Launch) { return Ok(()); }
        let total = sizes.iter().try_fold(0u64, |sum, size| sum.checked_add(*size))
            .ok_or(PlanError::AggregateMaximum)?;
        if total > LAUNCH_AGGREGATE_MAXIMUM { return Err(PlanError::AggregateMaximum); }
        Ok(())
    }
}

#[derive(Debug)]
pub struct ProducerQuiesced {
    _private: (),
}

impl ProducerQuiesced {
    pub(crate) fn from_controller_observation() -> Self { Self { _private: () } }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Deadline {
    monotonic_nanoseconds: u64,
}

impl Deadline {
    pub fn from_monotonic_nanoseconds(value: u64) -> Self {
        Self { monotonic_nanoseconds: value }
    }

    pub fn has_expired(self, now: u64) -> bool { now >= self.monotonic_nanoseconds }
}

pub trait Cancellation {
    fn is_cancelled(&self) -> bool;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phase_plans_bind_ordinals_and_launch_allowlist() {
        let build = PhasePlan::build_one();
        assert_eq!(build.outputs()[0].output_ordinal, 1);
        assert_eq!(build.outputs()[1].output_ordinal, 2);
        let launch = PhasePlan::launch(&["serial.log".into(), "framebuffer-001.bin".into()]).unwrap();
        assert_eq!(launch.outputs()[0].output_ordinal, 1);
        assert_eq!(launch.outputs()[1].output_ordinal, 2);
        assert_eq!(PhasePlan::launch(&["../escape".into()]), Err(PlanError::Basename));
        assert_eq!(PhasePlan::launch(&["same.bin".into(), "same.bin".into()]), Err(PlanError::DuplicateBasename));
    }
}
