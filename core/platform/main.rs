#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
mod abi;
mod context;
#[path="../../services/platform/runtime.rs"]
mod drivers;
#[path="../../services/platform/model.rs"]
mod services;
use core::{arch::asm,panic::PanicInfo};
use abi::*;
fn syscall(n:u64,a:u64,b:u64,c:u64,d:u64)->i64 {
    let result:u64;
    // The private int80 ABI preserves all registers except its RAX result.
    unsafe {asm!("int 0x80",inlateout("rax")n=>result,in("rdi")a,in("rsi")b,in("rdx")c,in("r10")d);}
    result as i64
}
fn yield_now(){let _=syscall(YIELD,0,0,0,0);}
fn report(code:u64){if syscall(REPORT,code,0,0,0)!=0 {fail();}}
fn exit()->!{let _=syscall(EXIT,0,0,0,0);loop{yield_now();}}
fn fail()->!{let _=syscall(REPORT,255,0,0,0);exit()}
fn check(value:bool){if !value{fail();}}
#[panic_handler]
fn panic(_:&PanicInfo)->!{fail()}
fn send(handle:u64,bytes:&[u8;128]){
    loop{match syscall(SEND,handle,bytes.as_ptr() as u64,128,0){0=>return,-4=>yield_now(),_=>fail()}}
}
fn receive(handle:u64)->Envelope{
    let mut envelope=Envelope::EMPTY;
    loop{
        let status=syscall(RECEIVE,handle,(&mut envelope as *mut Envelope) as u64,144,1);
        if status==128{return envelope;}
        if status != -5{fail();}
    }
}
fn call(boot:&Boot,op:u8,name:&[u8],data:&[u8])->[u8;128]{
    let request=services::request(op,name,data).unwrap_or_else(||fail());
    send(boot.caps[STORAGE],&request);
    let message=receive(boot.caps[SELF_RECV]);
    check(message.sender==1 && message.generation==1 && message.length==128);
    message.bytes
}
fn client(boot:&Boot)->!{
    let data=[0x41;128];
    check(syscall(SEND,boot.caps[SELF_RECV],data.as_ptr() as u64,128,0)==-2);
    check(syscall(SEND,boot.caps[STALE],data.as_ptr() as u64,128,0)==-3);
    check(syscall(SEND,u64::MAX,data.as_ptr() as u64,128,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],boot.kernel_probe,128,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],u64::MAX-1,128,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],0x800000,128,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],0,128,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],0x60ffff,2,0)<0);
    check(syscall(SEND,boot.caps[SELF_SEND],data.as_ptr() as u64,129,0)<0);
    check(syscall(PORT_READ,boot.caps[SELF_SEND],0x60,0,0)==-2);
    report(1);
    for _ in 0..4{check(syscall(SEND,boot.caps[SELF_SEND],data.as_ptr() as u64,128,0)==0);}
    check(syscall(SEND,boot.caps[SELF_SEND],data.as_ptr() as u64,128,0)==-4);
    check(syscall(RECEIVE,boot.caps[SELF_RECV],BOOT_ADDRESS as u64,144,0)<0);
    for _ in 0..4{let message=receive(boot.caps[SELF_RECV]);check(message.sender==0&&message.generation==1&&message.bytes==data);}
    let mut empty=Envelope::EMPTY;
    check(syscall(RECEIVE,boot.caps[SELF_RECV],(&mut empty as *mut Envelope) as u64,144,0)==-5);
    report(2);
    check(call(boot,services::CREATE,b"alpha",b"")[0]==services::OK);
    check(call(boot,services::WRITE,b"alpha",b"RAR Platform")[0]==services::OK);
    let got=call(boot,services::READ,b"alpha",b"");
    check(got[0]==services::OK&&got[2]==12&&&got[16..28]==b"RAR Platform");
    let list=call(boot,services::LIST,b"",b"");
    check(list[0]==services::OK&&list[1]==1&&&list[5..10]==b"alpha");
    for name in [b"b",b"c",b"d"]{check(call(boot,services::CREATE,name,b"")[0]==services::OK);}
    check(call(boot,services::CREATE,b"e",b"")[0]==services::QUOTA);
    check(call(boot,services::WRITE,b"alpha",&[1;64])[0]==services::OK);
    check(call(boot,services::WRITE,b"b",&[2;64])[0]==services::OK);
    check(call(boot,services::WRITE,b"c",&[3])[0]==services::QUOTA);
    check(call(boot,services::READ,b"alpha",b"")[16]==1);
    let mut dead=false;
    for _ in 0..128 {
        let result=syscall(SEND,boot.caps[DEAD_PEER],data.as_ptr() as u64,128,0);
        if result==-3 {dead=true;break;}
        check(result==0||result==-4);yield_now();
    }
    check(dead);
    // Wake the context fixture only after its actual blocking receive.
    loop{
        let result=syscall(REPORT,11,0,0,0);
        if result==0{break;}
        check(result==-5);check(call(boot,services::READ,b"alpha",b"")[16]==1);yield_now();
    }
    send(boot.caps[AUXILIARY],&data);
    loop{
        check(call(boot,services::READ,b"alpha",b"")[16]==1);
        check(call(boot,services::LIST,b"",b"")[1]==4);
        let result=syscall(REPORT,3,0,0,0);
        if result==0{break;}
        check(result==-5);yield_now();
    }
    // Remain live after the fault fixtures and non-yielding peer are preempted.
    loop{yield_now();}
}
fn second_client(boot:&Boot)->!{
    check(call(boot,services::READ,b"alpha",b"")[0]==services::NOT_FOUND);
    check(call(boot,services::LIST,b"",b"")[1]==0);
    check(call(boot,services::CREATE,b"alpha",b"")[0]==services::OK);
    check(call(boot,services::WRITE,b"alpha",b"private")[0]==services::OK);
    let got=call(boot,services::READ,b"alpha",b"");check(&got[16..23]==b"private");
    report(8);loop{yield_now();}
}
#[unsafe(no_mangle)]
pub extern "efiapi" fn efi_main()->!{
    // Fixed read-only kernel bootstrap page; no firmware pointer is accepted.
    let boot=unsafe{(BOOT_ADDRESS as *const Boot).read()};
    check(boot.magic==MAGIC&&boot.generation==1);
    match boot.role{
        0=>client(&boot),1=>drivers::storage(&boot),2=>drivers::keyboard(&boot),3=>drivers::framebuffer(&boot),
        4=>unsafe{context::exercise()},5=>loop{unsafe{asm!("pause",options(nomem,nostack));}},
        6=>{let _=unsafe{(boot.kernel_probe as *const u64).read_volatile()};fail()},
        7=>unsafe{let code=0x600000 as *mut u8;code.write_volatile(0xc3);asm!("jmp rax",in("rax")code,options(noreturn));},
        8=>{unsafe{(0x5ff000 as *mut u8).write_volatile(1);}fail()},
        9=>unsafe{asm!("out 0x80, al",in("al")0u8);fail()},
        10=>second_client(&boot),
        11=>{unsafe{(boot.entry as *mut u8).write_volatile(0);}fail()},
        12=>{let _=unsafe{(boot.peer_probe as *const u64).read_volatile()};fail()},
        13=>unsafe{let value:u64;asm!("mov {},cr3",out(reg)value,options(nostack));let _=value;fail()},
        14=>{
            let request=services::request(services::READ,b"missing",b"").unwrap_or_else(||fail());
            send(boot.caps[STORAGE],&request);
            // Invalid return state is revoked in this trap, before storage can
            // send its reply. The request remains a valid sender-stamped message.
            unsafe{asm!("xor eax,eax","xor esp,esp","int 0x80","ud2",options(noreturn));}
        }
        15=>{
            let request=services::request(services::READ,b"missing",b"").unwrap_or_else(||fail());
            for _ in 0..8{send(boot.caps[STORAGE],&request);}
            loop{let result=syscall(REPORT,10,0,0,0);if result==0{break;}check(result==-5);yield_now();}
            send(boot.caps[STORAGE],&request);exit()
        }
        _=>fail(),
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
