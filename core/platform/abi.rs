//! Private Platform-v0 fixture ABI; not a stable application API.
pub const BOOT_ADDRESS:usize=0x700000;
pub const MAGIC:u64=0x524152504c415430;
pub const YIELD:u64=0;
pub const SEND:u64=1;
pub const RECEIVE:u64=2;
pub const PORT_READ:u64=3;
pub const REPORT:u64=4;
pub const EXIT:u64=5;
pub const SELF_RECV:usize=0;
pub const STORAGE:usize=1;
pub const CLIENT:usize=2;
pub const SECOND_CLIENT:usize=3;
pub const INPUT:usize=4;
pub const FRAMEBUFFER:usize=5;
pub const SELF_SEND:usize=6;
pub const STALE:usize=7;
pub const DEAD_PEER:usize=8;
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Boot {
    pub magic:u64,pub role:u64,pub generation:u64,pub entry:u64,pub kernel_probe:u64,pub peer_probe:u64,
    pub framebuffer:u64,pub width:u64,pub height:u64,pub pitch:u64,pub format:u64,
    pub caps:[u64;10],
}
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Envelope {pub sender:u64,pub generation:u32,pub length:u32,pub bytes:[u8;128]}
impl Envelope {pub const EMPTY:Self=Self{sender:0,generation:0,length:0,bytes:[0;128]};}
