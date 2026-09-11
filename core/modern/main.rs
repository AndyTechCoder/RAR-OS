#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod abi;
#[path="../../apps/modern/settings.rs"] mod settings;
#[path="../../apps/modern/model.rs"] mod file_ui;
#[path="../desktop/memory.rs"] mod memory;
#[path="../../services/modern/gui.rs"] mod services;
#[path="../../services/modern/runtime.rs"] mod drivers;
#[path="../../apps/modern/runtime.rs"] mod apps;
#[path="../../services/modern/pio.rs"] mod pio;
#[path="../../services/modern/vault.rs"] mod vault;
#[path="../../services/modern/store.rs"] mod store;
#[path="../../services/modern/transport.rs"] mod transport;
#[path="../../services/modern/session.rs"] mod session;
#[path="../../services/modern/desktop_wire.rs"] mod desktop_wire;
#[path="../crypto/sha256.rs"] mod sha256;
#[path="../crypto/chacha20poly1305.rs"] mod chacha20poly1305;
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
        let status=syscall(RECEIVE,handle,(&mut envelope as *mut Envelope) as u64,ENVELOPE_BYTES,1);
        if (1..=128).contains(&status) {
            // SEND accepts bounded short messages. Discard noncanonical Modern
            // lengths after dequeue; malformed peers must not kill a service.
            if status==128&&envelope.length==128{return envelope;}
            continue;
        }
        if status != -5 {fail();}
    }
}
fn poll_checked(handle:u64)->Result<Option<Envelope>,()> {
    let mut envelope=Envelope::EMPTY;
    let status=syscall(RECEIVE,handle,(&mut envelope as *mut Envelope) as u64,ENVELOPE_BYTES,0);
    if (1..=128).contains(&status)&&envelope.length==status as u64{return Ok(Some(envelope));}
    if status==-5{return Ok(None);}
    Err(())
}
/// The complete full-width identity comes from the kernel, never an app frame.
fn settings_binding(boot:&Boot)->u64{
    let handle=match boot.role{0=>boot.caps[SETTINGS],3=>boot.caps[FRAMEBUFFER],_=>fail()};
    let mut bytes=[0u8;8];
    check(syscall(SETTINGS_BINDING,handle,bytes.as_mut_ptr()as u64,8,0)==0);
    u64::from_le_bytes(bytes)
}
fn publish(boot:&Boot,version:&mut u32,view:&services::apps::View) {
    *version=version.checked_add(1).unwrap_or_else(||fail());
    deliver(boot.caps[COMPOSITOR],&services::begin(*version));
    for i in 0..6 {deliver(boot.caps[COMPOSITOR],&services::line(*version,i,&view.lines[i]).unwrap_or_else(||fail()));}
    deliver(boot.caps[COMPOSITOR],&services::commit(*version));
}
fn boot_snapshot()->Boot {
    // SAFETY: kernel-owned aligned initialized Boot mapping at fixed address,
    // readable for this process's entire lifetime, never user-writable. The
    // kernel may republish it only while this process is not running. No Rust
    // reference crosses the trap; volatile reads obtain the post-cutover bytes.
    unsafe{core::ptr::read_volatile(BOOT_ADDRESS as *const Boot)}
}
#[unsafe(no_mangle)] pub extern "efiapi" fn efi_main()->! {
    let initial=boot_snapshot();
    check(valid_boot(&initial));
    let boot=if initial.phase==TRIAL {
        // Trial has only HEALTH, so do not call normal Settings IPC yet.
        check(settings::health());
        check(syscall(TRIAL_READY,initial.caps[HEALTH],initial.health_token,0,0)==0);
        // A successful health report blocks in the kernel. Only cutover may
        // resume this context, after publishing its new read-only bootstrap.
        let active=boot_snapshot();
        check(valid_trial_activation(&initial,&active));
        active
    }else{initial};
    match boot.role {
        0=>apps::shell(&boot),1=>drivers::storage(&boot),2=>drivers::keyboard(&boot),
        3=>drivers::compositor(&boot),4=>apps::files(&boot),5=>apps::settings(&boot),
        6=>apps::terminal(&boot),8=>drivers::manager(&boot),9=>drivers::system(&boot),
        15=>loop{yield_now();},_=>fail(),
    }
}
