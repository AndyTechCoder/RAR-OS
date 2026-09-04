//! Single-CPU descriptor, exception and legacy timer foundations for q35.
use core::{arch::{asm,naked_asm},ptr,sync::atomic::{AtomicU64,Ordering}};
use crate::{boot,out,fatal,record};
static TICKS:AtomicU64=AtomicU64::new(0);
pub fn ticks()->u64 {TICKS.load(Ordering::Acquire)}
#[repr(C,packed)]
struct Descriptor {limit:u16,base:u64}
#[repr(C,packed)]
#[derive(Clone,Copy)]
struct Gate {low:u16,selector:u16,ist:u8,attrs:u8,middle:u16,high:u32,reserved:u32}
impl Gate {
    const EMPTY:Self=Self{low:0,selector:0,ist:0,attrs:0,middle:0,high:0,reserved:0};
    fn new(address:u64,ist:u8)->Self {
        Self{low:address as u16,selector:8,ist,attrs:0x8e,middle:(address>>16) as u16,
             high:(address>>32) as u32,reserved:0}
    }
}
#[repr(C,packed)]
struct TaskState {reserved0:u32,rsp:[u64;3],reserved1:u64,ist:[u64;7],
                  reserved2:u64,reserved3:u16,iomap:u16}
static mut TSS:TaskState=TaskState{reserved0:0,rsp:[0;3],reserved1:0,ist:[0;7],
    reserved2:0,reserved3:0,iomap:104};
#[repr(C,align(16))]
struct GateTable([Gate;256]);
static mut IDT:GateTable=GateTable([Gate::EMPTY;256]);
static mut GDT:[u64;5]=[0,0x00af9a000000ffff,0x00cf92000000ffff,0,0];

#[unsafe(naked)]
unsafe extern "sysv64" fn unexpected()->! {
    naked_asm!("cli","cld","and rsp, -16","call {handler}","ud2",handler=sym unexpected_rust)
}
extern "sysv64" fn unexpected_rust()->! {fatal("RAR-PANIC:CODE=UNEXPECTED-VECTOR")}
#[unsafe(naked)]
unsafe extern "sysv64" fn invalid_opcode()->! {
    naked_asm!("cli","cld","and rsp, -16","call {handler}","ud2",handler=sym invalid_opcode_rust)
}
extern "sysv64" fn invalid_opcode_rust()->! {
    record("RAR-EXCEPTION:VECTOR=6");fatal("RAR-PANIC:CODE=EXCEPTION-06")
}
#[unsafe(naked)]
unsafe extern "sysv64" fn double_fault()->! {
    naked_asm!("cli","cld","and rsp, -16","call {handler}","ud2",handler=sym double_fault_rust)
}
extern "sysv64" fn double_fault_rust()->! {fatal("RAR-PANIC:CODE=EXCEPTION-08")}
/// The timer stub calls no Rust and touches no SIMD state. Its single aligned
/// 64-bit counter writer runs on the sole CPU with maskable IRQs disabled.
/// It saturates rather than wrapping, preserves RAX and returns with IRETQ.
#[unsafe(naked)]
unsafe extern "sysv64" fn timer() {
    naked_asm!(
        "push rax","mov rax, qword ptr [rip + {ticks}]","cmp rax, -1","je 2f",
        "inc rax","mov qword ptr [rip + {ticks}], rax",
        "2:","mov al, 0x20","out 0x20, al","pop rax","iretq",
        ticks=sym TICKS,
    )
}
/// All data is exclusively initialized with IF=0 before loading GDTR/IDTR.
/// Fatal exceptions use a dedicated guarded IST stack; they never return.
pub unsafe fn install(arena:u64) {
    unsafe {asm!("cli",options(nostack));}
    let tss=ptr::addr_of_mut!(TSS);
    unsafe {ptr::addr_of_mut!((*tss).ist).cast::<u64>().write_unaligned(arena+boot::EMERGENCY_TOP);}
    let base=tss as u64;
    let low=103u64 | ((base&0xffff)<<16) | (((base>>16)&0xff)<<32) |
        (0x89u64<<40) | (((base>>24)&0xff)<<56);
    let gdt=ptr::addr_of_mut!(GDT).cast::<u64>();
    unsafe {gdt.add(3).write(low);gdt.add(4).write(base>>32);}
    let gdtr=Descriptor{limit:39,base:gdt as u64};
    unsafe {
        asm!("lgdt [{table}]","push 8","lea rax, [rip + 2f]","push rax","retfq","2:",
             "mov ax, 16","mov ds, ax","mov es, ax","mov ss, ax",
             "xor eax, eax","mov fs, ax","mov gs, ax",
             "mov ax, 24","ltr ax", table=in(reg)&gdtr,out("rax") _);
    }
    let gates=ptr::addr_of_mut!(IDT).cast::<Gate>();
    for i in 0..256 {
        unsafe {gates.add(i).write(Gate::new(unexpected as *const () as u64,1));}
    }
    unsafe {
        gates.add(6).write(Gate::new(invalid_opcode as *const () as u64,1));
        gates.add(8).write(Gate::new(double_fault as *const () as u64,1));
        gates.add(32).write(Gate::new(timer as *const () as u64,0));
    }
    let idtr=Descriptor{limit:4095,base:gates as u64};
    unsafe {asm!("lidt [{}]",in(reg)&idtr,options(readonly,nostack));}
    // Validate the installed descriptor registers before permitting interrupts.
    let mut observed=Descriptor{limit:0,base:0};
    unsafe {asm!("sidt [{}]",in(reg)&mut observed,options(nostack));}
    if observed.base!=gates as u64 || observed.limit!=4095 {fatal("RAR-PANIC:CODE=IDT");}
    unsafe {
        out(0x21,0xff);out(0xa1,0xff);
        out(0x20,0x11);out(0xa0,0x11);
        out(0x21,0x20);out(0xa1,0x28);
        out(0x21,4);out(0xa1,2);
        out(0x21,1);out(0xa1,1);
        out(0x21,0xff);out(0xa1,0xff);
    }
}
pub unsafe fn start_timer() {
    // 1,193,182 Hz / 11,932 ~ 100 Hz; sufficient for bootstrap tick evidence.
    unsafe {out(0x43,0x36);out(0x40,11932u16 as u8);out(0x40,(11932u16>>8) as u8);
            out(0x21,0xfe);out(0xa1,0xff);}
}
