//! Native standalone Notes entry; cloud object check never executes this file.
#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
#[path="../../../sdk/alpha/rust/lib.rs"] mod sdk;
#[path="../../../core/desktop/memory.rs"] mod memory;
mod model;
use sdk::{native,wire::Boot,protocol};
#[panic_handler]fn panic(_: &core::panic::PanicInfo)->!{native::exit()}
fn send(boot:&Boot,peer:usize,m:&sdk::wire::Message)->Result<(),()>{
    let deadline=native::ticks().map_err(|_|())?.checked_add(100).ok_or(())?;
    for _ in 0..256{
        match native::send(boot,peer,m){
            Ok(())=>return Ok(()),
            Err(native::Failure::Kernel(-4))=>{
                if native::ticks().map_err(|_|())?>=deadline{return Err(());}native::yield_now();
            },
            Err(_)=>return Err(()),
        }
    }Err(())
}
fn paint(boot:&Boot,notes:&model::Notes,version:&mut u32)->Result<(),()>{
    *version=version.checked_add(1).ok_or(())?;
    send(boot,0,&protocol::begin(*version,6).map_err(|_|())?)?;
    for row in 0..6{send(boot,0,&protocol::line(*version,row as u8,notes.line(row)).map_err(|_|())?)?;}
    send(boot,0,&protocol::commit(*version).map_err(|_|())?)
}
#[unsafe(no_mangle)]pub extern "efiapi" fn efi_main()->!{
    // SAFETY: this entry is only scheduled by the RAR private app-root builder;
    // its validated immutable bootstrap page is mapped before first instruction.
    let boot=unsafe{native::bootstrap()}.unwrap_or_else(|_|native::exit());
    let mut notes=model::Notes::new(boot).unwrap_or_else(|_|native::exit());
    let mut version=0;
    let now=native::ticks().unwrap_or_else(|_|native::exit());
    let request=notes.request(false,now).unwrap_or_else(|_|native::exit());
    if send(&boot,1,&request).is_err(){notes.send_failed();}
    if paint(&boot,&notes,&mut version).is_err(){native::exit();}
    loop{
        let now=native::ticks().unwrap_or_else(|_|native::exit());
        let mut changed=notes.tick(now);
        match native::receive(&boot){
            Ok(Some(received))=>{
                if let Ok(request)=notes.receive(received,now){
                    changed=true;
                    if let Some(request)=request{
                        if send(&boot,1,&request).is_err(){notes.send_failed();}
                    }
                }
            },
            Ok(None)|Err(native::Failure::Invalid)=>{},
            Err(_)=>native::exit(),
        }
        if changed&&paint(&boot,&notes,&mut version).is_err(){native::exit();}
        native::yield_now();
    }
}
