//! GENERATED FILE — DO NOT EDIT.
//! Sources: spec/boot/handoff-v1.fields and spec/hardware/rhd-v1.fields.
//! These are owned semantic types. Rust layout is not the wire format.

#![no_std]
#![deny(unsafe_code)]

pub const ENTRY_MAGIC: [u8; 8] = *b"RARENTRY";

pub const ENTRY_VERSION_MAJOR: u16 = 1;

pub const ENTRY_VERSION_MINOR: u16 = 0;

pub const ENTRY_V1_HEADER_BYTES: u16 = 64;

pub const WINDOW_DESCRIPTOR_V1_BYTES: u16 = 32;

pub const MAX_ENTRY_BYTES: u32 = 4_096;

pub const MAX_WINDOW_DESCRIPTORS: u16 = 126;

pub const ADAPTER_ENTRY_ALIGNMENT: u16 = 8;

pub const ADAPTER_STACK_ALIGNMENT: u16 = 16;

pub const BOOT_MAGIC: [u8; 8] = *b"RARBOOT\0";

pub const BOOT_VERSION_MAJOR: u16 = 1;

pub const BOOT_VERSION_MINOR: u16 = 0;

pub const BOOT_V1_BYTES: u16 = 128;

pub const MEMORY_MAP_V1_ENTRY_BYTES: u16 = 32;

pub const MAX_HANDOFF_BYTES: u32 = 4_096;

pub const MAX_HANDOFF_SOURCE_BYTES: u32 = 4_096;

pub const MAX_MEMORY_MAP_ENTRIES: u32 = 256;

pub const MAX_MEMORY_MAP_SOURCE_BYTES: u32 = 8_192;

pub const MAX_RHD_SOURCE_BYTES: u32 = 65_536;

pub const MIN_ENTROPY_BYTES: u32 = 32;

pub const MAX_ENTROPY_BYTES: u32 = 64;

pub const MAX_ENTROPY_SOURCE_BYTES: u32 = 64;

pub const MIN_TRACE_BYTES: u32 = 4_096;

pub const MAX_TRACE_BYTES: u32 = 1_048_576;

pub const MAX_TRACE_SOURCE_BYTES: u32 = 1_048_576;

pub const INERT_DESCRIPTOR_BASE: u64 = 0;

pub const INERT_DESCRIPTOR_LENGTH: u64 = 0;

pub const INERT_DESCRIPTOR_RIGHTS: u16 = 0;

pub const INERT_DESCRIPTOR_TRANSFER: u16 = 0;

pub const INERT_DESCRIPTOR_OWNER_KIND: u16 = 0;

pub const INERT_DESCRIPTOR_FLAGS: u16 = 0;

pub const INERT_DESCRIPTOR_OWNER_ID: u32 = 0;

pub const RIGHT_READ: u16 = 1;

pub const RIGHT_WRITE: u16 = 2;

pub const RIGHT_EXECUTE: u16 = 4;

pub const RIGHT_DEVICE: u16 = 8;

pub const WINDOW_IMMUTABLE: u16 = 1;

pub const WINDOW_DMA_REVOKED: u16 = 2;

pub const WINDOW_CRITICAL: u16 = 4;

pub const ENTROPY_CLEAR_AFTER_SNAPSHOT: u32 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Architecture {
    X86_64 = 1,
    Aarch64 = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum DescriptorPurpose {
    Handoff = 1,
    MemoryMap = 2,
    Rhd = 3,
    Entropy = 4,
    Trace = 5,
    DeviceMmio = 6,
    DeviceIoPort = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum DescriptorRights {
    Read = 1,
    Write = 2,
    Execute = 4,
    Device = 8,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum Producer {
    Root = 1,
    Recovery = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum TransferMode {
    Snapshot = 1,
    Exclusive = 2,
    Authority = 3,
    ClearAfterSnapshot = 4,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum MemoryKind {
    Usable = 1,
    Firmware = 2,
    BootOwned = 3,
    Nucleus = 4,
    Mmio = 5,
    Reserved = 6,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum MemoryAttribute {
    Read = 1,
    Write = 2,
    Execute = 4,
    Cacheable = 8,
    Device = 16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum MemoryOwner {
    None = 0,
    Root = 1,
    Recovery = 2,
    Nucleus = 3,
    Firmware = 4,
    Device = 5,
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
    InvalidEntry = 30,
    SnapshotViolation = 31,
    InvalidRegisterWindow = 32,
    UnauthorizedDeviceWindow = 33,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ArchitectureAdapterInputsV1 {
    pub expected_architecture: Architecture,
    pub external_entry_address: u64,
    pub external_entry_bytes: u32,
    pub address_bits: u8,
    pub page_bytes: u64,
    pub entry_alignment: u16,
    pub stack_alignment: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DescriptorSelectorV1 {
    pub purpose: DescriptorPurpose,
    pub owner_kind: u16,
    pub owner_id: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootEntryHeaderV1 {
    pub architecture: Architecture,
    pub address_bits: u8,
    pub descriptor_count: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowDescriptorV1 {
    pub selector: DescriptorSelectorV1,
    pub base: u64,
    pub length: u64,
    pub rights: u16,
    pub producer: Producer,
    pub transfer: TransferMode,
    pub flags: u16,
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
    pub rhd_alignment: u16,
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
    pub kind: MemoryKind,
    pub attributes: u16,
    pub owner: MemoryOwner,
    pub region_id: u32,
}
pub const RHD_MAGIC: [u8; 8] = *b"RARRHD\0\0";

pub const RHD_VERSION_MAJOR: u16 = 1;

pub const RHD_VERSION_MINOR: u16 = 0;

pub const RHD_V1_HEADER_BYTES: u16 = 32;

pub const RHD_V1_RECORD_HEADER_BYTES: u16 = 16;

pub const MAX_RHD_BYTES: u32 = 65_536;

pub const MAX_RHD_RECORDS: u16 = 256;

pub const CPU_RECORD_BYTES: u32 = 48;

pub const MEMORY_RECORD_BYTES: u32 = 48;

pub const INTERRUPT_RECORD_BYTES: u32 = 48;

pub const TIMER_RECORD_BYTES: u32 = 48;

pub const SERIAL_RECORD_BYTES: u32 = 48;

pub const BOOT_SOURCE_RECORD_BYTES: u32 = 24;

pub const REGISTER_WINDOW_RECORD_BYTES: u32 = 56;

pub const RECORD_CRITICAL: u16 = 1;

pub const X86_GLOBAL_INTERRUPT_LIMIT: u32 = 256;

pub const AARCH64_GLOBAL_INTERRUPT_LIMIT: u32 = 1_020;

pub const CPU_BOOT: u32 = 1;

pub const WINDOW_REQUIRED: u16 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum RecordKind {
    Cpu = 1,
    Memory = 2,
    Interrupt = 3,
    Timer = 4,
    Serial = 5,
    BootSource = 6,
    RegisterWindow = 7,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum InterruptModel {
    X86Apic = 1,
    ArmGicV3 = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum TimerModel {
    X86Architectural = 1,
    ArmGeneric = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum SerialModel {
    Compatible16550 = 1,
    Pl011Compatible = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum BootSourceKind {
    BootVolume = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum AddressSpace {
    SystemMemory = 1,
    X86IoPort = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u16)]
pub enum RegisterWindowRole {
    Apic = 1,
    GicDistributor = 2,
    GicRedistributor = 3,
    Timer = 4,
    Serial = 5,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum WireByteOrder {
    LittleEndian = 1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterruptTrigger {
    Edge = 1,
    Level = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum InterruptPolarity {
    ActiveHigh = 1,
    ActiveLow = 2,
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
    pub kind: RecordKind,
    pub flags: u16,
    pub record_bytes: u32,
    pub record_id: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CpuRecordV1 {
    pub flags: u32,
    pub hardware_id: u64,
    pub interrupt_controller_id: u32,
    pub timer_id: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRecordV1 {
    pub base: u64,
    pub length: u64,
    pub kind: MemoryKind,
    pub attributes: u16,
    pub owner: MemoryOwner,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InterruptRecordV1 {
    pub model: InterruptModel,
    pub flags: u16,
    pub global_interrupt_base: u32,
    pub interrupt_count: u32,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimerRecordV1 {
    pub model: TimerModel,
    pub flags: u16,
    pub frequency_hz: u64,
    pub interrupt_index: u32,
    pub interrupt_controller_id: u32,
    pub interrupt_trigger: InterruptTrigger,
    pub interrupt_polarity: InterruptPolarity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SerialRecordV1 {
    pub model: SerialModel,
    pub flags: u16,
    pub interrupt_index: u32,
    pub interrupt_controller_id: u32,
    pub input_clock_hz: u64,
    pub interrupt_trigger: InterruptTrigger,
    pub interrupt_polarity: InterruptPolarity,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BootSourceRecordV1 {
    pub kind: BootSourceKind,
    pub flags: u16,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegisterWindowRecordV1 {
    pub parent_kind: RecordKind,
    pub role: u16,
    pub parent_id: u32,
    pub address_space: AddressSpace,
    pub access_width: u8,
    pub byte_order: WireByteOrder,
    pub stride: u16,
    pub flags: u16,
    pub base: u64,
    pub length: u64,
    pub authority_selector: DescriptorSelectorV1,
}
