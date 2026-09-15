//! x86-64 native SDK adapter. Object-only compilation is not runtime proof.
//! No ambient device/file/network API and no automatic operation retry.
#![deny(unsafe_op_in_unsafe_fn)]
use super::{wire::{Boot,Message},transport::{Received,ENVELOPE_BYTES}};
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Failure{Invalid,Kernel(i64)}
/// Safety: only called by this SDK inside a RAR-owned CPL3 task. The kernel
/// copies validated spans before return and preserves all registers except RAX.
/// Do not mark this asm pure/nomem: SEND/RECEIVE copy caller memory. The int80
/// CPU context preserves SIMD and compiler-visible registers. No privileged I/O.
fn call(n:u64,a:u64,b:u64,c:u64,d:u64)->i64{
    let result:u64;
    unsafe{core::arch::asm!("int 0x80",inlateout("rax")n=>result,
        in("rdi")a,in("rsi")b,in("rdx")c,in("r10")d);}
    result as i64
}
/// # Safety
/// Entry may call once only after the RAR kernel mapped the initialized 256-byte
/// bootstrap read-only at0x700000. Never call from a host process or firmware.
pub unsafe fn bootstrap()->Result<Boot,Failure>{
    let mut bytes=[0;256];
    for(i,b)in bytes.iter_mut().enumerate(){
        // SAFETY: fixed initialized, readable kernel mapping promised above.
        *b=unsafe{(0x700000usize as *const u8).add(i).read_volatile()};
    }
    Boot::decode(&bytes).map_err(|_|Failure::Invalid)
}
pub fn yield_now(){let _=call(0,0,0,0,0);}
pub fn ticks()->Result<u64,Failure>{
    let n=call(6,0,0,0,0);if n<0{Err(Failure::Kernel(n))}else{Ok(n as u64)}
}
/// Exactly one kernel SEND. Full queues are returned to the caller; successful
/// or uncertain requests are never silently resent.
pub fn send(boot:&Boot,peer:usize,message:&Message)->Result<(),Failure>{
    if peer>=4||boot.peer_incarnations[peer]==0{return Err(Failure::Invalid);}
    let bytes=message.encode();
    match call(1,boot.caps[peer+1],bytes.as_ptr()as u64,128,0){
        0=>Ok(()),n=>Err(Failure::Kernel(n)),
    }
}
/// Nonblocking receive. Short/malformed frames are dequeued and returned as
/// Invalid, not interpreted as authority. Caller controls bounded wait policy.
pub fn receive(boot:&Boot)->Result<Option<Received>,Failure>{
    let mut bytes=[0;ENVELOPE_BYTES];
    let n=call(2,boot.caps[0],bytes.as_mut_ptr()as u64,ENVELOPE_BYTES as u64,0);
    if n==-5{return Ok(None);}
    if n<0{return Err(Failure::Kernel(n));}
    Received::decode(&bytes,n).map(Some).map_err(|_|Failure::Invalid)
}
pub fn exit()->!{let _=call(5,0,0,0,0);loop{yield_now();}}
