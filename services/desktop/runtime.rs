//! Ring3 device/service loops; app policy has no hardware capability.
#[path="render.rs"] mod render;
use crate::{abi::*,services,receive,deliver,syscall,check,fail,report,yield_now};
pub fn storage(boot:&Boot)->! {
    let mut store=services::DesktopStore::new();
    loop {
        let e=receive(boot.caps[SELF_RECV]);
        if e.generation!=1||!matches!(e.sender,4|6){continue;}
        let reply=store.process(e.sender,e.generation,&e.bytes);
        let slot=if e.sender==4{FILES}else{TERMINAL};
        // One attempt. A dead or backpressured app never blocks storage.
        match syscall(SEND,boot.caps[slot],reply.as_ptr() as u64,128,0){0|-3|-4=>{},_=>fail()}
    }
}
pub fn keyboard(boot:&Boot)->! {
    let mut decoder=services::Keyboard::new();report(1);
    loop {
        let status=syscall(PORT_READ,boot.caps[INPUT],0x64,0,0);check(status>=0);
        if status&1!=0 {
            let value=syscall(PORT_READ,boot.caps[INPUT],0x60,0,0);check(value>=0);
            if status&0xc0!=0{decoder.reset();}
            else if status&0x20==0 {
                if let Some(key)=decoder.feed(value as u8){
                    if let Some(m)=services::apps::key_wire(key){deliver(boot.caps[SHELL],&m);}
                }
            }
        }
        yield_now();
    }
}
pub fn compositor(boot:&Boot)->! {
    let mut state=services::Compositor::new();render::draw(boot,&state);report(2);
    loop {
        let e=receive(boot.caps[SELF_RECV]);
        if state.apply(e.sender,e.generation,&e.bytes)==Ok(true){render::draw(boot,&state);}
    }
}
