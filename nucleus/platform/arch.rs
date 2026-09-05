//! Single-CPU ring3 trap boundary. All state mutation occurs with IF=0.
use core::{arch::{asm,naked_asm},ptr};
use crate::{fatal,out};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Trap{
    pub gs:u64,pub fs:u64,pub es:u64,pub ds:u64,
    pub r15:u64,pub r14:u64,pub r13:u64,pub r12:u64,pub r11:u64,pub r10:u64,pub r9:u64,pub r8:u64,
    pub rbp:u64,pub rdi:u64,pub rsi:u64,pub rdx:u64,pub rcx:u64,pub rbx:u64,pub rax:u64,
    pub vector:u64,pub error:u64,pub rip:u64,pub cs:u64,pub flags:u64,pub rsp:u64,pub ss:u64,
}
impl Trap{
    pub const EMPTY:Self=Self{gs:0,fs:0,es:0x23,ds:0x23,r15:0,r14:0,r13:0,r12:0,r11:0,r10:0,r9:0,r8:0,
        rbp:0,rdi:0,rsi:0,rdx:0,rcx:0,rbx:0,rax:0,vector:0,error:0,rip:0,cs:0x1b,flags:0x202,rsp:0,ss:0x23};
}
#[repr(C,packed)]struct Descriptor{limit:u16,base:u64}
#[repr(C,packed)]#[derive(Clone,Copy)]
struct Gate{low:u16,selector:u16,ist:u8,attrs:u8,middle:u16,high:u32,reserved:u32}
impl Gate{
    const EMPTY:Self=Self{low:0,selector:0,ist:0,attrs:0,middle:0,high:0,reserved:0};
    fn new(address:u64,ist:u8,user:bool)->Self{Self{low:address as u16,selector:8,ist,attrs:if user{0xee}else{0x8e},
        middle:(address>>16)as u16,high:(address>>32)as u32,reserved:0}}
}
#[repr(C,packed)]
struct Tss{reserved0:u32,rsp:[u64;3],reserved1:u64,ist:[u64;7],reserved2:u64,reserved3:u16,iomap:u16}
static mut TSS:Tss=Tss{reserved0:0,rsp:[0;3],reserved1:0,ist:[0;7],reserved2:0,reserved3:0,iomap:104};
static mut GDT:[u64;7]=[0,0x00af9a000000ffff,0x00cf92000000ffff,0x00affa000000ffff,0x00cff2000000ffff,0,0];
#[repr(C,align(16))]struct Gates([Gate;256]);
static mut IDT:Gates=Gates([Gate::EMPTY;256]);
static MXCSR:u32=0x1f80;

/// Hardware RSP0 is 16-aligned. The normalized 208-byte Trap and 512-byte FXSAVE
/// block are contiguous/aligned. Every user GPR, data selector, x87 and XMM state
/// is saved before Rust can run. IF remains clear; DF/MXCSR/x87 are sanitized for
/// the kernel. The returned frame belongs to an owned guarded kernel stack.
#[unsafe(naked)]
unsafe extern "sysv64" fn common()->!{
    naked_asm!(
        "cld",
        "push rax","push rbx","push rcx","push rdx","push rsi","push rdi","push rbp",
        "push r8","push r9","push r10","push r11","push r12","push r13","push r14","push r15",
        "xor eax,eax","mov ax,ds","push rax","mov ax,es","push rax","mov ax,fs","push rax","mov ax,gs","push rax",
        "sub rsp,512","fxsave64 [rsp]","fninit","ldmxcsr [rip + {mxcsr}]",
        "lea rdi,[rsp+512]","mov rsi,rsp","call {handler}",
        "mov rsp,rax","jmp {restore}",
        mxcsr=sym MXCSR,handler=sym super::trap,restore=sym restore,
    )
}
#[unsafe(naked)]
unsafe extern "sysv64" fn restore()->!{
    naked_asm!(
        "fxrstor64 [rsp]","add rsp,512",
        "pop rax","mov gs,ax","pop rax","mov fs,ax","pop rax","mov es,ax","pop rax","mov ds,ax",
        "pop r15","pop r14","pop r13","pop r12","pop r11","pop r10","pop r9","pop r8",
        "pop rbp","pop rdi","pop rsi","pop rdx","pop rcx","pop rbx","pop rax",
        "add rsp,16","iretq",
    )
}
#[unsafe(naked)]
pub unsafe extern "sysv64" fn first(frame:u64)->!{naked_asm!("mov rsp,rdi","jmp {restore}",restore=sym restore)}
#[unsafe(naked)]
unsafe extern "sysv64" fn unexpected()->!{
    naked_asm!("cli","cld","and rsp,-16","call {f}","ud2",f=sym unexpected_rust)
}
extern "sysv64" fn unexpected_rust()->!{fatal("RAR-PANIC:CODE=PLATFORM-VECTOR")}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector0()->!{naked_asm!("push 0","push 0","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector1()->!{naked_asm!("push 0","push 1","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector2()->!{naked_asm!("push 0","push 2","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector3()->!{naked_asm!("push 0","push 3","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector4()->!{naked_asm!("push 0","push 4","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector5()->!{naked_asm!("push 0","push 5","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector6()->!{naked_asm!("push 0","push 6","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector7()->!{naked_asm!("push 0","push 7","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector8()->!{naked_asm!("push 8","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector9()->!{naked_asm!("push 0","push 9","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector10()->!{naked_asm!("push 10","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector11()->!{naked_asm!("push 11","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector12()->!{naked_asm!("push 12","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector13()->!{naked_asm!("push 13","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector14()->!{naked_asm!("push 14","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector15()->!{naked_asm!("push 0","push 15","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector16()->!{naked_asm!("push 0","push 16","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector17()->!{naked_asm!("push 17","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector18()->!{naked_asm!("push 0","push 18","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector19()->!{naked_asm!("push 0","push 19","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector20()->!{naked_asm!("push 0","push 20","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector21()->!{naked_asm!("push 21","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector22()->!{naked_asm!("push 0","push 22","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector23()->!{naked_asm!("push 0","push 23","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector24()->!{naked_asm!("push 0","push 24","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector25()->!{naked_asm!("push 0","push 25","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector26()->!{naked_asm!("push 0","push 26","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector27()->!{naked_asm!("push 0","push 27","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector28()->!{naked_asm!("push 0","push 28","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector29()->!{naked_asm!("push 29","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector30()->!{naked_asm!("push 30","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector31()->!{naked_asm!("push 0","push 31","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector32()->!{naked_asm!("push 0","push 32","jmp {common}",common=sym common)}
#[unsafe(naked)]
unsafe extern "sysv64" fn vector128()->!{naked_asm!("push 0","push 128","jmp {common}",common=sym common)}

/// RSP0 and CR3 are changed only with IF=0 while both roots map identical kernel
/// code/data/stacks. User return frame was validated by the scheduler beforehand.
pub unsafe fn activate(root:u64,stack:u64){
    unsafe{
        ptr::addr_of_mut!((*ptr::addr_of_mut!(TSS)).rsp).cast::<u64>().write_unaligned(stack);
        asm!("mov cr3,{}",in(reg)root,options(nostack));
    }
}
pub unsafe fn install(arena:u64){
    unsafe{asm!("cli",options(nostack));}
    let tss=ptr::addr_of_mut!(TSS);
    unsafe{ptr::addr_of_mut!((*tss).ist).cast::<u64>().write_unaligned(arena+crate::boot::EMERGENCY_TOP);}
    let base=tss as u64;
    let low=103u64|((base&0xffff)<<16)|(((base>>16)&0xff)<<32)|(0x89u64<<40)|(((base>>24)&0xff)<<56);
    let gdt=ptr::addr_of_mut!(GDT).cast::<u64>();
    unsafe{gdt.add(5).write(low);gdt.add(6).write(base>>32);}
    let gdtr=Descriptor{limit:55,base:gdt as u64};
    unsafe{
        asm!("lgdt [{table}]","push 8","lea rax,[rip+2f]","push rax","retfq","2:",
            "mov ax,16","mov ds,ax","mov es,ax","mov ss,ax","xor eax,eax","mov fs,ax","mov gs,ax",
            "mov ax,40","ltr ax",table=in(reg)&gdtr,out("rax")_);
        // User code has no FSGSBASE/XSAVE facility in this fixed SSE2 profile.
        let cr0:u64;let cr4:u64;
        asm!("mov {},cr0",out(reg)cr0,options(nostack));
        asm!("mov cr0,{}",in(reg)((cr0|2)&!12u64),options(nostack));
        asm!("mov {},cr4",out(reg)cr4,options(nostack));
        asm!("mov cr4,{}",in(reg)((cr4|(1<<9)|(1<<10))&!((1<<16)|(1<<18))),options(nostack));
        for msr in [0xc0000100u32,0xc0000101u32]{
            asm!("wrmsr",in("ecx")msr,in("eax")0u32,in("edx")0u32,options(nostack));
        }
    }
    let gates=ptr::addr_of_mut!(IDT).cast::<Gate>();
    for i in 0..256{unsafe{gates.add(i).write(Gate::new(unexpected as *const () as u64,1,false));}}
    unsafe{gates.add(0).write(Gate::new(vector0 as *const () as u64,0,false));}
    unsafe{gates.add(1).write(Gate::new(vector1 as *const () as u64,0,false));}
    unsafe{gates.add(2).write(Gate::new(vector2 as *const () as u64,1,false));}
    unsafe{gates.add(3).write(Gate::new(vector3 as *const () as u64,0,false));}
    unsafe{gates.add(4).write(Gate::new(vector4 as *const () as u64,0,false));}
    unsafe{gates.add(5).write(Gate::new(vector5 as *const () as u64,0,false));}
    unsafe{gates.add(6).write(Gate::new(vector6 as *const () as u64,0,false));}
    unsafe{gates.add(7).write(Gate::new(vector7 as *const () as u64,0,false));}
    unsafe{gates.add(8).write(Gate::new(vector8 as *const () as u64,1,false));}
    unsafe{gates.add(9).write(Gate::new(vector9 as *const () as u64,0,false));}
    unsafe{gates.add(10).write(Gate::new(vector10 as *const () as u64,0,false));}
    unsafe{gates.add(11).write(Gate::new(vector11 as *const () as u64,0,false));}
    unsafe{gates.add(12).write(Gate::new(vector12 as *const () as u64,0,false));}
    unsafe{gates.add(13).write(Gate::new(vector13 as *const () as u64,0,false));}
    unsafe{gates.add(14).write(Gate::new(vector14 as *const () as u64,0,false));}
    unsafe{gates.add(15).write(Gate::new(vector15 as *const () as u64,0,false));}
    unsafe{gates.add(16).write(Gate::new(vector16 as *const () as u64,0,false));}
    unsafe{gates.add(17).write(Gate::new(vector17 as *const () as u64,0,false));}
    unsafe{gates.add(18).write(Gate::new(vector18 as *const () as u64,0,false));}
    unsafe{gates.add(19).write(Gate::new(vector19 as *const () as u64,0,false));}
    unsafe{gates.add(20).write(Gate::new(vector20 as *const () as u64,0,false));}
    unsafe{gates.add(21).write(Gate::new(vector21 as *const () as u64,0,false));}
    unsafe{gates.add(22).write(Gate::new(vector22 as *const () as u64,0,false));}
    unsafe{gates.add(23).write(Gate::new(vector23 as *const () as u64,0,false));}
    unsafe{gates.add(24).write(Gate::new(vector24 as *const () as u64,0,false));}
    unsafe{gates.add(25).write(Gate::new(vector25 as *const () as u64,0,false));}
    unsafe{gates.add(26).write(Gate::new(vector26 as *const () as u64,0,false));}
    unsafe{gates.add(27).write(Gate::new(vector27 as *const () as u64,0,false));}
    unsafe{gates.add(28).write(Gate::new(vector28 as *const () as u64,0,false));}
    unsafe{gates.add(29).write(Gate::new(vector29 as *const () as u64,0,false));}
    unsafe{gates.add(30).write(Gate::new(vector30 as *const () as u64,0,false));}
    unsafe{gates.add(31).write(Gate::new(vector31 as *const () as u64,0,false));}
    unsafe{gates.add(32).write(Gate::new(vector32 as *const () as u64,0,false));}
    unsafe{gates.add(128).write(Gate::new(vector128 as *const () as u64,0,true));}
    let idtr=Descriptor{limit:4095,base:gates as u64};
    unsafe{asm!("lidt [{}]",in(reg)&idtr,options(readonly,nostack));out(0x21,0xfe);out(0xa1,0xff);}
}
