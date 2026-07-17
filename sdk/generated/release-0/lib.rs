//! GENERATED FILE — DO NOT EDIT.
//! Sources: spec/hardware/rhd-v1.fields and spec/boot/handoff-v1.fields.
//! These are owned semantic types. Rust layout is not the wire format.

#![no_std]
#![deny(unsafe_code)]

pub const BOOT_MAGIC: [u8; 8] = *b"RARBOOT\0";
pub const RHD_MAGIC: [u8; 8] = *b"RARRHD\0\0";
pub const BOOT_V1_BYTES: u16 = 128;
pub const RHD_V1_HEADER_BYTES: u16 = 32;
pub const RHD_V1_RECORD_HEADER_BYTES: u16 = 16;
pub const MEMORY_MAP_V1_ENTRY_BYTES: u16 = 32;
pub const MAX_HANDOFF_BYTES: u32 = 4_096;
pub const MAX_RHD_BYTES: u32 = 65_536;
pub const MAX_RHD_RECORDS: u16 = 256;
pub const MAX_MEMORY_MAP_ENTRIES: u32 = 256;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Architecture {
    X86_64 = 1,
    Aarch64 = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ValidationCode {
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
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootHandoffV1 {
    pub architecture: Architecture,
    pub address_bits: u8,
    pub page_shift: u8,
    pub boot_source_kind: u16,
    pub memory_map_address: u64,
    pub memory_map_count: u32,
    pub rhd_address: u64,
    pub rhd_bytes: u32,
    pub entropy_address: u64,
    pub entropy_bytes: u32,
    pub entropy_flags: u32,
    pub trace_channel_id: u32,
    pub trace_major: u16,
    pub trace_minor: u16,
    pub trace_buffer_address: u64,
    pub trace_buffer_bytes: u32,
    pub trace_flags: u32,
    pub boot_cpu_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRegionV1 {
    pub base: u64,
    pub length: u64,
    pub kind: u16,
    pub attributes: u16,
    pub owner: u16,
    pub region_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RhdHeaderV1 {
    pub total_bytes: u32,
    pub record_count: u16,
    pub architecture: Architecture,
    pub address_bits: u8,
    pub page_shift: u8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordHeaderV1 {
    pub kind: u16,
    pub flags: u16,
    pub record_bytes: u32,
    pub record_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuRecordV1 {
    pub cpu_id: u32,
    pub flags: u32,
    pub hardware_id: u64,
    pub interrupt_controller_id: u32,
    pub timer_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptRecordV1 {
    pub controller_id: u32,
    pub model: u16,
    pub flags: u16,
    pub register_base: u64,
    pub register_bytes: u32,
    pub global_interrupt_base: u32,
    pub interrupt_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerRecordV1 {
    pub timer_id: u32,
    pub model: u16,
    pub flags: u16,
    pub frequency_hz: u64,
    pub register_base: u64,
    pub interrupt: u32,
    pub interrupt_controller_id: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerialRecordV1 {
    pub serial_id: u32,
    pub model: u16,
    pub flags: u16,
    pub register_base: u64,
    pub register_bytes: u16,
    pub interrupt: u16,
    pub interrupt_controller_id: u32,
    pub input_clock_hz: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootSourceRecordV1 {
    pub kind: u16,
    pub flags: u16,
    pub stable_source_id: u32,
}
