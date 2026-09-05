//! Narrow UEFI GOP discovery and PS/2 setup for the fixed cloud platform.
use core::{mem,ptr};
use super::BootHardware;
use crate::{fatal,input,out};
#[repr(C)]struct Guid{a:u32,b:u16,c:u16,d:[u8;8]}
const GOP:Guid=Guid{a:0x9042a9de,b:0x23dc,c:0x4a38,d:[0x96,0xfb,0x7a,0xde,0xd0,0x80,0x51,0x6a]};
#[repr(C)]#[derive(Clone,Copy)]
struct Info{version:u32,width:u32,height:u32,format:u32,masks:[u32;4],pitch:u32}
#[repr(C)]struct Mode{max:u32,current:u32,info:*const Info,size:usize,base:u64,bytes:usize}
#[repr(C)]struct Gop{query:usize,set:usize,blt:usize,mode:*const Mode}
fn pointer(value:usize)->bool{value!=0 && value<0x1_0000_0000}
/// Firmware tables/protocol buffers originate from the hash-bound OVMF. Their
/// bounded structural fields are checked before calls/copies; no guest parser
/// or application can supply these addresses.
pub unsafe fn configure(functions:&[usize;44],image_base:u64,image_size:u64)->BootHardware{
    type Locate=unsafe extern "efiapi" fn(*const Guid,usize,*mut *mut Gop)->usize;
    type Free=unsafe extern "efiapi" fn(*mut Info)->usize;
    type Query=unsafe extern "efiapi" fn(*mut Gop,u32,*mut usize,*mut *mut Info)->usize;
    type Set=unsafe extern "efiapi" fn(*mut Gop,u32)->usize;
    if !pointer(functions[37])||!pointer(functions[6]){fatal("RAR-PANIC:CODE=GOP-SERVICE");}
    let locate:Locate=unsafe{mem::transmute(functions[37])};let free:Free=unsafe{mem::transmute(functions[6])};
    let mut gop=ptr::null_mut();
    if unsafe{locate(&GOP,0,&mut gop)}!=0 || !pointer(gop as usize) || gop as usize%8!=0 {
        fatal("RAR-PANIC:CODE=GOP");
    }
    let (query,set,mode)=unsafe{((*gop).query,(*gop).set,(*gop).mode)};
    if !pointer(query)||!pointer(set)||!pointer(mode as usize)||mode as usize%8!=0{fatal("RAR-PANIC:CODE=GOP-POINTER");}
    let max=unsafe{(*mode).max};
    if max==0||max>256{fatal("RAR-PANIC:CODE=GOP-MODES");}
    let query:Query=unsafe{mem::transmute(query)};let set:Set=unsafe{mem::transmute(set)};
    let mut chosen=None;
    for index in 0..max{
        let mut size=0;let mut info=ptr::null_mut();
        if unsafe{query(gop,index,&mut size,&mut info)}!=0{continue;}
        if !pointer(info as usize)||info as usize%4!=0||!(36..=256).contains(&size){fatal("RAR-PANIC:CODE=GOP-INFO");}
        let value=unsafe{info.read()};
        if unsafe{free(info)}!=0{fatal("RAR-PANIC:CODE=GOP-FREE");}
        if value.version==0&&value.width==640&&value.height==480&&value.format<=1{chosen=Some(index);break;}
    }
    let chosen=chosen.unwrap_or_else(||fatal("RAR-PANIC:CODE=GOP-640"));
    if unsafe{set(gop,chosen)}!=0{fatal("RAR-PANIC:CODE=GOP-SET");}
    let mode=unsafe{(*gop).mode};
    if !pointer(mode as usize)||mode as usize%8!=0{fatal("RAR-PANIC:CODE=GOP-MODE");}
    let info=unsafe{(*mode).info};
    if !pointer(info as usize)||info as usize%4!=0||unsafe{(*mode).size}<36{fatal("RAR-PANIC:CODE=GOP-ACTIVE");}
    let value=unsafe{info.read()};
    let (framebuffer,bytes)=unsafe{((*mode).base,(*mode).bytes)};
    if value.version!=0{fatal("RAR-PANIC:CODE=GOP-VERSION");}
    let rounded=super::model::framebuffer_span(value.width,value.height,value.pitch,value.format,framebuffer,bytes as u64)
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=GOP-RANGE"));
    BootHardware{image_base,image_size,framebuffer,framebuffer_bytes:rounded,pitch:value.pitch as u64,format:value.format as u64}
}
unsafe fn wait_write(){
    for _ in 0..100_000{if unsafe{input(0x64)}&2==0{return;}core::hint::spin_loop();}
    fatal("RAR-PANIC:CODE=PS2-BUSY");
}
unsafe fn command(value:u8){unsafe{wait_write();out(0x64,value);}}
unsafe fn data(value:u8){unsafe{wait_write();out(0x60,value);}}
unsafe fn read()->u8{
    for _ in 0..100_000{if unsafe{input(0x64)}&1!=0{return unsafe{input(0x60)};}core::hint::spin_loop();}
    fatal("RAR-PANIC:CODE=PS2-EMPTY");
}
/// Fixed setup only: no reset command, arbitrary port write or hardware passthrough.
/// IRQs remain disabled for PS/2; the isolated ring3 service polls the read broker.
pub unsafe fn keyboard(){
    unsafe{command(0xad);command(0xa7);}
    for _ in 0..32{if unsafe{input(0x64)}&1==0{break;}let _=unsafe{input(0x60)};}
    unsafe{command(0x20);}
    let config=unsafe{read()};
    unsafe{command(0x60);data((config|0x40)&!0x13);command(0xae);data(0xf4);}
    if unsafe{read()}!=0xfa{fatal("RAR-PANIC:CODE=PS2-ACK");}
}
