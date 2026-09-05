#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod model;
mod boot;
mod paging;
mod interrupts;
#[cfg(rar_platform)]
#[path="../platform/main.rs"]
mod platform;
use core::{arch::{asm,naked_asm}, panic::PanicInfo, sync::atomic::{AtomicUsize,Ordering}};

static LOG_BYTES: AtomicUsize=AtomicUsize::new(0);
const SERIAL_LIMIT:usize=4096;
pub unsafe fn out(port:u16,byte:u8) {
    // SAFETY: caller confines ports to the fixed cloud platform's UART/PIC/PIT.
    unsafe { asm!("out dx, al", in("dx")port,in("al")byte,options(nomem,nostack,preserves_flags)); }
}
pub unsafe fn input(port:u16)->u8 {
    let value:u8;
    // SAFETY: caller confines port reads to UART status.
    unsafe { asm!("in al, dx",in("dx")port,out("al")value,options(nomem,nostack,preserves_flags)); }
    value
}
pub fn record(text:&str) {
    if text.len()>96 || !text.bytes().all(|b|b.is_ascii_graphic()) { halt(); }
    let length=text.len()+1;
    if LOG_BYTES.fetch_add(length,Ordering::Relaxed)+length>SERIAL_LIMIT { halt(); }
    for byte in text.bytes().chain(core::iter::once(b'\n')) {
        let mut ready=false;
        for _ in 0..100_000 {
            // SAFETY: fixed COM1 transmit status, single bootstrap writer.
            if unsafe {input(0x3fd)}&0x20!=0 {ready=true;break;}
            core::hint::spin_loop();
        }
        if !ready {halt();}
        unsafe {out(0x3f8,byte);}
    }
}
pub fn halt()->! {
    loop { unsafe {asm!("cli","hlt",options(nomem,nostack));} }
}
pub fn fatal(code:&str)->! {
    unsafe {asm!("cli",options(nomem,nostack));}
    record("RAR-PANIC:BEGIN");
    record(code);
    record("RAR-PANIC:HALT");
    halt()
}
#[panic_handler]
fn panic(_: &PanicInfo)->! {fatal("RAR-PANIC:CODE=RUST")}
#[unsafe(no_mangle)]
pub unsafe extern "efiapi" fn efi_main(image:usize, system:*const boot::SystemTable)->usize {
    // SAFETY: firmware entry contract supplies valid UEFI handles/table.
    unsafe {
        out(0x3f9,0); out(0x3fb,0x80); out(0x3f8,1); out(0x3f9,0);
        out(0x3fb,3); out(0x3fa,0xc7); out(0x3fc,0x0b);
        record("RAR-BOOT:UEFI");
        boot::start(image,system)
    }
}
/// Switch stacks and address space without leaving compiler stack temporaries.
/// SysV input registers are fixed; all called code and BootInfo remain mapped.
#[unsafe(naked)]
pub unsafe extern "sysv64" fn enter(cr3:u64, stack:u64, info:*const boot::BootInfo)->! {
    naked_asm!(
        "cli", "mov cr3, rdi", "mov rsp, rsi", "and rsp, -16",
        "xor rbp, rbp", "mov rdi, rdx", "call {entry}", "ud2",
        entry=sym kernel_entry,
    )
}
extern "sysv64" fn kernel_entry(info:*const boot::BootInfo)->! {
    record("RAR-KERNEL:ENTRY");
    // SAFETY: the boot adapter owns and mapped the immutable handoff object.
    let info=unsafe {&*info};
    if info.magic!=boot::MAGIC || info.count==0 || info.count>model::MAX_REGIONS || info.table_used==0 || info.table_used>256 || info.arena%4096!=0 {fatal("RAR-PANIC:CODE=HANDOFF");}
    let active_root:u64;
    unsafe {asm!("mov {}, cr3",out(reg)active_root,options(nostack));}
    if active_root!=info.arena {fatal("RAR-PANIC:CODE=ADDRESS-SPACE");}
    let mut frames=model::Frames::new(&info.regions[..info.count])
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=MEMORY-MAP"));
    let frame=frames.allocate().unwrap_or_else(|_|fatal("RAR-PANIC:CODE=NO-FRAME"));
    let mut tables=unsafe {paging::Tables::resume(info.arena,info.table_used)};
    let address=0xffff_8000_0010_0000;
    let map=model::Mapping {virtual_start:address,physical_start:frame,pages:1,writable:true,executable:false};
    if unsafe {tables.map(map,frame,frame+4096)}.is_err() {fatal("RAR-PANIC:CODE=MAP");}
    // SAFETY: a fresh owned page is mapped writable NX at the test address.
    unsafe {
        (address as *mut u64).write_volatile(0x5241525f4d454d31);
        if (address as *const u64).read_volatile()!=0x5241525f4d454d31 {fatal("RAR-PANIC:CODE=MAP-READ");}
        tables.unmap(address).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=UNMAP"));
    }
    frames.release_last(frame).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=FRAME-FREE"));
    record("RAR-MEMORY:READY");
    let mut heap=model::Heap::new();
    let a=heap.allocate(33,4096).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=ALLOC"));
    let p=(info.arena+boot::HEAP_OFFSET+a.offset as u64) as *mut u8;
    unsafe {p.write_volatile(0xa5); if p.read_volatile()!=0xa5 {fatal("RAR-PANIC:CODE=HEAP");}}
    heap.deallocate(a).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=FREE"));
    if heap.deallocate(a).is_ok() {fatal("RAR-PANIC:CODE=DOUBLE-FREE");}
    record("RAR-ALLOCATOR:READY");
    if cfg!(rar_profile="panic") {fatal("RAR-PANIC:CODE=SELFTEST");}
    unsafe {interrupts::install(info.arena);}
    record("RAR-INTERRUPTS:READY");
    if cfg!(rar_profile="exception") {unsafe {asm!("ud2",options(noreturn));}}
    unsafe {interrupts::start_timer();}
    // IRQ0 wakes HLT; serial-only harness imposes a wall-clock bound if it fails.
    while interrupts::ticks()<3 {unsafe {asm!("sti","hlt",options(nomem,nostack));}}
    unsafe {asm!("cli",options(nomem,nostack));}
    if interrupts::ticks()<3 {fatal("RAR-PANIC:CODE=TIMER");}
    record("RAR-TIMER:READY");
    record("RAR-FOUNDATION-READY");
    #[cfg(rar_platform)]
    unsafe {platform::start(info)}
    #[cfg(not(rar_platform))]
    halt()
}

// RAR-owned C memory intrinsics. Volatile byte operations prevent the compiler
// from lowering these definitions recursively into themselves. Callers provide
// valid ranges; memmove explicitly supports overlap, memcpy requires disjointness.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(dst:*mut u8,src:*const u8,n:usize)->*mut u8 {
    for i in 0..n {unsafe {dst.add(i).write_volatile(src.add(i).read_volatile());}}
    dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(dst:*mut u8,value:i32,n:usize)->*mut u8 {
    for i in 0..n {unsafe {dst.add(i).write_volatile(value as u8);}}
    dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(dst:*mut u8,src:*const u8,n:usize)->*mut u8 {
    if (dst as usize)<(src as usize) {
        for i in 0..n {unsafe {dst.add(i).write_volatile(src.add(i).read_volatile());}}
    } else {
        for i in (0..n).rev() {unsafe {dst.add(i).write_volatile(src.add(i).read_volatile());}}
    }
    dst
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a:*const u8,b:*const u8,n:usize)->i32 {
    for i in 0..n {
        let x=unsafe {a.add(i).read_volatile()};let y=unsafe {b.add(i).read_volatile()};
        if x!=y {return x as i32-y as i32;}
    }
    0
}
