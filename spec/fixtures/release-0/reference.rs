#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

const BUNDLE_MAGIC: &[u8; 8] = b"R0FXBIN\0";
const ENTRY_MAGIC: &[u8; 8] = b"RARENTRY";
const BOOT_MAGIC: &[u8; 8] = b"RARBOOT\0";
const RHD_MAGIC: &[u8; 8] = b"RARRHD\0\0";

#[allow(dead_code)]
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
    OutOfAddressRange = 11,
    InvalidPointerRange = 12,
    Overlap = 13,
    BadCountOrLength = 14,
    UnknownCritical = 15,
    DuplicateId = 16,
    BadReference = 17,
    ArchitectureMismatch = 18,
    PageSizeMismatch = 19,
    InvalidMemoryMap = 20,
    InvalidCpuSet = 21,
    InvalidInterrupt = 22,
    InvalidTimer = 23,
    InvalidSerial = 24,
    InvalidBootSource = 25,
    InvalidEntropy = 26,
    InvalidTrace = 27,
    NoncanonicalOrder = 28,
    InconsistentDescription = 29,
    InvalidEntry = 30,
    SnapshotViolation = 31,
    InvalidRegisterWindow = 32,
    UnauthorizedDeviceWindow = 33,
}

#[derive(Debug)]
struct Bundle<'a> {
    expected: u16,
    copy_fault: bool,
    entry: &'a [u8],
    handoff: &'a [u8],
    map: &'a [u8],
    rhd: &'a [u8],
}

#[derive(Clone, Debug)]
struct Descriptor {
    base: u64,
    length: u64,
    purpose: u16,
    rights: u16,
    producer: u16,
    transfer: u16,
    owner_kind: u16,
    flags: u16,
    owner_id: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Memory {
    id: u32,
    base: u64,
    length: u64,
    kind: u16,
    attributes: u16,
    owner: u16,
}

#[derive(Clone, Debug)]
struct Window {
    parent_kind: u16,
    role: u16,
    parent_id: u32,
    space: u16,
    width: u8,
    byte_order: u8,
    stride: u16,
    flags: u16,
    base: u64,
    length: u64,
    authority: u16,
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

fn take<'a>(bytes: &'a [u8], offset: &mut usize, length: usize) -> Option<&'a [u8]> {
    let end = offset.checked_add(length)?;
    let value = bytes.get(*offset..end)?;
    *offset = end;
    Some(value)
}

fn parse_bundle(bytes: &[u8]) -> Option<Bundle<'_>> {
    if bytes.get(0..8)? != BUNDLE_MAGIC || u16_at(bytes, 8)? != 1 || u32_at(bytes, 28)? != 0 {
        return None;
    }
    let mut offset = 40usize;
    let entry = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 12)?).ok()?)?;
    let handoff = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 16)?).ok()?)?;
    let map = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 20)?).ok()?)?;
    let rhd = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 24)?).ok()?)?;
    (offset == bytes.len()).then_some(Bundle {
        expected: u16_at(bytes, 10)?,
        copy_fault: u32_at(bytes, 32)? != 0,
        entry,
        handoff,
        map,
        rhd,
    })
}

fn checked_end(base: u64, length: u64) -> Result<u64, Code> {
    base.checked_add(length).ok_or(Code::RangeOverflow)
}

fn overlaps(a: (u64, u64), b: (u64, u64)) -> Result<bool, Code> {
    Ok(a.0 < checked_end(b.0, b.1)? && b.0 < checked_end(a.0, a.1)?)
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) { bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes()); }
fn put_u32(bytes: &mut [u8], offset: usize, value: u32) { bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes()); }
fn put_u64(bytes: &mut [u8], offset: usize, value: u64) { bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes()); }

fn offsets(bytes: &[u8]) -> (usize, usize, usize, usize) {
    let entry = 40usize;
    let handoff = entry + usize::try_from(u32_at(bytes, 12).unwrap_or(0)).unwrap_or(0);
    let map = handoff + usize::try_from(u32_at(bytes, 16).unwrap_or(0)).unwrap_or(0);
    let rhd = map + usize::try_from(u32_at(bytes, 20).unwrap_or(0)).unwrap_or(0);
    (entry, handoff, map, rhd)
}

fn mutate(mut bytes: Vec<u8>, predicate: &str) -> Vec<u8> {
    let (entry, handoff, map, rhd) = offsets(&bytes);
    let descriptor = |index: usize| entry + 64 + index * 32;
    match predicate {
        "external-bounds" => { let old = usize::try_from(u32_at(&bytes, 12).unwrap()).unwrap(); bytes.drain(entry + 63..entry + old); put_u32(&mut bytes, 12, 63); }
        "external-ceiling" => { bytes.splice(entry + 288..entry + 288, vec![0; 3809]); put_u32(&mut bytes, 12, 4097); put_u32(&mut bytes, entry + 16, 4097); }
        "magic" => bytes[entry] ^= 0xff,
        "major-version" => put_u16(&mut bytes, entry + 8, 2),
        "fixed-sizes" => put_u16(&mut bytes, entry + 12, 63),
        "flags" => bytes[entry + 23] = 1,
        "reserved" => bytes[entry + 32] = 1,
        "range-arithmetic" => { put_u64(&mut bytes, descriptor(0), u64::MAX); put_u64(&mut bytes, descriptor(0) + 8, 2); }
        "address-width" => { bytes[entry + 22] = 32; }
        "alignment" => put_u64(&mut bytes, descriptor(1), 8193),
        "entry-semantics" => put_u16(&mut bytes, entry + 24, 6),
        "minor-compatibility" => { put_u16(&mut bytes, entry + 10, 1); put_u16(&mut bytes, descriptor(5) + 16, 99); put_u16(&mut bytes, descriptor(5) + 26, 4); }
        "descriptor-binding" => put_u64(&mut bytes, descriptor(1) + 8, 95),
        "range-aliasing" => put_u64(&mut bytes, descriptor(1), 4096),
        "snapshot-acquisition" => put_u32(&mut bytes, 32, 1),
        "counts-and-lengths" => put_u32(&mut bytes, handoff + 16, 127),
        "critical-extension" => { put_u16(&mut bytes, rhd + 368, 99); put_u16(&mut bytes, rhd + 370, 1); }
        "identity-uniqueness" => put_u32(&mut bytes, rhd + 456, 1),
        "references" => put_u32(&mut bytes, rhd + 64, 99),
        "memory-map" => put_u64(&mut bytes, map + 8, 0),
        "cpu-records" => put_u32(&mut bytes, rhd + 48, 0),
        "interrupt-records" => put_u32(&mut bytes, rhd + 248, 0),
        "timer-records" => put_u64(&mut bytes, rhd + 296, 0),
        "serial-records" => put_u32(&mut bytes, rhd + 340, 224),
        "boot-source" => put_u16(&mut bytes, rhd + 384, 2),
        "entropy" => put_u32(&mut bytes, handoff + 72, 31),
        "trace" => put_u32(&mut bytes, handoff + 96, 4095),
        "canonical-order" => { put_u32(&mut bytes, rhd + 136, 3); put_u32(&mut bytes, rhd + 184, 2); }
        "map-rhd-consistency" => put_u64(&mut bytes, rhd + 144, 1_572_864),
        "register-window" => put_u16(&mut bytes, rhd + 420, 4),
        "device-authority" => put_u16(&mut bytes, rhd + 440, 6),
        "architecture" => put_u16(&mut bytes, rhd + 24, 2),
        "page-size" => bytes[rhd + 27] = 13,
        _ => panic!("unknown predicate mutation: {predicate}"),
    }
    bytes
}

fn poison_nested(bytes: &mut [u8]) {
    let (_, handoff, _, rhd) = offsets(bytes);
    if handoff < bytes.len() { bytes[handoff] ^= 0x5a; }
    if rhd < bytes.len() { bytes[rhd] ^= 0xa5; }
}

fn validate(bundle: &Bundle<'_>) -> Result<(), Code> {
    let entry = bundle.entry;
    if entry.len() < 64 {
        return Err(Code::Truncated);
    }
    let entry_total = usize::try_from(u32_at(entry, 16).ok_or(Code::Truncated)?).map_err(|_| Code::Oversized)?;
    if entry_total > 4096 || entry.len() > 4096 {
        return Err(Code::Oversized);
    }
    if entry.get(0..8) != Some(ENTRY_MAGIC) {
        return Err(Code::BadMagic);
    }
    if u16_at(entry, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(entry, 12) != Some(64) || u16_at(entry, 14) != Some(32) {
        return Err(Code::BadHeaderSize);
    }
    if entry.get(23) != Some(&0) {
        return Err(Code::UnsupportedFlags);
    }
    if entry.get(26..64) != Some([0; 38].as_slice()) {
        return Err(Code::NonzeroReserved);
    }

    let count = usize::from(u16_at(entry, 24).ok_or(Code::Truncated)?);
    let expected_entry_total = 64usize.checked_add(count.checked_mul(32).ok_or(Code::RangeOverflow)?).ok_or(Code::RangeOverflow)?;
    let address_bits = *entry.get(22).ok_or(Code::Truncated)?;
    if !(32..=64).contains(&address_bits) {
        return Err(Code::OutOfAddressRange);
    }
    let mut descriptors = Vec::with_capacity(count);
    for index in 0..count {
        let offset = 64usize.checked_add(index.checked_mul(32).ok_or(Code::RangeOverflow)?).ok_or(Code::RangeOverflow)?;
        let base = u64_at(entry, offset).ok_or(Code::InvalidEntry)?;
        let length = u64_at(entry, offset + 8).ok_or(Code::InvalidEntry)?;
        let end = checked_end(base, length)?;
        if address_bits < 64 && end > (1u64 << address_bits) {
            return Err(Code::OutOfAddressRange);
        }
        let purpose = u16_at(entry, offset + 16).ok_or(Code::InvalidEntry)?;
        if purpose != 7 && base % 8 != 0 {
            return Err(Code::BadAlignment);
        }
        descriptors.push(Descriptor {
            base,
            length,
            purpose,
            rights: u16_at(entry, offset + 18).ok_or(Code::InvalidEntry)?,
            producer: u16_at(entry, offset + 20).ok_or(Code::InvalidEntry)?,
            transfer: u16_at(entry, offset + 22).ok_or(Code::InvalidEntry)?,
            owner_kind: u16_at(entry, offset + 24).ok_or(Code::InvalidEntry)?,
            flags: u16_at(entry, offset + 26).ok_or(Code::InvalidEntry)?,
            owner_id: u32_at(entry, offset + 28).ok_or(Code::InvalidEntry)?,
        });
    }
    if count == 0 || count > 126 || entry_total != entry.len() || entry_total != expected_entry_total {
        return Err(Code::InvalidEntry);
    }
    for purpose in 1..=5 {
        if descriptors.iter().filter(|d| d.purpose == purpose).count() != 1 {
            return Err(Code::InvalidEntry);
        }
    }
    if u16_at(entry, 10).unwrap_or(0) > 0 && descriptors.iter().any(|item| item.purpose > 7 && item.flags & 4 != 0) { return Err(Code::UnsupportedMinor); }
    for d in &descriptors {
        let valid = match d.purpose {
            1..=3 => d.length != 0 && d.rights == 1 && d.transfer == 1 && d.flags == 3 && matches!(d.producer, 1 | 2),
            4 => d.length != 0 && ((d.rights == 1 && d.transfer == 1) || (d.rights == 3 && d.transfer == 4)) && d.flags == 3 && matches!(d.producer, 1 | 2),
            5 => d.length != 0 && d.rights == 3 && d.transfer == 2 && d.flags == 3 && matches!(d.producer, 1 | 2),
            6 | 7 => d.length != 0 && d.rights == 11 && d.transfer == 3 && d.flags == 0 && matches!(d.producer, 1 | 2) && matches!(d.owner_kind, 3 | 5),
            _ => false,
        };
        if !valid {
            return Err(Code::InvalidEntry);
        }
    }
    let descriptor = |purpose| descriptors.iter().find(|item| item.purpose == purpose).ok_or(Code::InvalidPointerRange);
    let handoff_descriptor = descriptor(1)?;
    let map_descriptor = descriptor(2)?;
    let rhd_descriptor = descriptor(3)?;
    let entropy_descriptor = descriptor(4)?;
    let trace_descriptor = descriptor(5)?;
    if handoff_descriptor.length != u64::try_from(bundle.handoff.len()).unwrap_or(u64::MAX)
        || map_descriptor.length != u64::try_from(bundle.map.len()).unwrap_or(u64::MAX)
        || rhd_descriptor.length != u64::try_from(bundle.rhd.len()).unwrap_or(u64::MAX)
    {
        return Err(Code::InvalidPointerRange);
    }
    for first in 0..descriptors.len() {
        for second in first + 1..descriptors.len() {
            let same_space = (descriptors[first].purpose == 7) == (descriptors[second].purpose == 7);
            if same_space && overlaps((descriptors[first].base, descriptors[first].length), (descriptors[second].base, descriptors[second].length))? {
                return Err(Code::Overlap);
            }
        }
    }
    for item in &descriptors {
        if item.purpose != 7 && overlaps((0x1000_0000, u64::try_from(entry.len()).unwrap_or(u64::MAX)), (item.base, item.length))? { return Err(Code::Overlap); }
    }
    if bundle.copy_fault {
        return Err(Code::SnapshotViolation);
    }

    let handoff = bundle.handoff;
    let rhd = bundle.rhd;
    if handoff.get(0..8) != Some(BOOT_MAGIC) || rhd.get(0..8) != Some(RHD_MAGIC) {
        return Err(Code::BadMagic);
    }
    if u16_at(handoff, 8) != Some(1) || u16_at(rhd, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(handoff, 12) != Some(128) || u16_at(rhd, 12) != Some(32) || u16_at(rhd, 14) != Some(16) {
        return Err(Code::BadHeaderSize);
    }
    if u16_at(handoff, 14) != Some(0) || u16_at(rhd, 22) != Some(0) {
        return Err(Code::UnsupportedFlags);
    }
    if handoff.get(26..32) != Some([0; 6].as_slice()) || handoff.get(108..128) != Some([0; 20].as_slice()) || rhd.get(28..32) != Some([0; 4].as_slice()) {
        return Err(Code::NonzeroReserved);
    }
    if handoff.len() != 128 || u32_at(handoff, 16) != Some(128) || u32_at(handoff, 44) != Some(0x0001_0020) || rhd.len() < 32 {
        return Err(Code::BadCountOrLength);
    }
    let map_count = usize::try_from(u32_at(handoff, 40).ok_or(Code::BadCountOrLength)?).map_err(|_| Code::BadCountOrLength)?;
    if map_count.checked_mul(32) != Some(bundle.map.len()) || u64_at(handoff, 32) != Some(map_descriptor.base) || u32_at(handoff, 56) != u32::try_from(rhd.len()).ok() || u64_at(handoff, 48) != Some(rhd_descriptor.base) || u64_at(handoff, 64) != Some(entropy_descriptor.base) || u64_at(handoff, 88) != Some(trace_descriptor.base) {
        return Err(Code::BadCountOrLength);
    }
    let rhd_total = usize::try_from(u32_at(rhd, 16).ok_or(Code::BadCountOrLength)?).map_err(|_| Code::BadCountOrLength)?;
    let record_count = usize::from(u16_at(rhd, 20).ok_or(Code::BadCountOrLength)?);
    if rhd_total != rhd.len() || rhd_total > 65536 || record_count == 0 || record_count > 256 {
        return Err(Code::BadCountOrLength);
    }

    let mut offset = 32usize;
    let mut raw_records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        let kind = u16_at(rhd, offset).ok_or(Code::Truncated)?;
        let flags = u16_at(rhd, offset + 2).ok_or(Code::Truncated)?;
        let size = usize::try_from(u32_at(rhd, offset + 4).ok_or(Code::Truncated)?).map_err(|_| Code::BadCountOrLength)?;
        let id = u32_at(rhd, offset + 8).ok_or(Code::Truncated)?;
        if size < 16 || size % 8 != 0 || offset.checked_add(size).filter(|end| *end <= rhd.len()).is_none() {
            return Err(Code::BadCountOrLength);
        }
        if kind > 7 && flags & 1 != 0 {
            return Err(Code::UnknownCritical);
        }
        let expected_size = match kind { 1..=5 => Some(48), 6 => Some(24), 7 => Some(56), _ => None };
        if expected_size.is_some_and(|expected| expected != size) || flags & !1 != 0 || u32_at(rhd, offset + 12) != Some(0) {
            return Err(Code::BadCountOrLength);
        }
        let reserved_zero = match kind {
            1 => rhd.get(offset + 20..offset + 24) == Some([0; 4].as_slice()) && rhd.get(offset + 40..offset + 48) == Some([0; 8].as_slice()),
            2 => rhd.get(offset + 38..offset + 48) == Some([0; 10].as_slice()),
            3 => rhd.get(offset + 28..offset + 48) == Some([0; 20].as_slice()),
            4 => rhd.get(offset + 20..offset + 24) == Some([0; 4].as_slice()) && rhd.get(offset + 42..offset + 48) == Some([0; 6].as_slice()),
            5 => rhd.get(offset + 28..offset + 32) == Some([0; 4].as_slice()) && rhd.get(offset + 42..offset + 48) == Some([0; 6].as_slice()),
            6 => rhd.get(offset + 20..offset + 24) == Some([0; 4].as_slice()),
            7 => rhd.get(offset + 50..offset + 56) == Some([0; 6].as_slice()),
            _ => true,
        };
        if !reserved_zero { return Err(Code::NonzeroReserved); }
        raw_records.push((kind, id, offset, size));
        offset += size;
    }
    if offset != rhd.len() {
        return Err(Code::BadCountOrLength);
    }

    let mut identities = BTreeSet::new();
    for &(kind, id, _, _) in &raw_records {
        if !identities.insert((kind, id)) {
            return Err(Code::DuplicateId);
        }
    }
    let by_kind: BTreeMap<u16, BTreeSet<u32>> = raw_records.iter().fold(BTreeMap::new(), |mut map, &(kind, id, _, _)| {
        map.entry(kind).or_default().insert(id);
        map
    });
    let cpu = raw_records.iter().find(|r| r.0 == 1).ok_or(Code::BadReference)?;
    let cpu_interrupt = u32_at(rhd, cpu.2 + 32).ok_or(Code::BadReference)?;
    let cpu_timer = u32_at(rhd, cpu.2 + 36).ok_or(Code::BadReference)?;
    if !by_kind.get(&3).is_some_and(|ids| ids.contains(&cpu_interrupt)) || !by_kind.get(&4).is_some_and(|ids| ids.contains(&cpu_timer)) {
        return Err(Code::BadReference);
    }
    for &(kind, _, at, _) in &raw_records {
        if matches!(kind, 4 | 5) && !by_kind.get(&3).is_some_and(|ids| ids.contains(&u32_at(rhd, at + if kind == 4 { 36 } else { 24 }).unwrap_or(0))) {
            return Err(Code::BadReference);
        }
        if kind == 7 {
            let parent_kind = u16_at(rhd, at + 16).ok_or(Code::BadReference)?;
            let parent_id = u32_at(rhd, at + 20).ok_or(Code::BadReference)?;
            if !by_kind.get(&parent_kind).is_some_and(|ids| ids.contains(&parent_id)) {
                return Err(Code::BadReference);
            }
        }
    }

    let mut map_memory = Vec::with_capacity(map_count);
    for index in 0..map_count {
        let at = index * 32;
        let memory = Memory {
            id: u32_at(bundle.map, at + 24).ok_or(Code::InvalidMemoryMap)?,
            base: u64_at(bundle.map, at).ok_or(Code::InvalidMemoryMap)?,
            length: u64_at(bundle.map, at + 8).ok_or(Code::InvalidMemoryMap)?,
            kind: u16_at(bundle.map, at + 16).ok_or(Code::InvalidMemoryMap)?,
            attributes: u16_at(bundle.map, at + 18).ok_or(Code::InvalidMemoryMap)?,
            owner: u16_at(bundle.map, at + 20).ok_or(Code::InvalidMemoryMap)?,
        };
        let combination = match memory.kind {
            1 => memory.attributes == 11 && memory.owner == 0,
            2 => memory.attributes == 9 && memory.owner == 4,
            3 => memory.attributes == 11 && matches!(memory.owner, 1 | 2),
            4 => memory.attributes == 11 && memory.owner == 3,
            5 => memory.attributes == 19 && memory.owner == 5,
            6 => memory.attributes == 0 && memory.owner == 0,
            _ => false,
        };
        if memory.length == 0 || checked_end(memory.base, memory.length).is_err() || !combination || bundle.map.get(at + 22..at + 24) != Some([0; 2].as_slice()) || bundle.map.get(at + 28..at + 32) != Some([0; 4].as_slice()) {
            return Err(Code::InvalidMemoryMap);
        }
        if map_memory.last().is_some_and(|previous: &Memory| previous.base >= memory.base || checked_end(previous.base, previous.length).is_ok_and(|end| end > memory.base)) {
            return Err(Code::InvalidMemoryMap);
        }
        map_memory.push(memory);
    }
    for source in descriptors.iter().filter(|item| (1..=5).contains(&item.purpose)) {
        let expected_owner = source.producer;
        if !map_memory.iter().any(|memory| memory.kind == 3 && memory.owner == expected_owner && memory.base <= source.base && checked_end(memory.base, memory.length).is_ok_and(|end| checked_end(source.base, source.length).is_ok_and(|source_end| source_end <= end))) { return Err(Code::InvalidPointerRange); }
    }

    if by_kind.get(&1).map_or(0, BTreeSet::len) == 0 || raw_records.iter().filter(|r| r.0 == 1 && u32_at(rhd, r.2 + 16).is_some_and(|flags| flags == 1)).count() != 1 || raw_records.iter().filter(|r| r.0 == 1).any(|r| u32_at(rhd, r.2 + 16).is_none_or(|flags| flags & !1 != 0)) || cpu.1 != u32_at(handoff, 104).unwrap_or(0) {
        return Err(Code::InvalidCpuSet);
    }
    let mut controllers = BTreeMap::new();
    for record in raw_records.iter().filter(|r| r.0 == 3) {
        let model = u16_at(rhd, record.2 + 16).ok_or(Code::InvalidInterrupt)?;
        let count = u32_at(rhd, record.2 + 24).ok_or(Code::InvalidInterrupt)?;
        if !matches!(model, 1 | 2) || count == 0 || u16_at(rhd, record.2 + 18) != Some(0) { return Err(Code::InvalidInterrupt); }
        controllers.insert(record.1, (model, count));
    }
    for timer in raw_records.iter().filter(|r| r.0 == 4) {
        let controller_id = u32_at(rhd, timer.2 + 36).unwrap_or(0);
        let count = controllers.get(&controller_id).map(|value| value.1).ok_or(Code::InvalidTimer)?;
        if u16_at(rhd, timer.2 + 18) != Some(0) || u64_at(rhd, timer.2 + 24).unwrap_or(0) == 0 || u32_at(rhd, timer.2 + 32).unwrap_or(u32::MAX) >= count || !rhd.get(timer.2 + 40).is_some_and(|value| matches!(*value, 1 | 2)) || !rhd.get(timer.2 + 41).is_some_and(|value| matches!(*value, 1 | 2)) { return Err(Code::InvalidTimer); }
    }
    for serial in raw_records.iter().filter(|r| r.0 == 5) {
        let model = u16_at(rhd, serial.2 + 16).ok_or(Code::InvalidSerial)?;
        let controller_id = u32_at(rhd, serial.2 + 24).unwrap_or(0);
        let count = controllers.get(&controller_id).map(|value| value.1).ok_or(Code::InvalidSerial)?;
        if !matches!(model, 1 | 2) || u16_at(rhd, serial.2 + 18) != Some(0) || u32_at(rhd, serial.2 + 20).unwrap_or(u32::MAX) >= count || !rhd.get(serial.2 + 40).is_some_and(|value| matches!(*value, 1 | 2)) || !rhd.get(serial.2 + 41).is_some_and(|value| matches!(*value, 1 | 2)) { return Err(Code::InvalidSerial); }
    }
    if by_kind.get(&6).map_or(0, BTreeSet::len) == 0 {
        return Err(Code::InvalidBootSource);
    }
    let boot_source = raw_records.iter().find(|record| record.0 == 6).ok_or(Code::InvalidBootSource)?;
    if u16_at(rhd, boot_source.2 + 16) != Some(1) || u16_at(rhd, boot_source.2 + 18) != Some(0) || u16_at(handoff, 24) != Some(1) { return Err(Code::InvalidBootSource); }
    let entropy_flags = u32_at(handoff, 76).ok_or(Code::InvalidEntropy)?;
    if entropy_flags & !1 != 0 || (entropy_flags == 0 && (entropy_descriptor.rights != 1 || entropy_descriptor.transfer != 1)) || (entropy_flags == 1 && (entropy_descriptor.rights != 3 || entropy_descriptor.transfer != 4)) { return Err(Code::InvalidEntropy); }
    if !matches!(u32_at(handoff, 72), Some(32..=64)) || u64::from(u32_at(handoff, 72).unwrap_or(0)) != entropy_descriptor.length {
        return Err(Code::InvalidEntropy);
    }
    if !matches!(u32_at(handoff, 96), Some(4096..=1_048_576)) || u64::from(u32_at(handoff, 96).unwrap_or(0)) != trace_descriptor.length {
        return Err(Code::InvalidTrace);
    }
    if u16_at(handoff, 84) != Some(1) || u16_at(handoff, 86) != Some(0) || u32_at(handoff, 100) != Some(0) { return Err(Code::InvalidTrace); }
    if raw_records.windows(2).any(|pair| (pair[0].0, pair[0].1) >= (pair[1].0, pair[1].1)) {
        return Err(Code::NoncanonicalOrder);
    }

    let rhd_memory: Vec<Memory> = raw_records.iter().filter(|r| r.0 == 2).map(|r| Memory {
        id: r.1,
        base: u64_at(rhd, r.2 + 16).unwrap_or(0),
        length: u64_at(rhd, r.2 + 24).unwrap_or(0),
        kind: u16_at(rhd, r.2 + 32).unwrap_or(0),
        attributes: u16_at(rhd, r.2 + 34).unwrap_or(0),
        owner: u16_at(rhd, r.2 + 36).unwrap_or(0),
    }).collect();
    if rhd_memory != map_memory {
        return Err(Code::InconsistentDescription);
    }

    let windows: Vec<Window> = raw_records.iter().filter(|r| r.0 == 7).map(|r| Window {
        parent_kind: u16_at(rhd, r.2 + 16).unwrap_or(0),
        role: u16_at(rhd, r.2 + 18).unwrap_or(0),
        parent_id: u32_at(rhd, r.2 + 20).unwrap_or(0),
        space: u16_at(rhd, r.2 + 24).unwrap_or(0),
        width: *rhd.get(r.2 + 26).unwrap_or(&0),
        byte_order: *rhd.get(r.2 + 27).unwrap_or(&0),
        stride: u16_at(rhd, r.2 + 28).unwrap_or(0),
        flags: u16_at(rhd, r.2 + 30).unwrap_or(0),
        base: u64_at(rhd, r.2 + 32).unwrap_or(0),
        length: u64_at(rhd, r.2 + 40).unwrap_or(0),
        authority: u16_at(rhd, r.2 + 48).unwrap_or(u16::MAX),
    }).collect();
    let entry_arch = u16_at(entry, 20).ok_or(Code::InvalidRegisterWindow)?;
    let parent_models: BTreeMap<(u16, u32), u16> = raw_records.iter().filter(|record| matches!(record.0, 3 | 5)).map(|record| ((record.0, record.1), u16_at(rhd, record.2 + 16).unwrap_or(0))).collect();
    for window in &windows {
        let parent_model = parent_models.get(&(window.parent_kind, window.parent_id)).copied().unwrap_or(0);
        let common = window.byte_order == 1 && window.flags == 1 && window.length != 0 && window.width.is_power_of_two() && window.stride.is_power_of_two() && u16::from(window.width) <= window.stride && u64::from(window.width) <= window.length && u64::from(window.stride) <= window.length;
        let model_valid = match (window.parent_kind, parent_model, window.role, window.space, window.width, window.stride) {
            (3, 1, 1, 1, 4, 16) if entry_arch == 1 => true,
            (3, 2, 2 | 3, 1, 4, 4) if entry_arch == 2 => true,
            (5, 1, 5, 1 | 2, 1, 1) if entry_arch == 1 => true,
            (5, 2, 5, 1, 4, 4) if entry_arch == 2 => true,
            _ => false,
        };
        if !common || !model_valid || (window.space == 2 && checked_end(window.base, window.length).is_err()) || (window.space == 2 && checked_end(window.base, window.length).unwrap_or(u64::MAX) > 65_536) {
            return Err(Code::InvalidRegisterWindow);
        }
    }
    for (&parent, &model) in &parent_models {
        let roles: Vec<_> = windows.iter().filter(|window| (window.parent_kind, window.parent_id) == parent).map(|window| window.role).collect();
        let expected: &[u16] = match (parent.0, model) { (3, 1) => &[1], (3, 2) => &[2, 3], (5, _) => &[5], _ => &[] };
        if roles.len() != expected.len() || expected.iter().any(|role| roles.iter().filter(|actual| *actual == role).count() != 1) { return Err(Code::InvalidRegisterWindow); }
    }

    for window in &windows {
        let authority = descriptors.get(usize::from(window.authority)).ok_or(Code::UnauthorizedDeviceWindow)?;
        let purpose = if window.space == 1 { 6 } else { 7 };
        if authority.purpose != purpose || authority.base > window.base || checked_end(authority.base, authority.length).is_err() || checked_end(window.base, window.length).is_err() || checked_end(window.base, window.length).unwrap_or(u64::MAX) > checked_end(authority.base, authority.length).unwrap_or(0) || authority.owner_kind != window.parent_kind || authority.owner_id != window.parent_id || authority.rights != 11 {
            return Err(Code::UnauthorizedDeviceWindow);
        }
        if window.space == 1 && !map_memory.iter().any(|m| m.kind == 5 && m.owner == 5 && m.attributes == 19 && m.base <= window.base && checked_end(m.base, m.length).is_ok_and(|end| checked_end(window.base, window.length).is_ok_and(|window_end| window_end <= end))) {
            return Err(Code::UnauthorizedDeviceWindow);
        }
    }
    if entry_arch != u16_at(handoff, 20).unwrap_or(0) || entry_arch != u16_at(rhd, 24).unwrap_or(0) {
        return Err(Code::ArchitectureMismatch);
    }
    if *handoff.get(23).unwrap_or(&0) != *rhd.get(27).unwrap_or(&0) {
        return Err(Code::PageSizeMismatch);
    }
    Ok(())
}

fn expected_name(code: u16) -> &'static str {
    match code {
        0 => "ok", 1 => "truncated", 2 => "oversized", 3 => "bad-magic", 4 => "unsupported-major",
        5 => "unsupported-minor", 6 => "bad-header-size", 7 => "unsupported-flags", 8 => "nonzero-reserved",
        9 => "bad-alignment", 10 => "range-overflow", 11 => "out-of-address-range", 12 => "invalid-pointer-range",
        13 => "overlap", 14 => "bad-count-or-length", 15 => "unknown-critical", 16 => "duplicate-id",
        17 => "bad-reference", 18 => "architecture-mismatch", 19 => "page-size-mismatch", 20 => "invalid-memory-map",
        21 => "invalid-cpu-set", 22 => "invalid-interrupt", 23 => "invalid-timer", 24 => "invalid-serial",
        25 => "invalid-boot-source", 26 => "invalid-entropy", 27 => "invalid-trace", 28 => "noncanonical-order",
        29 => "inconsistent-description", 30 => "invalid-entry", 31 => "snapshot-violation",
        32 => "invalid-register-window", 33 => "unauthorized-device-window", _ => "unmapped",
    }
}

fn check_precedence(root: &Path) -> Result<(), String> {
    let fields = fs::read_to_string(root.join("../../boot/handoff-v1.fields")).map_err(|error| error.to_string())?;
    let fixture = fs::read_to_string(root.join("validation-precedence.v1")).map_err(|error| error.to_string())?;
    let predicates: Vec<_> = fields.lines().filter(|line| line.starts_with("validation-predicate|")).collect();
    if predicates.len() != 33 { return Err("canonical predicate count is not 33".into()); }
    let baseline = fs::read(root.join("bin/valid-x86_64.bin")).map_err(|error| error.to_string())?;
    for (index, row) in predicates.iter().enumerate() {
        let columns: Vec<_> = row.split('|').collect();
        let single_row = format!("single|{}|{}|{}", index + 1, columns[2], columns[3]);
        if !fixture.lines().any(|line| line == single_row) { return Err(format!("missing executable mutation declaration: {single_row}")); }
        let mut single = mutate(baseline.clone(), columns[2]);
        if index < 15 { poison_nested(&mut single); }
        let bundle = parse_bundle(&single).ok_or_else(|| format!("single mutation malformed bundle: {}", columns[2]))?;
        let actual = validate(&bundle).map_or_else(|code| code as u16, |()| 0);
        if expected_name(actual) != columns[3] { return Err(format!("single {} expected {}, got {}", columns[2], columns[3], expected_name(actual))); }
        if index + 1 < predicates.len() {
            let next: Vec<_> = predicates[index + 1].split('|').collect();
            let dual_row = format!("dual|{}|{}|{}", columns[2], next[2], columns[3]);
            if !fixture.lines().any(|line| line == dual_row) { return Err(format!("missing executable dual declaration: {dual_row}")); }
            let mut dual = mutate(mutate(baseline.clone(), next[2]), columns[2]);
            if index < 15 { poison_nested(&mut dual); }
            let bundle = parse_bundle(&dual).ok_or_else(|| format!("dual mutation malformed bundle: {} + {}", columns[2], next[2]))?;
            let actual = validate(&bundle).map_or_else(|code| code as u16, |()| 0);
            if expected_name(actual) != columns[3] { return Err(format!("dual {} + {} expected {}, got {}", columns[2], next[2], columns[3], expected_name(actual))); }
        }
    }
    Ok(())
}

fn main() {
    let root = PathBuf::from(env::args_os().nth(1).expect("fixture root argument"));
    let manifest = fs::read_to_string(root.join("cases.v1")).expect("read cases.v1");
    let mut failures = 0usize;
    let mut count = 0usize;
    for line in manifest.lines() {
        if line.is_empty() || line.starts_with("schema=") || line.starts_with("id|") { continue; }
        let fields: Vec<_> = line.split('|').collect();
        if fields.len() != 3 { eprintln!("malformed manifest row: {line}"); failures += 1; continue; }
        let bytes = fs::read(root.join("bin").join(Path::new(fields[2]))).expect("read binary fixture");
        let Some(bundle) = parse_bundle(&bytes) else { eprintln!("{}: malformed bundle", fields[0]); failures += 1; continue; };
        let actual = validate(&bundle).map_or_else(|code| code as u16, |()| Code::Ok as u16);
        if bundle.expected != actual || fields[1] != expected_name(actual) {
            eprintln!("{}: expected {}, got {}", fields[0], fields[1], expected_name(actual));
            failures += 1;
        }
        count += 1;
    }
    if let Err(error) = check_precedence(&root) { eprintln!("{error}"); failures += 1; }
    if count != 18 { eprintln!("fixture count is {count}, expected 18"); failures += 1; }
    if failures != 0 { std::process::exit(1); }
    println!("R0-002 binary conformance passed: {count} fixtures, 33 singles, 32 adjacent precedence edges");
}
