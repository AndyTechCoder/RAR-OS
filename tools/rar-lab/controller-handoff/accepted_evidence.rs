//! Side-effect-free codec for the accepted Alpha evidence record.
//!
//! This module implements only the already-reviewed 20-line record contract.
//! It has no filesystem, publication, cleanup, retry, recovery, or controller
//! authority. In particular, it does not choose any final or temporary name.

use std::fmt::Write as _;

use crate::sha256;

pub const ACCEPTED_EVIDENCE_MAXIMUM_BYTES: usize = 4096;
pub const ACCEPTANCE_PROTOCOL_SHA256: [u8; 32] = [
    0xff, 0xdb, 0x07, 0xb5, 0x84, 0xab, 0xc9, 0x41,
    0x22, 0xb1, 0x4a, 0x41, 0x65, 0x93, 0x91, 0x6c,
    0xf1, 0x8d, 0xf4, 0x39, 0xde, 0x04, 0x2c, 0x97,
    0xff, 0x83, 0xfd, 0xa9, 0xe4, 0x44, 0x4c, 0xcd,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Milestone { A, B, C, D, E, F, G }

impl Milestone {
    fn as_str(self) -> &'static str {
        match self {
            Self::A => "milestone-a", Self::B => "milestone-b",
            Self::C => "milestone-c", Self::D => "milestone-d",
            Self::E => "milestone-e", Self::F => "milestone-f",
            Self::G => "milestone-g",
        }
    }

    fn parse(value: &str) -> Result<Self, AcceptedEvidenceError> {
        match value {
            "milestone-a" => Ok(Self::A), "milestone-b" => Ok(Self::B),
            "milestone-c" => Ok(Self::C), "milestone-d" => Ok(Self::D),
            "milestone-e" => Ok(Self::E), "milestone-f" => Ok(Self::F),
            "milestone-g" => Ok(Self::G),
            _ => Err(AcceptedEvidenceError::new("probe-invalid")),
        }
    }

    fn requires_reference(self) -> bool { matches!(self, Self::F | Self::G) }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedEvidenceBindings {
    pub attempt_nonce: [u8; 32],
    pub probe: Milestone,
    pub controller_revision: [u8; 20],
    pub source_revision: [u8; 20],
    pub artifact_sha256: [u8; 32],
    pub acceptance_protocol_sha256: [u8; 32],
    pub machine_profile_sha256: [u8; 32],
    pub qemu_sha256: [u8; 32],
    pub firmware_sha256: [u8; 32],
    pub qmp_client_sha256: [u8; 32],
    pub actions_sha256: [u8; 32],
    pub serial_sha256: [u8; 32],
    pub final_capture_sha256: [u8; 32],
    pub capture_set_sha256: [u8; 32],
    pub handoff_manifest_set_sha256: [u8; 32],
    pub reference_verdict_sha256: [u8; 32],
    pub role_inventories_sha256: [u8; 32],
    pub accepted_outputs_sha256: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedEvidenceRecord { bindings: AcceptedEvidenceBindings }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedEvidenceError { pub code: &'static str }

impl AcceptedEvidenceError {
    fn new(code: &'static str) -> Self { Self { code } }
}

fn is_zero<const N: usize>(value: &[u8; N]) -> bool {
    value.iter().all(|byte| *byte == 0)
}

fn validate(bindings: &AcceptedEvidenceBindings) -> Result<(), AcceptedEvidenceError> {
    if is_zero(&bindings.attempt_nonce) {
        return Err(AcceptedEvidenceError::new("attempt-nonce-zero"));
    }
    if is_zero(&bindings.controller_revision) || is_zero(&bindings.source_revision) {
        return Err(AcceptedEvidenceError::new("revision-zero"));
    }
    if bindings.acceptance_protocol_sha256 != ACCEPTANCE_PROTOCOL_SHA256 {
        return Err(AcceptedEvidenceError::new("protocol-mismatch"));
    }
    for digest in [
        &bindings.artifact_sha256, &bindings.machine_profile_sha256,
        &bindings.qemu_sha256, &bindings.firmware_sha256,
        &bindings.qmp_client_sha256, &bindings.actions_sha256,
        &bindings.serial_sha256, &bindings.final_capture_sha256,
        &bindings.capture_set_sha256, &bindings.handoff_manifest_set_sha256,
        &bindings.role_inventories_sha256, &bindings.accepted_outputs_sha256,
    ] {
        if is_zero(digest) {
            return Err(AcceptedEvidenceError::new("required-digest-zero"));
        }
    }
    if bindings.probe.requires_reference() == is_zero(&bindings.reference_verdict_sha256) {
        return Err(AcceptedEvidenceError::new("reference-rule"));
    }
    Ok(())
}

fn push_hex<const N: usize>(output: &mut String, value: &[u8; N]) {
    for byte in value {
        write!(output, "{byte:02x}").expect("String formatting cannot fail");
    }
}

fn push_field<const N: usize>(output: &mut String, key: &str, value: &[u8; N]) {
    output.push_str(key);
    output.push('=');
    push_hex(output, value);
    output.push('\n');
}

fn decode_nibble(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

fn parse_hex<const N: usize>(value: &str) -> Result<[u8; N], AcceptedEvidenceError> {
    if value.len() != N * 2 {
        return Err(AcceptedEvidenceError::new("hex-length"));
    }
    let mut output = [0u8; N];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        let high = decode_nibble(pair[0])
            .ok_or_else(|| AcceptedEvidenceError::new("hex-grammar"))?;
        let low = decode_nibble(pair[1])
            .ok_or_else(|| AcceptedEvidenceError::new("hex-grammar"))?;
        output[index] = high << 4 | low;
    }
    Ok(output)
}

fn value<'a>(line: &'a str, key: &str) -> Result<&'a str, AcceptedEvidenceError> {
    line.strip_prefix(key).and_then(|rest| rest.strip_prefix('='))
        .ok_or_else(|| AcceptedEvidenceError::new("field-order-or-name"))
}

impl AcceptedEvidenceRecord {
    pub fn new(bindings: AcceptedEvidenceBindings) -> Result<Self, AcceptedEvidenceError> {
        validate(&bindings)?;
        let record = Self { bindings };
        if is_zero(&sha256(&record.preimage())) {
            return Err(AcceptedEvidenceError::new("record-digest-zero"));
        }
        Ok(record)
    }

    pub fn bindings(&self) -> &AcceptedEvidenceBindings { &self.bindings }

    pub fn record_sha256(&self) -> [u8; 32] { sha256(&self.preimage()) }

    fn preimage(&self) -> Vec<u8> {
        let mut output = String::with_capacity(1400);
        output.push_str("schema=rar-alpha-accepted-evidence-v0\n");
        push_field(&mut output, "attempt_nonce", &self.bindings.attempt_nonce);
        output.push_str("probe=");
        output.push_str(self.bindings.probe.as_str());
        output.push('\n');
        push_field(&mut output, "controller_revision", &self.bindings.controller_revision);
        push_field(&mut output, "source_revision", &self.bindings.source_revision);
        push_field(&mut output, "artifact_sha256", &self.bindings.artifact_sha256);
        push_field(&mut output, "acceptance_protocol_sha256", &self.bindings.acceptance_protocol_sha256);
        push_field(&mut output, "machine_profile_sha256", &self.bindings.machine_profile_sha256);
        push_field(&mut output, "qemu_sha256", &self.bindings.qemu_sha256);
        push_field(&mut output, "firmware_sha256", &self.bindings.firmware_sha256);
        push_field(&mut output, "qmp_client_sha256", &self.bindings.qmp_client_sha256);
        push_field(&mut output, "actions_sha256", &self.bindings.actions_sha256);
        push_field(&mut output, "serial_sha256", &self.bindings.serial_sha256);
        push_field(&mut output, "final_capture_sha256", &self.bindings.final_capture_sha256);
        push_field(&mut output, "capture_set_sha256", &self.bindings.capture_set_sha256);
        push_field(&mut output, "handoff_manifest_set_sha256", &self.bindings.handoff_manifest_set_sha256);
        push_field(&mut output, "reference_verdict_sha256", &self.bindings.reference_verdict_sha256);
        push_field(&mut output, "role_inventories_sha256", &self.bindings.role_inventories_sha256);
        push_field(&mut output, "accepted_outputs_sha256", &self.bindings.accepted_outputs_sha256);
        output.into_bytes()
    }

    pub fn encode(&self) -> Vec<u8> {
        let mut output = String::from_utf8(self.preimage())
            .expect("canonical record is ASCII");
        let digest = sha256(output.as_bytes());
        push_field(&mut output, "record_sha256", &digest);
        debug_assert!(output.len() <= ACCEPTED_EVIDENCE_MAXIMUM_BYTES);
        output.into_bytes()
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, AcceptedEvidenceError> {
        if bytes.is_empty() || bytes.len() > ACCEPTED_EVIDENCE_MAXIMUM_BYTES {
            return Err(AcceptedEvidenceError::new("size"));
        }
        if bytes.last() != Some(&b'\n')
            || bytes.iter().any(|byte| *byte == 0 || *byte == b'\r')
        {
            return Err(AcceptedEvidenceError::new("framing"));
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| AcceptedEvidenceError::new("encoding"))?;
        if !text.is_ascii() { return Err(AcceptedEvidenceError::new("encoding")); }
        let lines: Vec<&str> = text[..text.len() - 1].split('\n').collect();
        if lines.len() != 20 || lines.iter().any(|line| line.is_empty()) {
            return Err(AcceptedEvidenceError::new("line-count"));
        }
        if lines[0] != "schema=rar-alpha-accepted-evidence-v0" {
            return Err(AcceptedEvidenceError::new("schema"));
        }
        let bindings = AcceptedEvidenceBindings {
            attempt_nonce: parse_hex(value(lines[1], "attempt_nonce")?)?,
            probe: Milestone::parse(value(lines[2], "probe")?)?,
            controller_revision: parse_hex(value(lines[3], "controller_revision")?)?,
            source_revision: parse_hex(value(lines[4], "source_revision")?)?,
            artifact_sha256: parse_hex(value(lines[5], "artifact_sha256")?)?,
            acceptance_protocol_sha256: parse_hex(value(lines[6], "acceptance_protocol_sha256")?)?,
            machine_profile_sha256: parse_hex(value(lines[7], "machine_profile_sha256")?)?,
            qemu_sha256: parse_hex(value(lines[8], "qemu_sha256")?)?,
            firmware_sha256: parse_hex(value(lines[9], "firmware_sha256")?)?,
            qmp_client_sha256: parse_hex(value(lines[10], "qmp_client_sha256")?)?,
            actions_sha256: parse_hex(value(lines[11], "actions_sha256")?)?,
            serial_sha256: parse_hex(value(lines[12], "serial_sha256")?)?,
            final_capture_sha256: parse_hex(value(lines[13], "final_capture_sha256")?)?,
            capture_set_sha256: parse_hex(value(lines[14], "capture_set_sha256")?)?,
            handoff_manifest_set_sha256: parse_hex(value(lines[15], "handoff_manifest_set_sha256")?)?,
            reference_verdict_sha256: parse_hex(value(lines[16], "reference_verdict_sha256")?)?,
            role_inventories_sha256: parse_hex(value(lines[17], "role_inventories_sha256")?)?,
            accepted_outputs_sha256: parse_hex(value(lines[18], "accepted_outputs_sha256")?)?,
        };
        let supplied_record_sha: [u8; 32] =
            parse_hex(value(lines[19], "record_sha256")?)?;
        if is_zero(&supplied_record_sha) {
            return Err(AcceptedEvidenceError::new("record-digest-zero"));
        }
        let preimage = lines[..19].join("\n") + "\n";
        if sha256(preimage.as_bytes()) != supplied_record_sha {
            return Err(AcceptedEvidenceError::new("record-digest"));
        }
        let record = Self::new(bindings)?;
        if record.encode() != bytes {
            return Err(AcceptedEvidenceError::new("noncanonical"));
        }
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical_lines(probe: Milestone) -> Vec<String> {
        let encoded = AcceptedEvidenceRecord::new(bindings(probe)).unwrap().encode();
        String::from_utf8(encoded).unwrap().lines()
            .take(19).map(str::to_owned).collect()
    }

    fn encode_lines(lines: &[String]) -> Vec<u8> {
        let mut preimage = lines.join("\n");
        preimage.push('\n');
        let digest = sha256(preimage.as_bytes());
        let mut output = preimage;
        push_field(&mut output, "record_sha256", &digest);
        output.into_bytes()
    }

    fn bindings(probe: Milestone) -> AcceptedEvidenceBindings {
        AcceptedEvidenceBindings {
            attempt_nonce: [0xaa; 32], probe,
            controller_revision: [0xcc; 20], source_revision: [0xdd; 20],
            artifact_sha256: [0x01; 32],
            acceptance_protocol_sha256: ACCEPTANCE_PROTOCOL_SHA256,
            machine_profile_sha256: [0x02; 32], qemu_sha256: [0x03; 32],
            firmware_sha256: [0x04; 32], qmp_client_sha256: [0x05; 32],
            actions_sha256: [0x06; 32], serial_sha256: [0x07; 32],
            final_capture_sha256: [0x08; 32], capture_set_sha256: [0x09; 32],
            handoff_manifest_set_sha256: [0x0a; 32],
            reference_verdict_sha256: if probe.requires_reference() { [0xee; 32] } else { [0; 32] },
            role_inventories_sha256: [0x0b; 32],
            accepted_outputs_sha256: [0x0c; 32],
        }
    }

    #[test]
    fn canonical_round_trip_and_mutations() {
        for probe in [Milestone::A, Milestone::B, Milestone::C, Milestone::D,
            Milestone::E, Milestone::F, Milestone::G]
        {
            let record = AcceptedEvidenceRecord::new(bindings(probe)).unwrap();
            let encoded = record.encode();
            assert_eq!(AcceptedEvidenceRecord::decode(&encoded).unwrap(), record);
            assert_eq!(encoded.iter().filter(|byte| **byte == b'\n').count(), 20);
        }
        let mut uppercase = AcceptedEvidenceRecord::new(bindings(Milestone::A))
            .unwrap().encode();
        let attempt = b"attempt_nonce=";
        let position = uppercase.windows(attempt.len())
            .position(|bytes| bytes == attempt).unwrap() + attempt.len();
        uppercase[position] = b'A';
        assert!(AcceptedEvidenceRecord::decode(&uppercase).is_err());

        let mut trailing = AcceptedEvidenceRecord::new(bindings(Milestone::A))
            .unwrap().encode();
        trailing.push(b'x');
        assert!(AcceptedEvidenceRecord::decode(&trailing).is_err());

        let text = String::from_utf8(
            AcceptedEvidenceRecord::new(bindings(Milestone::A)).unwrap().encode(),
        ).unwrap();
        let mut lines: Vec<&str> = text.lines().collect();
        lines.swap(1, 2);
        let swapped = (lines.join("\n") + "\n").into_bytes();
        assert!(AcceptedEvidenceRecord::decode(&swapped).is_err());

        let mut zero_record_digest = AcceptedEvidenceRecord::new(bindings(Milestone::A))
            .unwrap().encode();
        let digest = b"record_sha256=";
        let position = zero_record_digest.windows(digest.len())
            .position(|bytes| bytes == digest).unwrap() + digest.len();
        zero_record_digest[position..position + 64].fill(b'0');
        assert_eq!(AcceptedEvidenceRecord::decode(&zero_record_digest).unwrap_err().code,
            "record-digest-zero");
    }

    #[test]
    fn matches_language_neutral_golden_record() {
        let record = AcceptedEvidenceRecord::new(bindings(Milestone::A)).unwrap();
        assert_eq!(record.encode(), include_bytes!("fixtures/accepted-evidence-golden.v0"));
        assert_eq!(record.record_sha256(), [
            0x2a, 0xf4, 0x33, 0x51, 0xc6, 0xe5, 0x14, 0x75,
            0x5f, 0x59, 0x13, 0xdf, 0x85, 0x3c, 0x7a, 0xb7,
            0x2e, 0xe6, 0x64, 0x71, 0xc8, 0x19, 0x75, 0x23,
            0x45, 0x8b, 0xa0, 0xd7, 0x3d, 0x31, 0x46, 0x3f,
        ]);

        let reference_record = AcceptedEvidenceRecord::new(bindings(Milestone::F)).unwrap();
        assert_eq!(reference_record.encode(),
            include_bytes!("fixtures/accepted-evidence-golden-f.v0"));
        assert_eq!(reference_record.record_sha256(), [
            0xc3, 0xbe, 0x14, 0xc9, 0x6a, 0x0c, 0x60, 0x99,
            0xb1, 0x72, 0xf5, 0x12, 0x3a, 0x08, 0x99, 0x00,
            0x8f, 0x6b, 0x9a, 0x92, 0x29, 0xd7, 0x51, 0x8e,
            0x1e, 0x7e, 0x4f, 0xdd, 0x72, 0x17, 0x5f, 0x60,
        ]);
    }

    #[test]
    fn rejects_zero_and_reference_rule_violations() {
        let mut zero = bindings(Milestone::A);
        zero.actions_sha256 = [0; 32];
        assert_eq!(AcceptedEvidenceRecord::new(zero).unwrap_err().code,
            "required-digest-zero");

        let mut early_reference = bindings(Milestone::A);
        early_reference.reference_verdict_sha256 = [1; 32];
        assert_eq!(AcceptedEvidenceRecord::new(early_reference).unwrap_err().code,
            "reference-rule");

        let mut missing_reference = bindings(Milestone::F);
        missing_reference.reference_verdict_sha256 = [0; 32];
        assert_eq!(AcceptedEvidenceRecord::new(missing_reference).unwrap_err().code,
            "reference-rule");
    }

    #[test]
    fn rejects_malformed_and_boundary_records() {
        assert_eq!(AcceptedEvidenceRecord::decode(&[]).unwrap_err().code, "size");
        assert_eq!(AcceptedEvidenceRecord::decode(&vec![b'x'; 4097]).unwrap_err().code,
            "size");

        let canonical = AcceptedEvidenceRecord::new(bindings(Milestone::A)).unwrap().encode();
        for byte in [b'\r', 0] {
            let mut malformed = canonical.clone();
            malformed[0] = byte;
            assert_eq!(AcceptedEvidenceRecord::decode(&malformed).unwrap_err().code,
                "framing");
        }
        let mut non_utf8 = canonical.clone();
        non_utf8[0] = 0xff;
        assert_eq!(AcceptedEvidenceRecord::decode(&non_utf8).unwrap_err().code,
            "encoding");

        let mut missing = canonical_lines(Milestone::A);
        missing.remove(10);
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&missing)).unwrap_err().code,
            "line-count");

        let mut duplicate = canonical_lines(Milestone::A);
        duplicate[10] = duplicate[9].clone();
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&duplicate)).unwrap_err().code,
            "field-order-or-name");

        let mut unknown = canonical_lines(Milestone::A);
        unknown[10] = unknown[10].replacen("qmp_client_sha256", "unknown_sha256", 1);
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&unknown)).unwrap_err().code,
            "field-order-or-name");

        let mut invalid_probe = canonical_lines(Milestone::A);
        invalid_probe[2] = "probe=milestone-z".into();
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&invalid_probe)).unwrap_err().code,
            "probe-invalid");

        let mut invalid_protocol = canonical_lines(Milestone::A);
        invalid_protocol[6] = format!("acceptance_protocol_sha256={}", "00".repeat(32));
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&invalid_protocol)).unwrap_err().code,
            "protocol-mismatch");

        for index in [1usize, 3, 4] {
            let mut zero = canonical_lines(Milestone::A);
            let key = zero[index].split('=').next().unwrap().to_owned();
            let bytes = if index == 1 { 32 } else { 20 };
            zero[index] = format!("{key}={}", "00".repeat(bytes));
            let error = AcceptedEvidenceRecord::decode(&encode_lines(&zero)).unwrap_err();
            assert!(matches!(error.code, "attempt-nonce-zero" | "revision-zero"));
        }

        let mut short_hex = canonical_lines(Milestone::A);
        short_hex[5].pop();
        assert_eq!(AcceptedEvidenceRecord::decode(&encode_lines(&short_hex)).unwrap_err().code,
            "hex-length");

        let mut bad_record_digest = canonical.clone();
        let last_hex = bad_record_digest.len() - 2;
        bad_record_digest[last_hex] = if bad_record_digest[last_hex] == b'0' { b'1' } else { b'0' };
        assert_eq!(AcceptedEvidenceRecord::decode(&bad_record_digest).unwrap_err().code,
            "record-digest");
    }
}
