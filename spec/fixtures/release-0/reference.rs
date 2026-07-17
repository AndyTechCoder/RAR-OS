#![forbid(unsafe_code)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BUNDLE_MAGIC: &[u8; 8] = b"R0FXBIN\0";
const BOOT_MAGIC: &[u8; 8] = b"RARBOOT\0";
const RHD_MAGIC: &[u8; 8] = b"RARRHD\0\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
enum Code {
    Ok = 0,
    Truncated = 1,
    Oversized = 2,
    BadMagic = 3,
    UnsupportedMajor = 4,
    UnsupportedMinor = 5,
    BadHeaderSize = 6,
    UnsupportedFlags = 7,
    NonzeroReserved = 8,
    BadAlignment = 9,
    RangeOverflow = 10,
    InvalidPointerRange = 12,
    Overlap = 13,
    BadCountOrLength = 14,
    UnknownCritical = 15,
    ArchitectureMismatch = 18,
    InvalidMemoryMap = 20,
    NoncanonicalOrder = 28,
}

#[derive(Debug)]
struct Bundle<'a> {
    expected: u16,
    trusted_base: u64,
    trusted_size: u64,
    handoff: &'a [u8],
    map: &'a [u8],
    rhd: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
struct Semantic {
    record_kinds: Vec<u16>,
    memory: Vec<(u64, u64, u16, u16, u16, u32)>,
}

fn take<'a>(bytes: &'a [u8], offset: &mut usize, length: usize) -> Option<&'a [u8]> {
    let end = offset.checked_add(length)?;
    let value = bytes.get(*offset..end)?;
    *offset = end;
    Some(value)
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(bytes.get(offset..offset.checked_add(2)?)?.try_into().ok()?))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(bytes.get(offset..offset.checked_add(4)?)?.try_into().ok()?))
}

fn u64_at(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(bytes.get(offset..offset.checked_add(8)?)?.try_into().ok()?))
}

fn parse_bundle(bytes: &[u8]) -> Option<Bundle<'_>> {
    if bytes.get(0..8)? != BUNDLE_MAGIC.as_slice() || u16_at(bytes, 8)? != 1 || u32_at(bytes, 24)? != 0 {
        return None;
    }
    let expected = u16_at(bytes, 10)?;
    let handoff_length = usize::try_from(u32_at(bytes, 12)?).ok()?;
    let map_length = usize::try_from(u32_at(bytes, 16)?).ok()?;
    let rhd_length = usize::try_from(u32_at(bytes, 20)?).ok()?;
    let trusted_base = u64_at(bytes, 28)?;
    let trusted_size = u64_at(bytes, 36)?;
    let mut offset = 44;
    let handoff = take(bytes, &mut offset, handoff_length)?;
    let map = take(bytes, &mut offset, map_length)?;
    let rhd = take(bytes, &mut offset, rhd_length)?;
    (offset == bytes.len()).then_some(Bundle { expected, trusted_base, trusted_size, handoff, map, rhd })
}

fn covered(base: u64, length: u64, trusted_base: u64, trusted_size: u64) -> Result<bool, Code> {
    let end = base.checked_add(length).ok_or(Code::RangeOverflow)?;
    let trusted_end = trusted_base.checked_add(trusted_size).ok_or(Code::RangeOverflow)?;
    Ok(base >= trusted_base && end <= trusted_end)
}

fn ranges_overlap(first: (u64, u64), second: (u64, u64)) -> Result<bool, Code> {
    let first_end = first.0.checked_add(first.1).ok_or(Code::RangeOverflow)?;
    let second_end = second.0.checked_add(second.1).ok_or(Code::RangeOverflow)?;
    Ok(first.0 < second_end && second.0 < first_end)
}

fn validate(bundle: &Bundle<'_>) -> Result<Semantic, Code> {
    let handoff = bundle.handoff;
    if handoff.len() < 128 {
        return Err(Code::Truncated);
    }
    if handoff.get(0..8) != Some(BOOT_MAGIC.as_slice()) {
        return Err(Code::BadMagic);
    }
    if u16_at(handoff, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(handoff, 10) != Some(0) {
        return Err(Code::UnsupportedMinor);
    }
    if u16_at(handoff, 12) != Some(128) {
        return Err(Code::BadHeaderSize);
    }
    if u32_at(handoff, 16).ok_or(Code::Truncated)? > 4096 {
        return Err(Code::Oversized);
    }
    if u16_at(handoff, 14) != Some(0) {
        return Err(Code::UnsupportedFlags);
    }
    if handoff.get(26..32) != Some([0; 6].as_slice()) || handoff.get(62..64) != Some([0; 2].as_slice()) || handoff.get(108..128) != Some([0; 20].as_slice()) {
        return Err(Code::NonzeroReserved);
    }
    let handoff_arch = u16_at(handoff, 20).ok_or(Code::Truncated)?;
    if !matches!(handoff_arch, 1 | 2) {
        return Err(Code::ArchitectureMismatch);
    }
    let rhd_address = u64_at(handoff, 48).ok_or(Code::Truncated)?;
    let rhd_claimed = u64::from(u32_at(handoff, 56).ok_or(Code::Truncated)?);
    if rhd_claimed > 65536 {
        return Err(Code::Oversized);
    }
    if rhd_address % 8 != 0 {
        return Err(Code::BadAlignment);
    }
    if !covered(rhd_address, rhd_claimed, bundle.trusted_base, bundle.trusted_size)? {
        return Err(Code::InvalidPointerRange);
    }
    if bundle.map.len() != 64 {
        return Err(Code::InvalidMemoryMap);
    }

    let rhd = bundle.rhd;
    if rhd.len() < 32 {
        return Err(Code::Truncated);
    }
    if rhd.get(0..8) != Some(RHD_MAGIC.as_slice()) {
        return Err(Code::BadMagic);
    }
    if u16_at(rhd, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(rhd, 10) != Some(0) {
        return Err(Code::UnsupportedMinor);
    }
    if u16_at(rhd, 12) != Some(32) || u16_at(rhd, 14) != Some(16) {
        return Err(Code::BadHeaderSize);
    }
    let total = usize::try_from(u32_at(rhd, 16).ok_or(Code::Truncated)?).map_err(|_| Code::Oversized)?;
    if total > 65536 {
        return Err(Code::Oversized);
    }
    if total != rhd.len() || u64::try_from(total).ok() != Some(rhd_claimed) {
        return Err(Code::BadCountOrLength);
    }
    if u16_at(rhd, 22) != Some(0) || rhd.get(28..32) != Some([0; 4].as_slice()) {
        return Err(Code::NonzeroReserved);
    }
    let rhd_arch = u16_at(rhd, 24).ok_or(Code::Truncated)?;
    let record_count = usize::from(u16_at(rhd, 20).ok_or(Code::Truncated)?);
    if record_count == 0 || record_count > 256 {
        return Err(Code::BadCountOrLength);
    }

    let mut offset = 32usize;
    let mut previous = None;
    let mut kinds = Vec::with_capacity(record_count);
    let mut memory = Vec::new();
    for _ in 0..record_count {
        let header_end = offset.checked_add(16).ok_or(Code::RangeOverflow)?;
        if header_end > total {
            return Err(Code::Truncated);
        }
        let kind = u16_at(rhd, offset).ok_or(Code::Truncated)?;
        let flags = u16_at(rhd, offset + 2).ok_or(Code::Truncated)?;
        let record_bytes = usize::try_from(u32_at(rhd, offset + 4).ok_or(Code::Truncated)?).map_err(|_| Code::Oversized)?;
        let record_id = u32_at(rhd, offset + 8).ok_or(Code::Truncated)?;
        if record_bytes < 16 || record_bytes % 8 != 0 {
            return Err(Code::BadAlignment);
        }
        let end = offset.checked_add(record_bytes).ok_or(Code::RangeOverflow)?;
        if end > total {
            return Err(Code::Truncated);
        }
        if flags & !1 != 0 {
            return Err(Code::UnsupportedFlags);
        }
        if u32_at(rhd, offset + 12) != Some(0) {
            return Err(Code::NonzeroReserved);
        }
        if let Some(last) = previous {
            if (kind, record_id) <= last {
                return Err(Code::NoncanonicalOrder);
            }
        }
        previous = Some((kind, record_id));
        if kind > 6 && flags & 1 != 0 {
            return Err(Code::UnknownCritical);
        }
        if kind == 2 {
            if record_bytes != 48 {
                return Err(Code::BadCountOrLength);
            }
            memory.push((
                u64_at(rhd, offset + 16).ok_or(Code::Truncated)?,
                u64_at(rhd, offset + 24).ok_or(Code::Truncated)?,
                u16_at(rhd, offset + 32).ok_or(Code::Truncated)?,
                u16_at(rhd, offset + 34).ok_or(Code::Truncated)?,
                u16_at(rhd, offset + 36).ok_or(Code::Truncated)?,
                u32_at(rhd, offset + 40).ok_or(Code::Truncated)?,
            ));
        }
        kinds.push(kind);
        offset = end;
    }
    if offset != total {
        return Err(Code::BadCountOrLength);
    }
    for first in 0..memory.len() {
        for second in first + 1..memory.len() {
            if ranges_overlap((memory[first].0, memory[first].1), (memory[second].0, memory[second].1))? {
                return Err(Code::Overlap);
            }
        }
    }
    if handoff_arch != rhd_arch {
        return Err(Code::ArchitectureMismatch);
    }
    Ok(Semantic { record_kinds: kinds, memory })
}

fn expected_name(code: u16) -> &'static str {
    match code {
        0 => "ok",
        1 => "truncated",
        2 => "oversized",
        9 => "bad-alignment",
        12 => "invalid-pointer-range",
        13 => "overlap",
        15 => "unknown-critical",
        18 => "architecture-mismatch",
        _ => "unmapped",
    }
}

fn main() {
    let root = PathBuf::from(env::args_os().nth(1).expect("fixture root argument"));
    let manifest = fs::read_to_string(root.join("cases.v1")).expect("read cases.v1");
    let mut failures = 0usize;
    let mut count = 0usize;
    let mut x86_semantic = None;
    let mut arm_semantic = None;

    for line in manifest.lines() {
        if line.is_empty() || line.starts_with("schema=") || line.starts_with("id|") {
            continue;
        }
        let fields: Vec<_> = line.split('|').collect();
        if fields.len() != 3 {
            eprintln!("malformed manifest row: {line}");
            failures += 1;
            continue;
        }
        let id = fields[0];
        let bytes = fs::read(root.join("bin").join(Path::new(fields[2]))).expect("read binary fixture");
        let Some(bundle) = parse_bundle(&bytes) else {
            eprintln!("{id}: malformed test-only binary bundle");
            failures += 1;
            continue;
        };
        let result = validate(&bundle);
        let actual = result.as_ref().map_or_else(|code| *code as u16, |_| Code::Ok as u16);
        if bundle.expected != actual || fields[1] != expected_name(actual) {
            eprintln!("{id}: manifest/bundle expected {}, got {}", fields[1], expected_name(actual));
            failures += 1;
        }
        if id == "valid-x86_64" {
            x86_semantic = result.ok();
        } else if id == "valid-aarch64" {
            arm_semantic = result.ok();
        }
        count += 1;
    }

    if count != 12 || x86_semantic.is_none() || x86_semantic != arm_semantic {
        eprintln!("fixture count or computed architecture semantics are invalid");
        failures += 1;
    }
    if failures != 0 {
        std::process::exit(1);
    }
    println!("R0-002 binary conformance passed: {count} fixtures");
}
