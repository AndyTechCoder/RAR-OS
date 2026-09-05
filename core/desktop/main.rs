#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod abi;
mod memory;
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
        if (1..=128).contains(&status) {
            // SEND accepts bounded short messages. Discard noncanonical Desktop
            // lengths after dequeue; malformed peers must not kill a service.
            if status==128&&envelope.length==128{return envelope;}
            continue;
        }
        if status != -5 {fail();}
    }
}
fn poll(handle:u64)->Option<Envelope> {
    let mut envelope=Envelope::EMPTY;
    let status=syscall(RECEIVE,handle,(&mut envelope as *mut Envelope) as u64,144,0);
    if status==128&&envelope.length==128{return Some(envelope);}
    if status==-5||(1..=128).contains(&status){return None;}
    fail()
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
    check(valid_boot(&boot));
    match boot.role {
        0=>apps::shell(&boot),1=>drivers::storage(&boot),2=>drivers::keyboard(&boot),
        3=>drivers::compositor(&boot),4=>apps::files(&boot),5=>apps::settings(&boot),
        6=>apps::terminal(&boot),7=>loop{yield_now();},_=>fail(),
    }
}
