const MAGIC: [u8; 8] = *b"RARHAND\0";
pub const MANIFEST_BYTES: usize = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Role { Build = 1, Reference = 2, Launch = 3 }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum OutputKind { Artifact = 1, Transcript = 2, ComparisonEvidence = 3, LaunchEvidence = 4 }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HandoffManifest {
    pub phase: u16,
    pub role: Role,
    pub output_kind: OutputKind,
    pub output_ordinal: u16,
    pub basename: String,
    pub output_bytes: u64,
    pub output_sha256: [u8; 32],
    pub source_device: u64,
    pub source_inode: u64,
    pub destination_device: u64,
    pub destination_inode: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestError {
    Size, Magic, Version, TotalBytes, PhaseRoleKind, Ordinal, Basename,
    Flags, Reserved, OutputBytes, Identity,
}

impl HandoffManifest {
    pub fn file_name(&self) -> Result<String, ManifestError> {
        self.validate()?;
        Ok(format!("handoff-p{:02}-o{:03}.v0", self.phase, self.output_ordinal))
    }

    pub fn encode(&self) -> Result<[u8; MANIFEST_BYTES], ManifestError> {
        self.validate()?;
        let mut out = [0u8; MANIFEST_BYTES];
        out[..8].copy_from_slice(&MAGIC);
        put_u16(&mut out, 8, 0);
        put_u16(&mut out, 10, 0);
        put_u16(&mut out, 12, MANIFEST_BYTES as u16);
        put_u16(&mut out, 14, self.phase);
        put_u16(&mut out, 16, self.role as u16);
        put_u16(&mut out, 18, self.output_kind as u16);
        put_u16(&mut out, 20, self.output_ordinal);
        put_u16(&mut out, 22, self.basename.len() as u16);
        put_u64(&mut out, 32, self.output_bytes);
        out[40..72].copy_from_slice(&self.output_sha256);
        put_u64(&mut out, 72, self.source_device);
        put_u64(&mut out, 80, self.source_inode);
        put_u64(&mut out, 88, self.destination_device);
        put_u64(&mut out, 96, self.destination_inode);
        out[104..104 + self.basename.len()].copy_from_slice(self.basename.as_bytes());
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, ManifestError> {
        if bytes.len() != MANIFEST_BYTES { return Err(ManifestError::Size); }
        if bytes[..8] != MAGIC { return Err(ManifestError::Magic); }
        if get_u16(bytes, 8) != 0 || get_u16(bytes, 10) != 0 { return Err(ManifestError::Version); }
        if get_u16(bytes, 12) as usize != MANIFEST_BYTES { return Err(ManifestError::TotalBytes); }
        if get_u32(bytes, 24) != 0 { return Err(ManifestError::Flags); }
        if get_u32(bytes, 28) != 0 || bytes[168..].iter().any(|byte| *byte != 0) { return Err(ManifestError::Reserved); }
        let basename_len = get_u16(bytes, 22) as usize;
        if !(1..=64).contains(&basename_len) { return Err(ManifestError::Basename); }
        if bytes[104 + basename_len..168].iter().any(|byte| *byte != 0) { return Err(ManifestError::Basename); }
        let basename = core::str::from_utf8(&bytes[104..104 + basename_len]).map_err(|_| ManifestError::Basename)?.to_owned();
        let role = match get_u16(bytes, 16) {
            1 => Role::Build, 2 => Role::Reference, 3 => Role::Launch,
            _ => return Err(ManifestError::PhaseRoleKind),
        };
        let output_kind = match get_u16(bytes, 18) {
            1 => OutputKind::Artifact, 2 => OutputKind::Transcript,
            3 => OutputKind::ComparisonEvidence, 4 => OutputKind::LaunchEvidence,
            _ => return Err(ManifestError::PhaseRoleKind),
        };
        let mut digest = [0u8; 32];
        digest.copy_from_slice(&bytes[40..72]);
        let value = Self {
            phase: get_u16(bytes, 14), role, output_kind,
            output_ordinal: get_u16(bytes, 20), basename,
            output_bytes: get_u64(bytes, 32), output_sha256: digest,
            source_device: get_u64(bytes, 72), source_inode: get_u64(bytes, 80),
            destination_device: get_u64(bytes, 88), destination_inode: get_u64(bytes, 96),
        };
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), ManifestError> {
        let combination = matches!((self.phase, self.role, self.output_kind),
            (2 | 3, Role::Build, OutputKind::Artifact | OutputKind::Transcript) |
            (5, Role::Reference, OutputKind::ComparisonEvidence) |
            (7, Role::Launch, OutputKind::LaunchEvidence));
        if !combination { return Err(ManifestError::PhaseRoleKind); }
        let ordinal_valid = match (self.phase, self.role, self.output_kind) {
            (2 | 3, Role::Build, OutputKind::Artifact) => self.output_ordinal == 1,
            (2 | 3, Role::Build, OutputKind::Transcript) => self.output_ordinal == 2,
            (5, Role::Reference, OutputKind::ComparisonEvidence) => self.output_ordinal == 1,
            (7, Role::Launch, OutputKind::LaunchEvidence) => (1..=999).contains(&self.output_ordinal),
            _ => false,
        };
        if !ordinal_valid { return Err(ManifestError::Ordinal); }
        let name = self.basename.as_bytes();
        if !(1..=64).contains(&name.len()) || self.basename == "." || self.basename == ".." ||
            !name.iter().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'.' | b'-')) {
            return Err(ManifestError::Basename);
        }
        let maximum = match self.output_kind {
            OutputKind::Transcript | OutputKind::ComparisonEvidence => 1_048_576,
            OutputKind::Artifact | OutputKind::LaunchEvidence => 67_108_864,
        };
        if self.output_bytes == 0 || self.output_bytes > maximum { return Err(ManifestError::OutputBytes); }
        if [self.source_device, self.source_inode, self.destination_device, self.destination_inode].contains(&0) {
            return Err(ManifestError::Identity);
        }
        Ok(())
    }
}

fn put_u16(out: &mut [u8], offset: usize, value: u16) { out[offset..offset + 2].copy_from_slice(&value.to_le_bytes()); }
fn put_u64(out: &mut [u8], offset: usize, value: u64) { out[offset..offset + 8].copy_from_slice(&value.to_le_bytes()); }
fn get_u16(input: &[u8], offset: usize) -> u16 { u16::from_le_bytes(input[offset..offset + 2].try_into().expect("bounded manifest")) }
fn get_u32(input: &[u8], offset: usize) -> u32 { u32::from_le_bytes(input[offset..offset + 4].try_into().expect("bounded manifest")) }
fn get_u64(input: &[u8], offset: usize) -> u64 { u64::from_le_bytes(input[offset..offset + 8].try_into().expect("bounded manifest")) }

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical() -> HandoffManifest {
        HandoffManifest {
            phase: 2, role: Role::Build, output_kind: OutputKind::Artifact,
            output_ordinal: 1, basename: "rar-os-alpha.img".into(), output_bytes: 4096,
            output_sha256: [0x5a; 32], source_device: 1, source_inode: 2,
            destination_device: 3, destination_inode: 4,
        }
    }

    #[test]
    fn round_trip_and_layout() {
        let value = canonical();
        let bytes = value.encode().unwrap();
        assert_eq!(&bytes[..8], b"RARHAND\0");
        assert_eq!(bytes.len(), 256);
        assert_eq!(HandoffManifest::decode(&bytes).unwrap(), value);
        assert_eq!(value.file_name().unwrap(), "handoff-p02-o001.v0");
    }

    #[test]
    fn matches_language_neutral_golden_vector() {
        let text = include_str!("fixtures/manifest-golden.v0.hex").trim();
        assert_eq!(text.len(), MANIFEST_BYTES * 2);
        let mut golden = [0u8; MANIFEST_BYTES];
        for (index, output) in golden.iter_mut().enumerate() {
            *output = u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap();
        }
        let value = canonical();
        assert_eq!(value.encode().unwrap(), golden);
        assert_eq!(HandoffManifest::decode(&golden).unwrap(), value);
    }

    #[test]
    fn accepts_every_phase_role_kind_combination() {
        for (phase, role, kind, ordinal, name, maximum) in [
            (2, Role::Build, OutputKind::Artifact, 1, "rar-os-alpha.img", 67_108_864),
            (2, Role::Build, OutputKind::Transcript, 2, "comparison.bin", 1_048_576),
            (3, Role::Build, OutputKind::Artifact, 1, "rar-os-alpha.img", 67_108_864),
            (3, Role::Build, OutputKind::Transcript, 2, "comparison.bin", 1_048_576),
            (5, Role::Reference, OutputKind::ComparisonEvidence, 1, "comparison-evidence.bin", 1_048_576),
            (7, Role::Launch, OutputKind::LaunchEvidence, 1, "framebuffer-001.bin", 67_108_864),
        ] {
            let mut value = canonical();
            value.phase = phase; value.role = role; value.output_kind = kind;
            value.output_ordinal = ordinal; value.basename = name.into(); value.output_bytes = maximum;
            let encoded = value.encode().unwrap();
            assert_eq!(HandoffManifest::decode(&encoded).unwrap(), value);
        }
    }

    #[test]
    fn rejects_noncanonical_values() {
        let mut value = canonical();
        value.basename = "../escape".into();
        assert_eq!(value.encode(), Err(ManifestError::Basename));
        value = canonical(); value.output_bytes = 67_108_865;
        assert_eq!(value.encode(), Err(ManifestError::OutputBytes));
        value = canonical(); value.phase = 5;
        assert_eq!(value.encode(), Err(ManifestError::PhaseRoleKind));
        value = canonical(); value.source_inode = 0;
        assert_eq!(value.encode(), Err(ManifestError::Identity));
        value = canonical(); value.output_ordinal = 2;
        assert_eq!(value.encode(), Err(ManifestError::Ordinal));
        value = canonical(); value.output_kind = OutputKind::Transcript;
        value.basename = "comparison.bin".into(); value.output_bytes = 1024;
        assert_eq!(value.encode(), Err(ManifestError::Ordinal));
    }

    #[test]
    fn rejects_mutated_wire_rules() {
        let original = canonical().encode().unwrap();
        for offset in [0usize, 8, 12, 24, 28, 104 + 63, 168, 255] {
            let mut changed = original;
            changed[offset] ^= 1;
            assert!(HandoffManifest::decode(&changed).is_err(), "offset {offset}");
        }
        let mut trailing = original.to_vec();
        trailing.push(0);
        assert_eq!(HandoffManifest::decode(&trailing), Err(ManifestError::Size));
    }
}
