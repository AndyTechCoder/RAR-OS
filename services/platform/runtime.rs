//! Actual ring3 storage/input/framebuffer service policy. No nucleus linkage.
use crate::{abi::*,services,receive,check,fail,report,yield_now,syscall};
pub fn storage(boot:&Boot)->!{
    let mut store=services::Store::new();
    loop{
        let message=receive(boot.caps[SELF_RECV]);
        check(message.length==128&&message.generation==1);
        let reply=store.process(services::Owner{task:message.sender as u8,generation:message.generation},&message.bytes);
        let handle=match message.sender{0=>boot.caps[CLIENT],10=>boot.caps[SECOND_CLIENT],
            14=>boot.caps[FAULT_REPLY],15=>boot.caps[AUXILIARY],_=>fail()};
        // A reply is attempted once. Dead/backpressured clients cannot block or
        // terminate this shared service; their undeliverable reply is discarded.
        match syscall(SEND,handle,reply.as_ptr() as u64,128,0){0|-3|-4=>{},_=>fail()}
    }
}
pub fn keyboard(boot:&Boot)->!{
    let mut decoder=services::Keyboard::new();
    report(6);
    loop{
        let status=syscall(PORT_READ,boot.caps[INPUT],0x64,0,0);
        check(status>=0);
        if status&1!=0{
            let value=syscall(PORT_READ,boot.caps[INPUT],0x60,0,0);check(value>=0);
            // Auxiliary-device bytes are not keyboard proof.
            if status&0x20==0 && decoder.feed(value as u8){report(7);loop{yield_now();}}
        }
        yield_now();
    }
}
pub fn framebuffer(boot:&Boot)->!{
    check(boot.caps[FRAMEBUFFER]!=0&&boot.framebuffer==0x800000&&boot.width==640&&boot.height==480&&boot.pitch>=640&&boot.format<=1);
    for y in 0..480usize{for x in 0..640usize{
        let (r,g,b)=if x==0||x==639||y==0||y==479{(0u32,0u32,0u32)}
            else if y<240{if x<320{(255,0,0)}else{(0,255,0)}}
            else if x<320{(0,0,255)}else{(255,255,255)};
        let pixel=if boot.format==0{r|(g<<8)|(b<<16)}else{b|(g<<8)|(r<<16)};
        // The nucleus maps only this role's validated framebuffer span RW+NX.
        unsafe{((boot.framebuffer as usize+(y*boot.pitch as usize+x)*4) as *mut u32).write_volatile(pixel);}
    }}
    report(5);loop{yield_now();}
}
