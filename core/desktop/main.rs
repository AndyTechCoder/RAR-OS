#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod abi;
#[path="../../services/desktop/model.rs"] mod services;
#[path="../../services/desktop/runtime.rs"] mod drivers;
#[path="../../apps/desktop/runtime.rs"] mod apps;
use core::{arch::asm,panic::PanicInfo};
use abi::*;
fn syscall(n:u64,a:u64,b:u64,c:u64,d:u64)->i64 {
    let result:u64;
    // Kernel-owned int80 frame preserves every register except RAX.
    unsafe{asm!("int 0x80",inlateout("rax")n=>result,in("rdi")a,in("rsi")b,in("rdx")c,in("r10")d);}
    result as i64
}
fn yield_now(){let _=syscall(YIELD,0,0,0,0);}
fn fail()->!{let _=syscall(REPORT,255,0,0,0);let _=syscall(EXIT,0,0,0,0);loop{yield_now();}}
fn check(ok:bool){if !ok{fail();}}
fn report(code:u64){check(syscall(REPORT,code,0,0,0)==0);}
#[panic_handler] fn panic(_:&PanicInfo)->!{fail()}
/// Bounded backpressure; a revoked destination is returned to the caller.
fn send(handle:u64,bytes:&[u8;128])->Result<(),i64>{
    for _ in 0..256 {
        match syscall(SEND,handle,bytes.as_ptr() as u64,128,0) {
            0=>return Ok(()),-4=>yield_now(),error=>return Err(error),
        }
    }
    Err(-4)
}
fn deliver(handle:u64,bytes:&[u8;128]){if send(handle,bytes).is_err(){fail();}}
fn receive(handle:u64)->Envelope {
    let mut envelope=Envelope::EMPTY;
    loop {
        let status=syscall(RECEIVE,handle,(&mut envelope as *mut Envelope) as u64,144,1);
        if status==128 {check(envelope.length==128);return envelope;}
        if status != -5 {fail();}
    }
}
fn publish(boot:&Boot,version:&mut u32,view:&services::apps::View) {
    *version=version.checked_add(1).unwrap_or_else(||fail());
    deliver(boot.caps[COMPOSITOR],&services::begin(*version));
    for i in 0..6 {deliver(boot.caps[COMPOSITOR],&services::line(*version,i,&view.lines[i]).unwrap_or_else(||fail()));}
    deliver(boot.caps[COMPOSITOR],&services::commit(*version));
}
#[unsafe(no_mangle)] pub extern "efiapi" fn efi_main()->! {
    // Fixed kernel-owned read-only bootstrap mapping, no firmware pointer.
    let boot=unsafe{(BOOT_ADDRESS as *const Boot).read()};
    check(boot.magic==MAGIC&&boot.generation==1);
    match boot.role {
        0=>apps::shell(&boot),1=>drivers::storage(&boot),2=>drivers::keyboard(&boot),
        3=>drivers::compositor(&boot),4=>apps::files(&boot),5=>apps::settings(&boot),
        6=>apps::terminal(&boot),7=>loop{yield_now();},_=>fail(),
    }
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcpy(d:*mut u8,s:*const u8,n:usize)->*mut u8{for i in 0..n{unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}d}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(d:*mut u8,v:i32,n:usize)->*mut u8{for i in 0..n{unsafe{d.add(i).write_volatile(v as u8);}}d}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memmove(d:*mut u8,s:*const u8,n:usize)->*mut u8{
    if (d as usize)<(s as usize){for i in 0..n{unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}}
    else{for i in (0..n).rev(){unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}}d
}
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memcmp(a:*const u8,b:*const u8,n:usize)->i32{for i in 0..n{let x=unsafe{a.add(i).read_volatile()};let y=unsafe{b.add(i).read_volatile()};if x!=y{return x as i32-y as i32;}}0}
