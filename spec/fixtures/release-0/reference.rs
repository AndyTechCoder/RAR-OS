#![forbid(unsafe_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

const BUNDLE_MAGIC: &[u8; 8] = b"R0FXBIN\0";
const ENTRY_MAGIC: &[u8; 8] = b"RARENTRY";
const BOOT_MAGIC: &[u8; 8] = b"RARBOOT\0";
const RHD_MAGIC: &[u8; 8] = b"RARRHD\0\0";
const ENTRY_ADDRESS: u64 = 0x1000_0000;
const BUNDLE_HEADER_BYTES: usize = 64;

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

#[derive(Clone, Copy, Debug)]
struct AdapterInputs {
    expected_architecture: u16,
    external_entry_address: u64,
    external_entry_bytes: u32,
    address_bits: u8,
    page_bytes: u64,
    entry_alignment: u16,
    stack_alignment: u16,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct DescriptorSelector {
    purpose: u16,
    owner_kind: u16,
    owner_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct AuthorityIdentity {
    selector: DescriptorSelector,
    base: u64,
    length: u64,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum AccessKey {
    Entry,
    Descriptor(DescriptorSelector),
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum Effect {
    ClearEntropy(DescriptorSelector),
    ActivateTrace(DescriptorSelector),
    ConstructAuthority(AuthorityIdentity),
}

#[derive(Clone, Debug)]
struct Descriptor {
    selector: DescriptorSelector,
    base: u64,
    length: u64,
    rights: u16,
    producer: u16,
    transfer: u16,
    flags: u16,
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
    authority_purpose: u16,
}

#[derive(Debug)]
struct Bundle<'a> {
    expected: u16,
    copy_fault: bool,
    adapter: AdapterInputs,
    entry: &'a [u8],
    handoff: &'a [u8],
    map: &'a [u8],
    rhd: &'a [u8],
}

#[derive(Clone, Copy, Debug, Default)]
struct Behavior {
    external_address: Option<u64>,
    external_length: Option<u32>,
    adapter_address_bits: Option<u8>,
    adapter_architecture: Option<u16>,
    page_bytes: Option<u64>,
    entry_alignment: Option<u16>,
    fault_purpose: Option<u16>,
    short_purpose: Option<u16>,
}

struct SourceProvider<'a> {
    entry: &'a [u8],
    handoff: &'a [u8],
    map: &'a [u8],
    rhd: &'a [u8],
    entropy: Vec<u8>,
    fault_purpose: Option<u16>,
    short_purpose: Option<u16>,
    reads: Vec<AccessKey>,
    copied: BTreeSet<AccessKey>,
}

impl<'a> SourceProvider<'a> {
    fn new(bundle: &'a Bundle<'a>, behavior: Behavior) -> Self {
        Self {
            entry: bundle.entry,
            handoff: bundle.handoff,
            map: bundle.map,
            rhd: bundle.rhd,
            entropy: vec![0x5a; 64],
            fault_purpose: behavior.fault_purpose.or(bundle.copy_fault.then_some(1)),
            short_purpose: behavior.short_purpose,
            reads: Vec::new(),
            copied: BTreeSet::new(),
        }
    }

    fn copy_entry(&mut self, expected: usize) -> Result<Vec<u8>, Code> {
        let data = self.entry.to_vec();
        self.copy(AccessKey::Entry, data, expected, false, false)
    }

    fn copy_descriptor(&mut self, descriptor: &Descriptor) -> Result<Vec<u8>, Code> {
        let data = match descriptor.selector.purpose {
            1 => self.handoff.to_vec(),
            2 => self.map.to_vec(),
            3 => self.rhd.to_vec(),
            4 => self.entropy[..usize::try_from(descriptor.length.min(64)).unwrap_or(0)].to_vec(),
            _ => return Err(Code::SnapshotViolation),
        };
        self.copy(
            AccessKey::Descriptor(descriptor.selector),
            data,
            usize::try_from(descriptor.length).map_err(|_| Code::SnapshotViolation)?,
            self.fault_purpose == Some(descriptor.selector.purpose),
            self.short_purpose == Some(descriptor.selector.purpose),
        )
    }

    fn copy(&mut self, key: AccessKey, mut data: Vec<u8>, expected: usize, fault: bool, short: bool) -> Result<Vec<u8>, Code> {
        self.reads.push(key);
        if !self.copied.insert(key) || fault {
            return Err(Code::SnapshotViolation);
        }
        if short && !data.is_empty() {
            data.pop();
        }
        if data.len() != expected {
            return Err(Code::SnapshotViolation);
        }
        Ok(data)
    }
}

#[derive(Default)]
struct EffectSink {
    effects: Vec<Effect>,
}

impl EffectSink {
    fn commit(&mut self, effect: Effect) {
        self.effects.push(effect);
    }
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

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn take<'a>(bytes: &'a [u8], offset: &mut usize, length: usize) -> Option<&'a [u8]> {
    let end = offset.checked_add(length)?;
    let value = bytes.get(*offset..end)?;
    *offset = end;
    Some(value)
}

fn parse_bundle(bytes: &[u8]) -> Option<Bundle<'_>> {
    if bytes.get(0..8)? != BUNDLE_MAGIC
        || u16_at(bytes, 8)? != 1
        || u32_at(bytes, 28)? != 0
        || bytes.get(39) != Some(&0)
        || u32_at(bytes, 60)? != 0
    {
        return None;
    }
    let mut offset = BUNDLE_HEADER_BYTES;
    let entry = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 12)?).ok()?)?;
    let handoff = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 16)?).ok()?)?;
    let map = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 20)?).ok()?)?;
    let rhd = take(bytes, &mut offset, usize::try_from(u32_at(bytes, 24)?).ok()?)?;
    (offset == bytes.len()).then_some(Bundle {
        expected: u16_at(bytes, 10)?,
        copy_fault: u32_at(bytes, 32)? != 0,
        adapter: AdapterInputs {
            expected_architecture: u16_at(bytes, 36)?,
            address_bits: *bytes.get(38)?,
            external_entry_address: u64_at(bytes, 40)?,
            external_entry_bytes: u32_at(bytes, 12)?,
            page_bytes: u64_at(bytes, 48)?,
            entry_alignment: u16_at(bytes, 56)?,
            stack_alignment: u16_at(bytes, 58)?,
        },
        entry,
        handoff,
        map,
        rhd,
    })
}

fn bundle_offsets(bytes: &[u8]) -> Option<(usize, usize, usize, usize)> {
    let entry = BUNDLE_HEADER_BYTES;
    let handoff = entry.checked_add(usize::try_from(u32_at(bytes, 12)?).ok()?)?;
    let map = handoff.checked_add(usize::try_from(u32_at(bytes, 16)?).ok()?)?;
    let rhd = map.checked_add(usize::try_from(u32_at(bytes, 20)?).ok()?)?;
    Some((entry, handoff, map, rhd))
}

fn checked_end(base: u64, length: u64) -> Result<u64, Code> {
    base.checked_add(length).ok_or(Code::RangeOverflow)
}

fn overlaps(a: (u64, u64), b: (u64, u64)) -> Result<bool, Code> {
    Ok(a.0 < checked_end(b.0, b.1)? && b.0 < checked_end(a.0, a.1)?)
}

fn address_fits(base: u64, length: u64, bits: u8) -> Result<bool, Code> {
    let end = checked_end(base, length)?;
    let ceiling = match bits {
        0..=63 => 1u64.checked_shl(u32::from(bits)),
        64 => None,
        _ => return Ok(false),
    };
    Ok(ceiling.is_none_or(|limit| end <= limit))
}

fn descriptor_offset(bytes: &[u8], purpose: u16, owner_kind: u16, owner_id: u32) -> Option<usize> {
    let (entry, _, _, _) = bundle_offsets(bytes)?;
    let count = usize::from(u16_at(bytes, entry + 24)?);
    (0..count).map(|index| entry + 64 + index * 32).find(|offset| {
        u16_at(bytes, *offset + 16) == Some(purpose)
            && u16_at(bytes, *offset + 24) == Some(owner_kind)
            && u32_at(bytes, *offset + 28) == Some(owner_id)
    })
}

fn rhd_records(bytes: &[u8]) -> Option<Vec<(u16, u32, usize, usize)>> {
    let (_, _, _, rhd) = bundle_offsets(bytes)?;
    let count = usize::from(u16_at(bytes, rhd + 20)?);
    let mut at = rhd + 32;
    let mut records = Vec::with_capacity(count);
    for _ in 0..count {
        let kind = u16_at(bytes, at)?;
        let size = usize::try_from(u32_at(bytes, at + 4)?).ok()?;
        let id = u32_at(bytes, at + 8)?;
        records.push((kind, id, at, size));
        at = at.checked_add(size)?;
    }
    Some(records)
}

fn record_offset(bytes: &[u8], kind: u16, id: u32) -> Option<usize> {
    rhd_records(bytes)?.into_iter().find(|record| record.0 == kind && record.1 == id).map(|record| record.2)
}

fn update_rhd_lengths(bytes: &mut [u8], new_length: u32, new_count: u16) {
    let (entry, handoff, _, rhd) = bundle_offsets(bytes).expect("bundle offsets");
    put_u32(bytes, 24, new_length);
    let rhd_descriptor = descriptor_offset(bytes, 3, 0, 0).expect("RHD descriptor");
    put_u64(bytes, rhd_descriptor + 8, u64::from(new_length));
    put_u32(bytes, handoff + 56, new_length);
    put_u32(bytes, rhd + 16, new_length);
    put_u16(bytes, rhd + 20, new_count);
    let _ = entry;
}

fn insert_rhd_record(bytes: &mut Vec<u8>, record: Vec<u8>) {
    let records = rhd_records(bytes).expect("RHD records");
    let new_key = (
        u16_at(&record, 0).expect("new record kind"),
        u32_at(&record, 8).expect("new record id"),
    );
    let insert = records.iter().find(|record| (record.0, record.1) > new_key).map_or_else(
        || {
            let (_, _, _, rhd) = bundle_offsets(bytes).expect("bundle offsets");
            rhd + usize::try_from(u32_at(bytes, rhd + 16).expect("RHD length")).expect("RHD length usize")
        },
        |record| record.2,
    );
    let (_, _, _, rhd) = bundle_offsets(bytes).expect("bundle offsets");
    let old_length = u32_at(bytes, rhd + 16).expect("RHD length");
    let old_count = u16_at(bytes, rhd + 20).expect("RHD count");
    update_rhd_lengths(bytes, old_length + u32::try_from(record.len()).expect("record size"), old_count + 1);
    bytes.splice(insert..insert, record);
}

fn duplicate_rhd_record(bytes: &mut Vec<u8>, kind: u16, source_id: u32, new_id: u32) {
    let source = rhd_records(bytes)
        .expect("RHD records")
        .into_iter()
        .find(|record| record.0 == kind && record.1 == source_id)
        .expect("record to duplicate");
    let mut duplicate = bytes[source.2..source.2 + source.3].to_vec();
    put_u32(&mut duplicate, 8, new_id);
    insert_rhd_record(bytes, duplicate);
}

fn swap_equal_rhd_records(bytes: &mut [u8], kind: u16, first_id: u32, second_id: u32) {
    let first = record_offset(bytes, kind, first_id).expect("first record");
    let second = record_offset(bytes, kind, second_id).expect("second record");
    let first_size = usize::try_from(u32_at(bytes, first + 4).expect("first size")).expect("first size usize");
    let second_size = usize::try_from(u32_at(bytes, second + 4).expect("second size")).expect("second size usize");
    assert_eq!(first_size, second_size, "records must have equal size");
    for index in 0..first_size {
        bytes.swap(first + index, second + index);
    }
}

fn remove_rhd_record(bytes: &mut Vec<u8>, kind: u16, id: u32) {
    let records = rhd_records(bytes).expect("RHD records");
    let record = records.into_iter().find(|record| record.0 == kind && record.1 == id).expect("record to remove");
    let (_, _, _, rhd) = bundle_offsets(bytes).expect("bundle offsets");
    let old_length = u32_at(bytes, rhd + 16).expect("RHD length");
    let old_count = u16_at(bytes, rhd + 20).expect("RHD count");
    update_rhd_lengths(bytes, old_length - u32::try_from(record.3).expect("record size"), old_count - 1);
    bytes.drain(record.2..record.2 + record.3);
}

fn record_header(kind: u16, flags: u16, bytes: u32, id: u32) -> Vec<u8> {
    let mut record = vec![0; usize::try_from(bytes).expect("record bytes")];
    put_u16(&mut record, 0, kind);
    put_u16(&mut record, 2, flags);
    put_u32(&mut record, 4, bytes);
    put_u32(&mut record, 8, id);
    record
}

fn add_entry_descriptor(bytes: &mut Vec<u8>, critical: bool, inert: bool) {
    let (entry, _, _, _) = bundle_offsets(bytes).expect("bundle offsets");
    let old_length = usize::try_from(u32_at(bytes, 12).expect("entry length")).expect("entry length usize");
    let count = u16_at(bytes, entry + 24).expect("descriptor count");
    let insert = entry + old_length;
    put_u32(bytes, 12, u32::try_from(old_length + 32).expect("entry length u32"));
    put_u32(bytes, entry + 16, u32::try_from(old_length + 32).expect("entry total u32"));
    put_u16(bytes, entry + 24, count + 1);
    put_u16(bytes, entry + 10, 1);
    let mut descriptor = vec![0; 32];
    put_u64(&mut descriptor, 0, 0x3000_0000);
    put_u64(&mut descriptor, 8, 8);
    put_u16(&mut descriptor, 16, 99);
    put_u16(&mut descriptor, 20, 1);
    put_u16(&mut descriptor, 26, if critical { 4 } else { 0 });
    if !inert {
        put_u16(&mut descriptor, 18, 1);
    }
    bytes.splice(insert..insert, descriptor);
}

fn swap_descriptors(bytes: &mut [u8]) {
    let (entry, _, _, _) = bundle_offsets(bytes).expect("bundle offsets");
    let count = usize::from(u16_at(bytes, entry + 24).expect("descriptor count"));
    let first = entry + 64;
    let last = entry + 64 + (count - 1) * 32;
    for index in 0..32 {
        bytes.swap(first + index, last + index);
    }
}

fn remove_entry_descriptor(bytes: &mut Vec<u8>, purpose: u16, owner_kind: u16, owner_id: u32) {
    let (entry, _, _, _) = bundle_offsets(bytes).expect("bundle offsets");
    let at = descriptor_offset(bytes, purpose, owner_kind, owner_id).expect("descriptor to remove");
    let old_length = u32_at(bytes, 12).expect("entry length");
    let count = u16_at(bytes, entry + 24).expect("descriptor count");
    put_u32(bytes, 12, old_length - 32);
    put_u32(bytes, entry + 16, old_length - 32);
    put_u16(bytes, entry + 24, count - 1);
    bytes.drain(at..at + 32);
}

fn apply_mutation(bytes: &mut Vec<u8>, predicate: &str, behavior: &mut Behavior) {
    let (entry, handoff, map, rhd) = bundle_offsets(bytes).expect("bundle offsets");
    match predicate {
        "adapter.entry.external-bounds" => behavior.external_length = Some(63),
        "adapter.entry.external-ceiling" => behavior.external_length = Some(4097),
        "adapter.entry.address-width" => { behavior.external_address = Some(0x1_0000_0000); behavior.adapter_address_bits = Some(32); }
        "adapter.entry.alignment" => behavior.external_address = Some(ENTRY_ADDRESS + 1),
        "entry.header.magic" => bytes[entry] ^= 0xff,
        "entry.header.major-version" => put_u16(bytes, entry + 8, 2),
        "entry.header.fixed-sizes" => put_u16(bytes, entry + 12, 63),
        "entry.header.flags" => bytes[entry + 23] = 1,
        "entry.header.reserved" => bytes[entry + 32] = 1,
        "entry.header.framing" => put_u16(bytes, entry + 24, 126),
        "descriptors.table.range-arithmetic" => {
            let at = descriptor_offset(bytes, 1, 0, 0).expect("handoff descriptor");
            put_u64(bytes, at, u64::MAX);
            put_u64(bytes, at + 8, 2);
        }
        "descriptors.table.minor-compatibility" => add_entry_descriptor(bytes, false, false),
        "descriptors.table.binding" => {
            let at = descriptor_offset(bytes, 1, 0, 0).expect("handoff descriptor");
            put_u16(bytes, at + 24, 1);
        }
        "descriptors.table.aliasing" => {
            let handoff_descriptor = descriptor_offset(bytes, 1, 0, 0).expect("handoff descriptor");
            let map_descriptor = descriptor_offset(bytes, 2, 0, 0).expect("map descriptor");
            let handoff_base = u64_at(bytes, handoff_descriptor).expect("handoff base");
            put_u64(bytes, map_descriptor, handoff_base);
        }
        "acquisition.sources.copy" => behavior.fault_purpose = Some(1),
        "handoff.header.framing" => put_u32(bytes, handoff + 16, 127),
        "handoff.rhd-location.alignment" => put_u16(bytes, handoff + 60, 16),
        "memory-map.entries.framing-and-semantics" => put_u64(bytes, map + 8, 0),
        "rhd.records.framing" => {
            let cpu = record_offset(bytes, 1, 1).expect("CPU record");
            put_u32(bytes, cpu + 4, 40);
        }
        "rhd.records.critical-extension" => {
            let boot = record_offset(bytes, 6, 1).expect("boot record");
            put_u16(bytes, boot, 99);
            put_u16(bytes, boot + 2, 1);
        }
        "rhd.records.minor-support" => insert_rhd_record(bytes, record_header(99, 0, 24, 1)),
        "rhd.records.identity" => {
            let window = record_offset(bytes, 7, 2).expect("second window");
            put_u32(bytes, window + 8, 1);
        }
        "rhd.records.references" => {
            let cpu = record_offset(bytes, 1, 2).expect("second CPU");
            put_u32(bytes, cpu + 32, 99);
        }
        "rhd.cpu.cardinality-and-boot" => {
            let cpu = record_offset(bytes, 1, 2).expect("second CPU");
            put_u32(bytes, cpu + 16, 1);
        }
        "rhd.interrupt.cardinality-and-model" => {
            let interrupt = record_offset(bytes, 3, 1).expect("interrupt record");
            put_u16(bytes, interrupt + 16, 0);
        }
        "rhd.timer.cardinality-and-model" => {
            let timer = record_offset(bytes, 4, 1).expect("timer record");
            put_u64(bytes, timer + 24, 0);
        }
        "rhd.serial.cardinality-and-model" => {
            let serial = record_offset(bytes, 5, 1).expect("serial record");
            put_u32(bytes, serial + 20, u32::MAX);
        }
        "rhd.boot-source.cardinality-and-value" => {
            let boot = record_offset(bytes, 6, 1).expect("boot record");
            put_u16(bytes, boot + 16, 2);
        }
        "handoff.entropy.binding" => put_u32(bytes, handoff + 72, 31),
        "handoff.trace.binding" => put_u32(bytes, handoff + 96, 4095),
        "rhd.records.canonical-order" => {
            let first = record_offset(bytes, 2, 1).expect("first memory record");
            let second = record_offset(bytes, 2, 2).expect("second memory record");
            put_u32(bytes, first + 8, 10);
            put_u32(bytes, second + 8, 9);
        }
        "cross-artifact.memory.map-rhd-consistency" => {
            let memory = record_offset(bytes, 2, 2).expect("memory record");
            put_u64(bytes, memory + 16, 1_572_864);
        }
        "cross-artifact.registers.model-and-cardinality" => {
            let window = record_offset(bytes, 7, 1).expect("window record");
            put_u16(bytes, window + 28, 2);
        }
        "cross-artifact.authority.descriptor-and-alias" => {
            let window = record_offset(bytes, 7, 1).expect("window record");
            put_u16(bytes, window + 48, 7);
        }
        "cross-artifact.architecture.agreement" => {
            let current = u16_at(bytes, rhd + 24).expect("RHD architecture");
            put_u16(bytes, rhd + 24, if current == 1 { 2 } else { 1 });
        }
        "cross-artifact.address-width.agreement" => bytes[rhd + 26] = 47,
        "cross-artifact.page-size.agreement" => bytes[rhd + 27] = 13,
        "entry-descriptor-alias" => {
            let at = descriptor_offset(bytes, 2, 0, 0).expect("map descriptor");
            put_u64(bytes, at, ENTRY_ADDRESS);
        }
        "descriptor-reordered" => swap_descriptors(bytes),
        "descriptor-range-plus-binding" => {
            let handoff_descriptor = descriptor_offset(bytes, 1, 0, 0).expect("handoff descriptor");
            put_u16(bytes, handoff_descriptor + 24, 1);
            let map_descriptor = descriptor_offset(bytes, 2, 0, 0).expect("map descriptor");
            put_u64(bytes, map_descriptor, u64::MAX);
            put_u64(bytes, map_descriptor + 8, 2);
        }
        "descriptor-binding-plus-alignment" => {
            let handoff_descriptor = descriptor_offset(bytes, 1, 0, 0).expect("handoff descriptor");
            put_u16(bytes, handoff_descriptor + 24, 1);
            let map_descriptor = descriptor_offset(bytes, 2, 0, 0).expect("map descriptor");
            put_u64(bytes, map_descriptor, 8193);
        }
        "compatible-entry-minor" => add_entry_descriptor(bytes, false, true),
        "critical-entry-minor" => add_entry_descriptor(bytes, true, true),
        "compatible-rhd-minor" => {
            put_u16(bytes, rhd + 10, 1);
            insert_rhd_record(bytes, record_header(99, 0, 24, 1));
        }
        "compatible-window-role" => {
            put_u16(bytes, rhd + 10, 1);
            let mut window = record_header(7, 0, 56, 99);
            put_u16(&mut window, 18, 99);
            insert_rhd_record(bytes, window);
        }
        "critical-window-role" => {
            put_u16(bytes, rhd + 10, 1);
            let mut window = record_header(7, 1, 56, 99);
            put_u16(&mut window, 18, 99);
            insert_rhd_record(bytes, window);
        }
        "critical-rhd-minor" => {
            put_u16(bytes, rhd + 10, 1);
            insert_rhd_record(bytes, record_header(99, 1, 24, 1));
        }
        "known-window-critical" => {
            let window = record_offset(bytes, 7, 1).expect("known window");
            put_u16(bytes, window + 2, 1);
        }
        "rhd-compatibility-compound" => {
            insert_rhd_record(bytes, record_header(99, 1, 24, 1));
            insert_rhd_record(bytes, record_header(99, 0, 24, 2));
        }
        "rhd-compatibility-reorder" => swap_equal_rhd_records(bytes, 99, 1, 2),
        "secondary-cpu-reference" => {
            let cpu = record_offset(bytes, 1, 2).expect("second CPU");
            put_u32(bytes, cpu + 36, 99);
        }
        "double-boot-cpu" => {
            let cpu = record_offset(bytes, 1, 2).expect("second CPU");
            put_u32(bytes, cpu + 16, 1);
        }
        "duplicate-cpu-hardware" => {
            let first = record_offset(bytes, 1, 1).expect("first CPU");
            let second = record_offset(bytes, 1, 2).expect("second CPU");
            let hardware_id = u64_at(bytes, first + 24).expect("hardware id");
            put_u64(bytes, second + 24, hardware_id);
        }
        "missing-serial" => {
            let arch = u16_at(bytes, entry + 20).expect("entry architecture");
            remove_entry_descriptor(bytes, if arch == 1 { 7 } else { 6 }, 5, 1);
            let window_id = if arch == 1 { 2 } else { 3 };
            remove_rhd_record(bytes, 7, window_id);
            remove_rhd_record(bytes, 5, 1);
        }
        "wrong-timer-model" => {
            let timer = record_offset(bytes, 4, 1).expect("timer record");
            let arch = u16_at(bytes, entry + 20).expect("entry architecture");
            put_u16(bytes, timer + 16, if arch == 1 { 2 } else { 1 });
        }
        "duplicate-boot-source" => {
            let mut boot = record_header(6, 0, 24, 2);
            put_u16(&mut boot, 16, 1);
            insert_rhd_record(bytes, boot);
        }
        "duplicate-interrupt" => duplicate_rhd_record(bytes, 3, 1, 2),
        "interrupt-overflow" => {
            let interrupt = record_offset(bytes, 3, 1).expect("interrupt record");
            put_u32(bytes, interrupt + 20, u32::MAX);
            put_u32(bytes, interrupt + 24, 2);
        }
        "interrupt-overlap" => duplicate_rhd_record(bytes, 3, 1, 2),
        "interrupt-upper-bound" => {
            let interrupt = record_offset(bytes, 3, 1).expect("interrupt record");
            let architecture = u16_at(bytes, entry + 20).expect("entry architecture");
            put_u32(bytes, interrupt + 20, if architecture == 1 { 32 } else { 796 });
            put_u32(bytes, interrupt + 24, 224);
        }
        "duplicate-timer" => duplicate_rhd_record(bytes, 4, 1, 2),
        "duplicate-serial" => duplicate_rhd_record(bytes, 5, 1, 2),
        "record-reordered" => swap_equal_rhd_records(bytes, 1, 1, 2),
        "unknown-record-reordered" => {
            put_u16(bytes, rhd + 10, 1);
            insert_rhd_record(bytes, record_header(99, 0, 24, 1));
            insert_rhd_record(bytes, record_header(100, 0, 24, 1));
            let first_unknown = record_offset(bytes, 99, 1).expect("first unknown record");
            put_u16(bytes, first_unknown, 101);
        }
        "map-count-over-limit" => put_u32(bytes, handoff + 40, 257),
        "map-region-overflow" => {
            put_u64(bytes, map, u64::MAX);
            put_u64(bytes, map + 8, 2);
        }
        "map-source-containment" => put_u64(bytes, map + 8, 64),
        "map-record-reordered" => {
            let count = usize::try_from(u32_at(bytes, handoff + 40).expect("map count")).expect("map count usize");
            let last = map + (count - 1) * 32;
            for index in 0..32 {
                bytes.swap(map + index, last + index);
            }
        }
        "map-address-mismatch" => put_u64(bytes, handoff + 32, 8192 + 32),
        "rhd-address-mismatch" => put_u64(bytes, handoff + 48, 12288 + 8),
        "rhd-length-mismatch" => {
            let length = u32_at(bytes, handoff + 56).expect("RHD length");
            put_u32(bytes, handoff + 56, length - 8);
        }
        "rhd-alignment" => put_u16(bytes, handoff + 60, 16),
        "descriptor-owner" => {
            let at = descriptor_offset(bytes, 3, 0, 0).expect("RHD descriptor");
            put_u16(bytes, at + 24, 3);
            put_u32(bytes, at + 28, 1);
        }
        "clear-entropy" => {
            put_u32(bytes, handoff + 76, 1);
            let at = descriptor_offset(bytes, 4, 0, 0).expect("entropy descriptor");
            put_u16(bytes, at + 18, 3);
            put_u16(bytes, at + 22, 4);
        }
        "invalid-adapter-address-bits" => behavior.adapter_address_bits = Some(0),
        "adapter-address-bits-65" => behavior.adapter_address_bits = Some(65),
        "adapter-address-bits-255" => behavior.adapter_address_bits = Some(255),
        "invalid-adapter-architecture" => behavior.adapter_architecture = Some(99),
        "invalid-adapter-page-zero" => behavior.page_bytes = Some(0),
        "invalid-adapter-page-nonpower" => behavior.page_bytes = Some(6144),
        "entry-address-bits-65" => bytes[entry + 22] = 65,
        "entry-address-bits-255" => bytes[entry + 22] = 255,
        "entry-address-bits-mismatch" => bytes[entry + 22] = 47,
        "entry-architecture-mismatch" => {
            let current = u16_at(bytes, entry + 20).expect("entry architecture");
            put_u16(bytes, entry + 20, if current == 1 { 2 } else { 1 });
        }
        "system-memory-window-overflow" => {
            let window = record_offset(bytes, 7, 1).expect("system-memory window");
            put_u64(bytes, window + 32, u64::MAX - 7);
            put_u64(bytes, window + 40, 16);
        }
        _ => panic!("unknown mutation: {predicate}"),
    }
}

fn adapter_for(bundle: &Bundle<'_>, behavior: Behavior) -> AdapterInputs {
    AdapterInputs {
        expected_architecture: behavior.adapter_architecture.unwrap_or(bundle.adapter.expected_architecture),
        external_entry_address: behavior.external_address.unwrap_or(bundle.adapter.external_entry_address),
        external_entry_bytes: behavior.external_length.unwrap_or(bundle.adapter.external_entry_bytes),
        address_bits: behavior.adapter_address_bits.unwrap_or(bundle.adapter.address_bits),
        page_bytes: behavior.page_bytes.unwrap_or(bundle.adapter.page_bytes),
        entry_alignment: behavior.entry_alignment.unwrap_or(bundle.adapter.entry_alignment),
        stack_alignment: bundle.adapter.stack_alignment,
    }
}

fn descriptor_space(purpose: u16) -> u8 {
    if purpose == 7 { 2 } else { 1 }
}

fn validate(adapter: AdapterInputs, provider: &mut SourceProvider<'_>, sink: &mut EffectSink) -> Result<(), Code> {
    if adapter.external_entry_bytes < 64 {
        return Err(Code::Truncated);
    }
    if adapter.external_entry_bytes > 4096 {
        return Err(Code::Oversized);
    }
    checked_end(adapter.external_entry_address, u64::from(adapter.external_entry_bytes)).map_err(|_| Code::OutOfAddressRange)?;
    if !(32..=64).contains(&adapter.address_bits)
        || !address_fits(adapter.external_entry_address, u64::from(adapter.external_entry_bytes), adapter.address_bits)?
    {
        return Err(Code::OutOfAddressRange);
    }
    if !matches!(adapter.expected_architecture, 1 | 2) {
        return Err(Code::ArchitectureMismatch);
    }
    if adapter.entry_alignment != 8
        || adapter.stack_alignment != 16
        || adapter.external_entry_address % u64::from(adapter.entry_alignment) != 0
        || !adapter.page_bytes.is_power_of_two()
        || !(4096..=1_073_741_824).contains(&adapter.page_bytes)
    {
        return Err(Code::BadAlignment);
    }

    let entry = provider.copy_entry(usize::try_from(adapter.external_entry_bytes).map_err(|_| Code::Oversized)?)?;
    if entry.get(0..8) != Some(ENTRY_MAGIC) {
        return Err(Code::BadMagic);
    }
    if u16_at(&entry, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(&entry, 12) != Some(64) || u16_at(&entry, 14) != Some(32) {
        return Err(Code::BadHeaderSize);
    }
    if entry.get(23) != Some(&0) {
        return Err(Code::UnsupportedFlags);
    }
    if entry.get(26..64) != Some([0; 38].as_slice()) {
        return Err(Code::NonzeroReserved);
    }
    let entry_minor = u16_at(&entry, 10).ok_or(Code::InvalidEntry)?;
    let entry_total = usize::try_from(u32_at(&entry, 16).ok_or(Code::InvalidEntry)?).map_err(|_| Code::InvalidEntry)?;
    let descriptor_count = usize::from(u16_at(&entry, 24).ok_or(Code::InvalidEntry)?);
    let exact_entry = 64usize.checked_add(descriptor_count.checked_mul(32).ok_or(Code::RangeOverflow)?).ok_or(Code::RangeOverflow)?;
    if descriptor_count == 0 || descriptor_count > 126 || entry_total != entry.len() || entry_total != exact_entry {
        return Err(Code::InvalidEntry);
    }

    let entry_architecture = u16_at(&entry, 20).ok_or(Code::InvalidEntry)?;
    let entry_address_bits = *entry.get(22).ok_or(Code::InvalidEntry)?;
    // Producer-controlled entry values are bound to the trusted adapter tuple
    // before they can influence descriptor arithmetic, bounds, or acquisition.
    if !matches!(entry_architecture, 1 | 2)
        || entry_architecture != adapter.expected_architecture
        || !(32..=64).contains(&entry_address_bits)
        || entry_address_bits != adapter.address_bits
    {
        return Err(Code::InvalidEntry);
    }
    let mut descriptors = Vec::with_capacity(descriptor_count);
    for index in 0..descriptor_count {
        let at = 64 + index * 32;
        let purpose = u16_at(&entry, at + 16).ok_or(Code::InvalidEntry)?;
        let descriptor = Descriptor {
            selector: DescriptorSelector {
                purpose,
                owner_kind: u16_at(&entry, at + 24).ok_or(Code::InvalidEntry)?,
                owner_id: u32_at(&entry, at + 28).ok_or(Code::InvalidEntry)?,
            },
            base: u64_at(&entry, at).ok_or(Code::InvalidEntry)?,
            length: u64_at(&entry, at + 8).ok_or(Code::InvalidEntry)?,
            rights: u16_at(&entry, at + 18).ok_or(Code::InvalidEntry)?,
            producer: u16_at(&entry, at + 20).ok_or(Code::InvalidEntry)?,
            transfer: u16_at(&entry, at + 22).ok_or(Code::InvalidEntry)?,
            flags: u16_at(&entry, at + 26).ok_or(Code::InvalidEntry)?,
        };
        descriptors.push(descriptor);
    }

    // Descriptor predicates are global stages. A later table row can never win
    // merely because its descriptor appeared earlier in the entry table.
    for descriptor in &descriptors {
        checked_end(descriptor.base, descriptor.length)?;
    }

    let unknown: Vec<_> = descriptors.iter().filter(|descriptor| !(1..=7).contains(&descriptor.selector.purpose)).collect();
    if entry_minor == 0 && !unknown.is_empty()
        || unknown.iter().any(|descriptor| descriptor.rights != 0 || descriptor.transfer != 0 || descriptor.selector.owner_kind != 0 || descriptor.selector.owner_id != 0 || descriptor.flags != 0)
    {
        return Err(Code::UnsupportedMinor);
    }

    // All binding predicates are one whole-table stage and collapse to the
    // declared invalid-pointer-range code. Descriptor order cannot select a
    // different range, address-width, alignment, selector, or rights error.
    if descriptors.iter().any(|descriptor| {
        descriptor.length == 0
            || !address_fits(descriptor.base, descriptor.length, entry_address_bits).unwrap_or(false)
            || descriptor_space(descriptor.selector.purpose) == 1 && descriptor.base % 8 != 0
            || descriptor.selector.purpose == 3 && descriptor.length % 8 != 0
            || descriptor.selector.purpose == 5 && (descriptor.base % 64 != 0 || descriptor.length % 64 != 0)
    }) {
        return Err(Code::InvalidPointerRange);
    }
    let mut source_selectors = BTreeSet::new();
    if descriptors.iter().filter(|descriptor| descriptor.selector.purpose <= 5).any(|descriptor| !source_selectors.insert(descriptor.selector)) {
        return Err(Code::InvalidPointerRange);
    }
    for purpose in 1..=5 {
        if descriptors.iter().filter(|descriptor| descriptor.selector.purpose == purpose).count() != 1 {
            return Err(Code::InvalidPointerRange);
        }
    }
    for descriptor in descriptors.iter().filter(|descriptor| descriptor.selector.purpose <= 7) {
        let valid = match descriptor.selector.purpose {
            1..=3 => descriptor.rights == 1 && descriptor.transfer == 1 && descriptor.flags == 3 && descriptor.selector.owner_kind == 0 && descriptor.selector.owner_id == 0,
            4 => matches!((descriptor.rights, descriptor.transfer), (1, 1) | (3, 4)) && descriptor.flags == 3 && descriptor.selector.owner_kind == 0 && descriptor.selector.owner_id == 0,
            5 => descriptor.rights == 3 && descriptor.transfer == 2 && descriptor.flags == 3 && descriptor.selector.owner_kind == 0 && descriptor.selector.owner_id == 0,
            6 | 7 => descriptor.rights == 11 && descriptor.transfer == 3 && descriptor.flags == 0 && matches!(descriptor.selector.owner_kind, 3 | 5) && descriptor.selector.owner_id != 0,
            _ => false,
        } && matches!(descriptor.producer, 1 | 2);
        if !valid {
            return Err(Code::InvalidPointerRange);
        }
    }

    for first in 0..descriptors.len() {
        for second in first + 1..descriptors.len() {
            if descriptor_space(descriptors[first].selector.purpose) == descriptor_space(descriptors[second].selector.purpose)
                && overlaps((descriptors[first].base, descriptors[first].length), (descriptors[second].base, descriptors[second].length))?
            {
                return Err(Code::Overlap);
            }
        }
    }
    for descriptor in descriptors.iter().filter(|descriptor| descriptor_space(descriptor.selector.purpose) == 1) {
        if overlaps(
            (adapter.external_entry_address, u64::from(adapter.external_entry_bytes)),
            (descriptor.base, descriptor.length),
        )? {
            return Err(Code::Overlap);
        }
    }

    let descriptor_by_selector: BTreeMap<_, _> = descriptors.iter().filter(|descriptor| descriptor.selector.purpose <= 5).map(|descriptor| (descriptor.selector, descriptor)).collect();
    let required = |purpose| DescriptorSelector { purpose, owner_kind: 0, owner_id: 0 };
    let handoff_descriptor = *descriptor_by_selector.get(&required(1)).ok_or(Code::InvalidPointerRange)?;
    let map_descriptor = *descriptor_by_selector.get(&required(2)).ok_or(Code::InvalidPointerRange)?;
    let rhd_descriptor = *descriptor_by_selector.get(&required(3)).ok_or(Code::InvalidPointerRange)?;
    let entropy_descriptor = *descriptor_by_selector.get(&required(4)).ok_or(Code::InvalidPointerRange)?;
    let trace_descriptor = *descriptor_by_selector.get(&required(5)).ok_or(Code::InvalidPointerRange)?;

    let handoff = provider.copy_descriptor(handoff_descriptor)?;
    let map = provider.copy_descriptor(map_descriptor)?;
    let rhd = provider.copy_descriptor(rhd_descriptor)?;
    let _entropy = provider.copy_descriptor(entropy_descriptor)?;

    if handoff.len() < 128 || handoff.get(0..8) != Some(BOOT_MAGIC) {
        return Err(Code::BadCountOrLength);
    }
    if u16_at(&handoff, 8) != Some(1) {
        return Err(Code::UnsupportedMajor);
    }
    if u16_at(&handoff, 12) != Some(128)
        || u32_at(&handoff, 16) != Some(128)
        || handoff.len() != 128
        || u16_at(&handoff, 14) != Some(0)
        || handoff.get(26..32) != Some([0; 6].as_slice())
        || handoff.get(108..128) != Some([0; 20].as_slice())
    {
        return Err(Code::BadCountOrLength);
    }
    if u16_at(&handoff, 10).unwrap_or(0) > 0 && u32_at(&handoff, 16) != Some(128) {
        return Err(Code::UnsupportedMinor);
    }
    if u16_at(&handoff, 60) != Some(8) || rhd_descriptor.base % 8 != 0 || rhd_descriptor.length % 8 != 0 {
        return Err(Code::BadAlignment);
    }

    let map_count = usize::try_from(u32_at(&handoff, 40).ok_or(Code::InvalidMemoryMap)?).map_err(|_| Code::InvalidMemoryMap)?;
    if map_count == 0
        || map_count > 256
        || map_count.checked_mul(32) != Some(map.len())
        || u64::try_from(map.len()).ok() != Some(map_descriptor.length)
        || u64_at(&handoff, 32) != Some(map_descriptor.base)
        || u16_at(&handoff, 44) != Some(32)
        || u16_at(&handoff, 46) != Some(1)
    {
        return Err(Code::InvalidMemoryMap);
    }
    let mut map_entries = Vec::with_capacity(map_count);
    for index in 0..map_count {
        let at = index * 32;
        let memory = Memory {
            id: u32_at(&map, at + 24).ok_or(Code::InvalidMemoryMap)?,
            base: u64_at(&map, at).ok_or(Code::InvalidMemoryMap)?,
            length: u64_at(&map, at + 8).ok_or(Code::InvalidMemoryMap)?,
            kind: u16_at(&map, at + 16).ok_or(Code::InvalidMemoryMap)?,
            attributes: u16_at(&map, at + 18).ok_or(Code::InvalidMemoryMap)?,
            owner: u16_at(&map, at + 20).ok_or(Code::InvalidMemoryMap)?,
        };
        map_entries.push((memory, at));
    }

    // The sole map predicate is a total whole-table reduction. Arithmetic is
    // checked for every entry before semantic, identity, ordering, overlap,
    // ownership, or source-containment facts are evaluated. Every failure in
    // this stage has the allocated invalid-memory-map result.
    if map_entries.iter().any(|(memory, _)| {
        memory.length == 0
            || checked_end(memory.base, memory.length).is_err()
            || !address_fits(memory.base, memory.length, adapter.address_bits).unwrap_or(false)
    }) {
        return Err(Code::InvalidMemoryMap);
    }

    let mut map_by_id = BTreeMap::new();
    let mut map_by_base = Vec::with_capacity(map_count);
    for (memory, at) in map_entries {
        let exact_authority = match memory.kind {
            1 => memory.attributes == 11 && memory.owner == 0,
            2 => memory.attributes == 9 && memory.owner == 4,
            3 => memory.attributes == 11 && matches!(memory.owner, 1 | 2),
            4 => memory.attributes == 11 && memory.owner == 3,
            5 => memory.attributes == 19 && memory.owner == 5,
            6 => memory.attributes == 0 && memory.owner == 0,
            _ => false,
        };
        if memory.id == 0
            || !exact_authority
            || map.get(at + 22..at + 24) != Some([0; 2].as_slice())
            || map.get(at + 28..at + 32) != Some([0; 4].as_slice())
            || map_by_id.insert(memory.id, memory.clone()).is_some()
        {
            return Err(Code::InvalidMemoryMap);
        }
        if map_by_base.last().is_some_and(|previous: &Memory| {
            previous.base >= memory.base || checked_end(previous.base, previous.length).is_ok_and(|end| end > memory.base)
        }) {
            return Err(Code::InvalidMemoryMap);
        }
        map_by_base.push(memory);
    }

    for descriptor in descriptors.iter().filter(|descriptor| (1..=5).contains(&descriptor.selector.purpose)) {
        if !map_by_base.iter().any(|memory| {
            memory.kind == 3
                && memory.owner == descriptor.producer
                && memory.base <= descriptor.base
                && checked_end(memory.base, memory.length).is_ok_and(|end| checked_end(descriptor.base, descriptor.length).is_ok_and(|source_end| source_end <= end))
        }) {
            return Err(Code::InvalidMemoryMap);
        }
    }

    if rhd.len() < 32 || rhd.get(0..8) != Some(RHD_MAGIC) || u16_at(&rhd, 8) != Some(1) {
        return Err(Code::BadCountOrLength);
    }
    let rhd_minor = u16_at(&rhd, 10).ok_or(Code::BadCountOrLength)?;
    let rhd_total = usize::try_from(u32_at(&rhd, 16).ok_or(Code::BadCountOrLength)?).map_err(|_| Code::BadCountOrLength)?;
    let record_count = usize::from(u16_at(&rhd, 20).ok_or(Code::BadCountOrLength)?);
    if u64_at(&handoff, 48) != Some(rhd_descriptor.base)
        || u32_at(&handoff, 56).map(u64::from) != Some(rhd_descriptor.length)
        || u64::try_from(rhd.len()).ok() != Some(rhd_descriptor.length)
        || u16_at(&rhd, 12) != Some(32)
        || u16_at(&rhd, 14) != Some(16)
        || u16_at(&rhd, 22) != Some(0)
        || rhd.get(28..32) != Some([0; 4].as_slice())
        || rhd_total != rhd.len()
        || rhd_total > 65_536
        || rhd_total % 8 != 0
        || record_count == 0
        || record_count > 256
    {
        return Err(Code::BadCountOrLength);
    }

    let mut at = 32usize;
    let mut records = Vec::with_capacity(record_count);
    for _ in 0..record_count {
        let kind = u16_at(&rhd, at).ok_or(Code::BadCountOrLength)?;
        let flags = u16_at(&rhd, at + 2).ok_or(Code::BadCountOrLength)?;
        let size = usize::try_from(u32_at(&rhd, at + 4).ok_or(Code::BadCountOrLength)?).map_err(|_| Code::BadCountOrLength)?;
        let id = u32_at(&rhd, at + 8).ok_or(Code::BadCountOrLength)?;
        if size < 16 || size % 8 != 0 || at.checked_add(size).filter(|end| *end <= rhd.len()).is_none() || u32_at(&rhd, at + 12) != Some(0) {
            return Err(Code::BadCountOrLength);
        }
        let known_size = match kind { 1..=5 => Some(48), 6 => Some(24), 7 => Some(56), _ => None };
        if known_size.is_some_and(|expected| expected != size)
            || flags & !1 != 0
            || known_size.is_some() && kind != 7 && flags != 0
        {
            return Err(Code::BadCountOrLength);
        }
        let reserved_zero = match kind {
            1 => rhd.get(at + 20..at + 24) == Some([0; 4].as_slice()) && rhd.get(at + 40..at + 48) == Some([0; 8].as_slice()),
            2 => rhd.get(at + 38..at + 48) == Some([0; 10].as_slice()),
            3 => rhd.get(at + 28..at + 48) == Some([0; 20].as_slice()),
            4 => rhd.get(at + 20..at + 24) == Some([0; 4].as_slice()) && rhd.get(at + 42..at + 48) == Some([0; 6].as_slice()),
            5 => rhd.get(at + 28..at + 32) == Some([0; 4].as_slice()) && rhd.get(at + 42..at + 48) == Some([0; 6].as_slice()),
            6 => rhd.get(at + 20..at + 24) == Some([0; 4].as_slice()),
            7 => rhd.get(at + 50..at + 56) == Some([0; 6].as_slice()),
            _ => true,
        };
        if !reserved_zero {
            return Err(Code::BadCountOrLength);
        }
        records.push((kind, id, at, size));
        at += size;
    }
    if at != rhd.len() {
        return Err(Code::BadCountOrLength);
    }

    // Known-record flag legality belongs to framing and is reduced globally
    // before compatibility, independent of physical record order.
    if records.iter().any(|(kind, _, offset, _)| {
        let flags = u16_at(&rhd, *offset + 2).unwrap_or(u16::MAX);
        let known_role = *kind == 7 && matches!(u16_at(&rhd, *offset + 18), Some(1..=5));
        ((1..=6).contains(kind) || known_role) && flags != 0
    }) {
        return Err(Code::BadCountOrLength);
    }

    // Compatibility facts are collected across the complete framed table,
    // then reduced in the canonical row order. Wire order cannot choose the
    // result. Optional additions remain excluded from semantic authority.
    let mut has_unknown_critical = false;
    let mut has_unsupported_minor = false;
    for &(kind, _, offset, _) in &records {
        let flags = u16_at(&rhd, offset + 2).ok_or(Code::BadCountOrLength)?;
        let unknown_record = !(1..=7).contains(&kind);
        let unknown_role = kind == 7 && !matches!(u16_at(&rhd, offset + 18), Some(1..=5));
        if unknown_record || unknown_role {
            has_unknown_critical |= flags & 1 != 0;
            has_unsupported_minor |= rhd_minor == 0 && flags & 1 == 0;
        }
    }
    if has_unknown_critical {
        return Err(Code::UnknownCritical);
    }
    if has_unsupported_minor {
        return Err(Code::UnsupportedMinor);
    }

    let semantic_records: Vec<_> = records.iter().copied().filter(|(kind, _, offset, _)| {
        (1..=7).contains(kind)
            && (*kind != 7 || matches!(u16_at(&rhd, *offset + 18), Some(1..=5)))
    }).collect();

    let mut identities = BTreeSet::new();
    let mut by_kind: BTreeMap<u16, BTreeMap<u32, usize>> = BTreeMap::new();
    for &(kind, id, offset, _) in &records {
        if !identities.insert((kind, id)) {
            return Err(Code::DuplicateId);
        }
        if kind <= 7 {
            by_kind.entry(kind).or_default().insert(id, offset);
        }
    }
    let has = |kind, id| by_kind.get(&kind).is_some_and(|items| items.contains_key(&id));
    for &(kind, _, offset, _) in &semantic_records {
        match kind {
            1 => {
                if !has(3, u32_at(&rhd, offset + 32).ok_or(Code::BadReference)?) || !has(4, u32_at(&rhd, offset + 36).ok_or(Code::BadReference)?) {
                    return Err(Code::BadReference);
                }
            }
            4 => if !has(3, u32_at(&rhd, offset + 36).ok_or(Code::BadReference)?) { return Err(Code::BadReference); },
            5 => if !has(3, u32_at(&rhd, offset + 24).ok_or(Code::BadReference)?) { return Err(Code::BadReference); },
            7 => {
                let parent_kind = u16_at(&rhd, offset + 16).ok_or(Code::BadReference)?;
                let parent_id = u32_at(&rhd, offset + 20).ok_or(Code::BadReference)?;
                if !matches!(parent_kind, 3 | 4 | 5) || !has(parent_kind, parent_id) {
                    return Err(Code::BadReference);
                }
            }
            _ => {}
        }
    }

    let cpus: Vec<_> = semantic_records.iter().filter(|record| record.0 == 1).collect();
    let boot_cpus: Vec<_> = cpus.iter().filter(|record| u32_at(&rhd, record.2 + 16) == Some(1)).collect();
    let mut hardware_ids = BTreeSet::new();
    if cpus.is_empty()
        || boot_cpus.len() != 1
        || cpus.iter().any(|record| u32_at(&rhd, record.2 + 16).is_none_or(|flags| flags & !1 != 0))
        || cpus.iter().any(|record| u64_at(&rhd, record.2 + 24).is_none_or(|id| !hardware_ids.insert(id)))
        || boot_cpus[0].1 != u32_at(&handoff, 104).unwrap_or(0)
    {
        return Err(Code::InvalidCpuSet);
    }

    let expected_interrupt_model = if adapter.expected_architecture == 1 { 1 } else { 2 };
    let mut controllers = BTreeMap::new();
    let interrupt_limit = if adapter.expected_architecture == 1 { 256 } else { 1020 };
    let mut interrupt_ranges = Vec::new();
    for record in semantic_records.iter().filter(|record| record.0 == 3) {
        let model = u16_at(&rhd, record.2 + 16).ok_or(Code::InvalidInterrupt)?;
        let base = u32_at(&rhd, record.2 + 20).ok_or(Code::InvalidInterrupt)?;
        let count = u32_at(&rhd, record.2 + 24).ok_or(Code::InvalidInterrupt)?;
        let end = base.checked_add(count).ok_or(Code::InvalidInterrupt)?;
        if model != expected_interrupt_model || count == 0 || end > interrupt_limit || u16_at(&rhd, record.2 + 18) != Some(0) {
            return Err(Code::InvalidInterrupt);
        }
        interrupt_ranges.push((base, end));
        controllers.insert(record.1, count);
    }
    if interrupt_ranges.iter().enumerate().any(|(index, first)| {
        interrupt_ranges.iter().skip(index + 1).any(|second| first.0 < second.1 && second.0 < first.1)
    }) || controllers.len() != 1 {
        return Err(Code::InvalidInterrupt);
    }

    let timers: Vec<_> = semantic_records.iter().filter(|record| record.0 == 4).collect();
    let expected_timer_model = if adapter.expected_architecture == 1 { 1 } else { 2 };
    if timers.len() != 1 {
        return Err(Code::InvalidTimer);
    }
    for timer in timers {
        let controller = u32_at(&rhd, timer.2 + 36).ok_or(Code::InvalidTimer)?;
        let count = *controllers.get(&controller).ok_or(Code::InvalidTimer)?;
        if u16_at(&rhd, timer.2 + 16) != Some(expected_timer_model)
            || u16_at(&rhd, timer.2 + 18) != Some(0)
            || u64_at(&rhd, timer.2 + 24).unwrap_or(0) == 0
            || u32_at(&rhd, timer.2 + 32).unwrap_or(u32::MAX) >= count
            || !rhd.get(timer.2 + 40).is_some_and(|value| matches!(*value, 1 | 2))
            || !rhd.get(timer.2 + 41).is_some_and(|value| matches!(*value, 1 | 2))
        {
            return Err(Code::InvalidTimer);
        }
    }

    let serials: Vec<_> = semantic_records.iter().filter(|record| record.0 == 5).collect();
    let expected_serial_model = if adapter.expected_architecture == 1 { 1 } else { 2 };
    if serials.len() != 1 {
        return Err(Code::InvalidSerial);
    }
    for serial in serials {
        let controller = u32_at(&rhd, serial.2 + 24).ok_or(Code::InvalidSerial)?;
        let count = *controllers.get(&controller).ok_or(Code::InvalidSerial)?;
        if u16_at(&rhd, serial.2 + 16) != Some(expected_serial_model)
            || u16_at(&rhd, serial.2 + 18) != Some(0)
            || u32_at(&rhd, serial.2 + 20).unwrap_or(u32::MAX) >= count
            || !rhd.get(serial.2 + 40).is_some_and(|value| matches!(*value, 1 | 2))
            || !rhd.get(serial.2 + 41).is_some_and(|value| matches!(*value, 1 | 2))
        {
            return Err(Code::InvalidSerial);
        }
    }

    let boot_sources: Vec<_> = semantic_records.iter().filter(|record| record.0 == 6).collect();
    if boot_sources.len() != 1
        || u16_at(&rhd, boot_sources[0].2 + 16) != Some(1)
        || u16_at(&rhd, boot_sources[0].2 + 18) != Some(0)
        || u16_at(&handoff, 24) != Some(1)
    {
        return Err(Code::InvalidBootSource);
    }

    let entropy_flags = u32_at(&handoff, 76).ok_or(Code::InvalidEntropy)?;
    if !matches!(u32_at(&handoff, 72), Some(32..=64))
        || u64::from(u32_at(&handoff, 72).unwrap_or(0)) != entropy_descriptor.length
        || u64_at(&handoff, 64) != Some(entropy_descriptor.base)
        || entropy_flags & !1 != 0
        || entropy_flags == 0 && (entropy_descriptor.rights != 1 || entropy_descriptor.transfer != 1)
        || entropy_flags == 1 && (entropy_descriptor.rights != 3 || entropy_descriptor.transfer != 4)
    {
        return Err(Code::InvalidEntropy);
    }
    if !matches!(u32_at(&handoff, 96), Some(4096..=1_048_576))
        || u64::from(u32_at(&handoff, 96).unwrap_or(0)) != trace_descriptor.length
        || u64_at(&handoff, 88) != Some(trace_descriptor.base)
        || u16_at(&handoff, 84) != Some(1)
        || u16_at(&handoff, 86) != Some(0)
        || u32_at(&handoff, 100) != Some(0)
    {
        return Err(Code::InvalidTrace);
    }

    if records.windows(2).any(|pair| (pair[0].0, pair[0].1) >= (pair[1].0, pair[1].1)) {
        return Err(Code::NoncanonicalOrder);
    }

    let mut rhd_memory = BTreeMap::new();
    for record in semantic_records.iter().filter(|record| record.0 == 2) {
        let memory = Memory {
            id: record.1,
            base: u64_at(&rhd, record.2 + 16).unwrap_or(0),
            length: u64_at(&rhd, record.2 + 24).unwrap_or(0),
            kind: u16_at(&rhd, record.2 + 32).unwrap_or(0),
            attributes: u16_at(&rhd, record.2 + 34).unwrap_or(0),
            owner: u16_at(&rhd, record.2 + 36).unwrap_or(0),
        };
        rhd_memory.insert(record.1, memory);
    }
    if rhd_memory != map_by_id {
        return Err(Code::InconsistentDescription);
    }

    let windows: Vec<_> = semantic_records.iter().filter(|record| record.0 == 7).map(|record| Window {
        parent_kind: u16_at(&rhd, record.2 + 16).unwrap_or(0),
        role: u16_at(&rhd, record.2 + 18).unwrap_or(0),
        parent_id: u32_at(&rhd, record.2 + 20).unwrap_or(0),
        space: u16_at(&rhd, record.2 + 24).unwrap_or(0),
        width: *rhd.get(record.2 + 26).unwrap_or(&0),
        byte_order: *rhd.get(record.2 + 27).unwrap_or(&0),
        stride: u16_at(&rhd, record.2 + 28).unwrap_or(0),
        flags: u16_at(&rhd, record.2 + 30).unwrap_or(0),
        base: u64_at(&rhd, record.2 + 32).unwrap_or(0),
        length: u64_at(&rhd, record.2 + 40).unwrap_or(0),
        authority_purpose: u16_at(&rhd, record.2 + 48).unwrap_or(0),
    }).collect();
    let parent_models: BTreeMap<(u16, u32), u16> = semantic_records.iter().filter(|record| matches!(record.0, 3 | 5)).map(|record| {
        ((record.0, record.1), u16_at(&rhd, record.2 + 16).unwrap_or(0))
    }).collect();
    for window in &windows {
        let parent_model = parent_models.get(&(window.parent_kind, window.parent_id)).copied().unwrap_or(0);
        let common = window.byte_order == 1
            && window.flags == 1
            && window.length != 0
            && window.width.is_power_of_two()
            && window.stride.is_power_of_two()
            && u16::from(window.width) <= window.stride
            && u64::from(window.stride) <= window.length;
        let model_valid = match (window.parent_kind, parent_model, window.role, window.space, window.width, window.stride) {
            (3, 1, 1, 1, 4, 16) if adapter.expected_architecture == 1 => true,
            (3, 2, 2 | 3, 1, 4, 4) if adapter.expected_architecture == 2 => true,
            (5, 1, 5, 1 | 2, 1, 1) if adapter.expected_architecture == 1 => true,
            (5, 2, 5, 1, 4, 4) if adapter.expected_architecture == 2 => true,
            _ => false,
        };
        let window_end = checked_end(window.base, window.length).map_err(|_| Code::InvalidRegisterWindow)?;
        if !common
            || !model_valid
            || window.space == 2 && window_end > 65_536
        {
            return Err(Code::InvalidRegisterWindow);
        }
    }
    for (&parent, &model) in &parent_models {
        let roles: Vec<_> = windows.iter().filter(|window| (window.parent_kind, window.parent_id) == parent).map(|window| window.role).collect();
        let expected: &[u16] = match (parent.0, model) { (3, 1) => &[1], (3, 2) => &[2, 3], (5, _) => &[5], _ => &[] };
        if roles.len() != expected.len() || expected.iter().any(|role| roles.iter().filter(|actual| *actual == role).count() != 1) {
            return Err(Code::InvalidRegisterWindow);
        }
    }

    for first in 0..windows.len() {
        for second in first + 1..windows.len() {
            if windows[first].space == windows[second].space
                && overlaps((windows[first].base, windows[first].length), (windows[second].base, windows[second].length))?
            {
                return Err(Code::UnauthorizedDeviceWindow);
            }
        }
    }
    let mut used_authorities = BTreeSet::new();
    for window in &windows {
        let expected_purpose = if window.space == 1 { 6 } else { 7 };
        let window_end = checked_end(window.base, window.length)?;
        let matches: Vec<_> = descriptors.iter().filter(|descriptor| {
            descriptor.selector.purpose == expected_purpose
                && window.authority_purpose == expected_purpose
                && descriptor.selector.owner_kind == window.parent_kind
                && descriptor.selector.owner_id == window.parent_id
                && descriptor.rights == 11
                && descriptor.transfer == 3
                && descriptor.base <= window.base
                && checked_end(descriptor.base, descriptor.length).is_ok_and(|authority_end| window_end <= authority_end)
        }).collect();
        if matches.len() != 1 {
            return Err(Code::UnauthorizedDeviceWindow);
        }
        let authority = matches[0];
        let identity = AuthorityIdentity { selector: authority.selector, base: authority.base, length: authority.length };
        if !used_authorities.insert(identity) { return Err(Code::UnauthorizedDeviceWindow); }
        if window.space == 1 && !map_by_base.iter().any(|memory| {
            memory.kind == 5
                && memory.attributes == 19
                && memory.owner == 5
                && memory.base <= window.base
                && checked_end(memory.base, memory.length).is_ok_and(|end| checked_end(window.base, window.length).is_ok_and(|window_end| window_end <= end))
        }) {
            return Err(Code::UnauthorizedDeviceWindow);
        }
    }
    let declared_authorities: BTreeSet<_> = descriptors.iter().filter(|descriptor| matches!(descriptor.selector.purpose, 6 | 7)).map(|descriptor| {
        AuthorityIdentity { selector: descriptor.selector, base: descriptor.base, length: descriptor.length }
    }).collect();
    if used_authorities != declared_authorities {
        return Err(Code::UnauthorizedDeviceWindow);
    }

    let handoff_architecture = u16_at(&handoff, 20).unwrap_or(0);
    let rhd_architecture = u16_at(&rhd, 24).unwrap_or(0);
    if adapter.expected_architecture != entry_architecture
        || adapter.expected_architecture != handoff_architecture
        || adapter.expected_architecture != rhd_architecture
    {
        return Err(Code::ArchitectureMismatch);
    }
    let handoff_address_bits = *handoff.get(22).unwrap_or(&0);
    let rhd_address_bits = *rhd.get(26).unwrap_or(&0);
    if adapter.address_bits != entry_address_bits || adapter.address_bits != handoff_address_bits || adapter.address_bits != rhd_address_bits {
        return Err(Code::InconsistentDescription);
    }
    let handoff_page_shift = *handoff.get(23).unwrap_or(&0);
    let rhd_page_shift = *rhd.get(27).unwrap_or(&0);
    if !(12..=30).contains(&handoff_page_shift)
        || handoff_page_shift != rhd_page_shift
        || 1u64.checked_shl(u32::from(handoff_page_shift)) != Some(adapter.page_bytes)
    {
        return Err(Code::PageSizeMismatch);
    }

    if entropy_flags == 1 {
        sink.commit(Effect::ClearEntropy(entropy_descriptor.selector));
    }
    sink.commit(Effect::ActivateTrace(trace_descriptor.selector));
    for identity in used_authorities {
        sink.commit(Effect::ConstructAuthority(identity));
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

fn access_name(key: AccessKey) -> &'static str {
    match key {
        AccessKey::Entry => "entry",
        AccessKey::Descriptor(key) => match key.purpose {
            1 => "handoff",
            2 => "memory-map",
            3 => "rhd",
            4 => "entropy",
            _ => "unexpected",
        },
    }
}

fn access_log(keys: &[AccessKey]) -> String {
    keys.iter().copied().map(access_name).collect::<Vec<_>>().join(",")
}

fn predicate_access_budget(order: usize) -> &'static str {
    match order {
        1..=4 => "",
        5..=14 => "entry",
        15 => "entry,handoff",
        _ => "entry,handoff,memory-map,rhd,entropy",
    }
}

fn run_bytes(bytes: &[u8], behavior: Behavior) -> Result<(u16, Vec<AccessKey>, Vec<Effect>), String> {
    let bundle = parse_bundle(bytes).ok_or_else(|| "malformed fixture bundle".to_string())?;
    let adapter = adapter_for(&bundle, behavior);
    let mut provider = SourceProvider::new(&bundle, behavior);
    let mut sink = EffectSink::default();
    let result = catch_unwind(AssertUnwindSafe(|| validate(adapter, &mut provider, &mut sink)))
        .map_err(|_| "validation panicked".to_string())?;
    let code = result.map_or_else(|code| code as u16, |()| 0);
    Ok((code, provider.reads, sink.effects))
}

fn predicate_rows(fields: &str) -> Vec<(usize, String, String)> {
    fields.lines().filter(|line| line.starts_with("validation-predicate|")).map(|line| {
        let columns: Vec<_> = line.split('|').collect();
        (
            columns[1].parse().expect("predicate order"),
            format!("{}.{}.{}", columns[2], columns[3], columns[4]),
            columns[5].to_string(),
        )
    }).collect()
}

fn check_precedence(root: &Path) -> Result<(), String> {
    let fields = fs::read_to_string(root.join("../../boot/handoff-v1.fields")).map_err(|error| error.to_string())?;
    let fixture = fs::read_to_string(root.join("validation-precedence.v1")).map_err(|error| error.to_string())?;
    let predicates = predicate_rows(&fields);
    if predicates.len() != 37 { return Err("canonical predicate count is not 37".into()); }
    for architecture in ["x86_64", "aarch64"] {
      let baseline = fs::read(root.join(format!("bin/valid-{architecture}.bin"))).map_err(|error| error.to_string())?;
      for (index, (order, predicate, expected)) in predicates.iter().enumerate() {
        let declaration = format!("single|{order}|{predicate}|{expected}");
        if !fixture.lines().any(|line| line == declaration) { return Err(format!("missing executable mutation declaration: {declaration}")); }
        let mut bytes = baseline.clone();
        let mut behavior = Behavior::default();
        apply_mutation(&mut bytes, predicate, &mut behavior);
        let (actual, reads, effects) = run_bytes(&bytes, behavior)?;
        if expected_name(actual) != expected { return Err(format!("single {predicate} on {architecture} expected {expected}, got {}", expected_name(actual))); }
        if access_log(&reads) != predicate_access_budget(*order) { return Err(format!("single {predicate} on {architecture} exceeded access budget: {}", access_log(&reads))); }
        if actual != 0 && !effects.is_empty() { return Err(format!("single {predicate} on {architecture} produced rejected effects")); }
        if index + 1 < predicates.len() {
            let (_, next, _) = &predicates[index + 1];
            let dual = format!("dual|{predicate}|{next}|{expected}");
            if !fixture.lines().any(|line| line == dual) { return Err(format!("missing executable dual declaration: {dual}")); }
            let mut bytes = baseline.clone();
            let mut behavior = Behavior::default();
            apply_mutation(&mut bytes, next, &mut behavior);
            apply_mutation(&mut bytes, predicate, &mut behavior);
            let (actual, reads, effects) = run_bytes(&bytes, behavior)?;
            if expected_name(actual) != expected { return Err(format!("dual {predicate} + {next} on {architecture} expected {expected}, got {}", expected_name(actual))); }
            if access_log(&reads) != predicate_access_budget(*order) { return Err(format!("dual {predicate} + {next} on {architecture} exceeded access budget: {}", access_log(&reads))); }
            if !effects.is_empty() { return Err(format!("dual {predicate} + {next} on {architecture} produced rejected effects")); }
        }
      }
    }

    for line in fixture.lines().filter(|line| line.starts_with("security-dual|")) {
        let columns: Vec<_> = line.split('|').collect();
        if columns.len() != 4 { return Err(format!("malformed security pair: {line}")); }
        for architecture in ["x86_64", "aarch64"] {
            let mut bytes = fs::read(root.join(format!("bin/valid-{architecture}.bin"))).map_err(|error| error.to_string())?;
            let mut behavior = Behavior::default();
            apply_mutation(&mut bytes, columns[2], &mut behavior);
            apply_mutation(&mut bytes, columns[1], &mut behavior);
            let (actual, reads, effects) = run_bytes(&bytes, behavior)?;
            if expected_name(actual) != columns[3] { return Err(format!("security pair {} + {} on {architecture} expected {}, got {}", columns[1], columns[2], columns[3], expected_name(actual))); }
            let order = predicates.iter().find(|row| row.1 == columns[1]).map(|row| row.0).ok_or_else(|| format!("unknown security predicate {}", columns[1]))?;
            if access_log(&reads) != predicate_access_budget(order) { return Err(format!("security pair on {architecture} exceeded access budget: {}", access_log(&reads))); }
            if !effects.is_empty() { return Err(format!("security pair on {architecture} produced rejected effects")); }
        }
    }
    Ok(())
}

fn check_scenarios(root: &Path) -> Result<usize, String> {
    let manifest = fs::read_to_string(root.join("conformance-scenarios.v1")).map_err(|error| error.to_string())?;
    let mut count = 0usize;
    for line in manifest.lines() {
        if line.is_empty() || line.starts_with("schema=") || line.starts_with("id|") { continue; }
        let columns: Vec<_> = line.split('|').collect();
        if columns.len() != 7 { return Err(format!("malformed scenario row: {line}")); }
        let mut bytes = fs::read(root.join(format!("bin/valid-{}.bin", columns[1]))).map_err(|error| error.to_string())?;
        let mut behavior = Behavior::default();
        for mutation in columns[2].split(',').filter(|mutation| *mutation != "none") {
            match mutation {
                "short-handoff" => behavior.short_purpose = Some(1),
                "short-rhd" => behavior.short_purpose = Some(3),
                "fault-map" => behavior.fault_purpose = Some(2),
                "fault-entropy" => behavior.fault_purpose = Some(4),
                other => apply_mutation(&mut bytes, other, &mut behavior),
            }
        }
        let (actual, reads, effects) = run_bytes(&bytes, behavior)?;
        if expected_name(actual) != columns[3] { return Err(format!("{} expected {}, got {}", columns[0], columns[3], expected_name(actual))); }
        if columns[6] == "single-copy" {
            let mut seen = BTreeSet::new();
            if reads.iter().any(|key| !seen.insert(*key)) { return Err(format!("{} repeated a provider copy", columns[0])); }
        }
        let read_names = access_log(&reads);
        if read_names != columns[4] { return Err(format!("{} expected reads {}, got {read_names}", columns[0], columns[4])); }
        let effect_class = if effects.is_empty() { "none" } else { "commit" };
        if effect_class != columns[5] { return Err(format!("{} expected effects {}, got {effect_class}", columns[0], columns[5])); }
        count += 1;
    }
    Ok(count)
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
        match run_bytes(&bytes, Behavior::default()) {
            Ok((actual, _, effects)) => {
                let bundle = parse_bundle(&bytes).expect("fixture bundle");
                if bundle.expected != actual || fields[1] != expected_name(actual) {
                    eprintln!("{}: expected {}, got {}", fields[0], fields[1], expected_name(actual));
                    failures += 1;
                }
                if actual != 0 && !effects.is_empty() {
                    eprintln!("{}: rejected fixture produced effects", fields[0]);
                    failures += 1;
                }
            }
            Err(error) => { eprintln!("{}: {error}", fields[0]); failures += 1; }
        }
        count += 1;
    }
    if let Err(error) = check_precedence(&root) { eprintln!("{error}"); failures += 1; }
    let scenario_count = match check_scenarios(&root) { Ok(value) => value, Err(error) => { eprintln!("{error}"); failures += 1; 0 } };
    if count != 23 { eprintln!("fixture count is {count}, expected 23"); failures += 1; }
    if scenario_count < 24 { eprintln!("scenario count is {scenario_count}, expected at least 24"); failures += 1; }
    if failures != 0 { std::process::exit(1); }
    println!("R0-002 conformance passed: {count} raw fixtures, {scenario_count} scenarios, 37 singles x 2 architectures, 36 adjacent edges x 2 architectures");
}
