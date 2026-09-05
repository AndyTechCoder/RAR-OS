//! Experimental Desktop runtime: mechanisms in ring0, service policy in ring3.
mod model;
#[path="../platform/arch.rs"] mod arch;
#[path="../platform/display.rs"] pub(crate) mod display;
#[path="../../core/desktop/abi.rs"]
mod abi;
use core::{mem,ptr};
use crate::{boot,fatal,record,paging::Tables,model::Mapping};
use model::{Caps,Queue,Message,Object,State,UserRange,Error,TASKS};
#[repr(C)]
#[derive(Clone,Copy)]
pub struct BootHardware{
    pub image_base:u64,pub image_size:u64,pub framebuffer:u64,
    pub framebuffer_bytes:u64,pub pitch:u64,pub format:u64,
}
impl BootHardware{pub const EMPTY:Self=Self{image_base:0,image_size:0,framebuffer:0,framebuffer_bytes:0,pitch:0,format:0};}
const PRIVATE_BASE:u64=0x400000;
const STRIDE:u64=0x200000;
const IMAGE:u64=0x100000;
const USER_STACK:u64=0x120000;
const KERNEL_BOTTOM:u64=0x141000;
const KERNEL_TOP:u64=0x151000;
const BOOT:u64=0x160000;
const STACK_VA:u64=0x600000;
const STACK_END:u64=0x610000;
const EMPTY_RANGE:UserRange=UserRange{start:0,end:0,writable:false,executable:false};
static SERVICE:&[u8]=include_bytes!("/tmp/desktop-service.efi");
#[derive(Clone,Copy)]
struct Process{
    state:State,generation:u32,root:u64,kernel_bottom:u64,kernel_top:u64,frame:u64,
    caps:Caps,queue:Queue,ranges:[UserRange;24],range_count:usize,preemptions:u64,entry:u64,
}
impl Process{
    const EMPTY:Self=Self{state:State::Dead,generation:1,root:0,kernel_bottom:0,kernel_top:0,frame:0,
        caps:Caps::new(),queue:Queue::new(),ranges:[EMPTY_RANGE;24],range_count:0,preemptions:0,entry:0};
    fn range(&mut self,start:u64,end:u64,writable:bool,executable:bool){
        if self.range_count>=self.ranges.len()||start>=end||writable&&executable{fatal("RAR-PANIC:CODE=USER-RANGE");}
        self.ranges[self.range_count]=UserRange{start,end,writable,executable};self.range_count+=1;
    }
    fn buffer(&self,pointer:u64,length:usize,write:bool)->Result<(),Error>{
        model::user_buffer(&self.ranges[..self.range_count],pointer,length,write)
    }
}
struct Runtime{
    processes:[Process;TASKS],current:usize,arena:u64,proofs:u8,ready:bool,
}
static mut RUNTIME:Runtime=Runtime{processes:[Process::EMPTY;TASKS],current:0,arena:0,proofs:0,ready:false};
fn private_region(arena:u64,index:usize)->u64{arena+PRIVATE_BASE+index as u64*STRIDE}
fn omit(arena:u64,address:u64)->bool{
    let offset=address-arena;
    if [boot::STACK_GUARD,boot::STACK_TOP,0x160000,boot::EMERGENCY_TOP].contains(&offset){return true;}
    for index in 0..TASKS{
        let base=PRIVATE_BASE+index as u64*STRIDE;
        if (base+IMAGE..base+USER_STACK+0x10000).contains(&offset) ||
            (base+BOOT..base+BOOT+4096).contains(&offset) ||
            offset==base+KERNEL_BOTTOM-4096 || offset==base+KERNEL_TOP{return true;}
    }
    false
}
fn mapping(v:u64,p:u64,pages:u64,w:bool,x:bool)->Mapping{
    Mapping{virtual_start:v,physical_start:p,pages,writable:w,executable:x}
}
unsafe fn add(tables:&mut Tables,process:&mut Process,v:u64,p:u64,pages:u64,w:bool,x:bool,device:bool){
    unsafe{tables.map_user(mapping(v,p,pages,w,x),p,p+pages*4096,device)}
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=USER-MAP"));
    process.range(v,v+pages*4096,w,x);
}
fn grant(process:&mut Process,slot:usize,object:Object,rights:u8)->u64{
    process.caps.grant(slot,object,rights).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=CAP-GRANT"))
}
fn endpoint(task:usize)->Object{Object::Endpoint{task:task as u8,generation:1}}
/// All process destinations are exclusively owned zeroed arena regions, not user
/// data. Copying finishes before roots omit kernel aliases and expose RX pages.
pub unsafe fn start(info:&boot::BootInfo)->!{
    if mem::size_of::<arch::Trap>()!=208||mem::size_of::<abi::Envelope>()!=144||
        info.arena<0x2000000||info.platform.image_base<0x2000000{
        fatal("RAR-PANIC:CODE=PLATFORM-LAYOUT");
    }
    let layout=model::pe::parse(SERVICE).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=SERVICE-PE"));
    let runtime=unsafe{&mut *ptr::addr_of_mut!(RUNTIME)};
    runtime.arena=info.arena;
    for index in 0..model::ACTIVE{
        let physical=private_region(info.arena,index);
        let process=&mut runtime.processes[index];
        process.state=State::Runnable;process.root=physical;process.entry=layout.entry;
        process.kernel_bottom=physical+KERNEL_BOTTOM;process.kernel_top=physical+KERNEL_TOP;
        unsafe{ptr::copy_nonoverlapping(SERVICE.as_ptr(),(physical+IMAGE)as *mut u8,layout.header_size);}
        for section in &layout.sections[..layout.count]{
            unsafe{ptr::copy_nonoverlapping(SERVICE.as_ptr().add(section.file_offset),
                (physical+IMAGE+section.virtual_offset as u64)as *mut u8,section.file_size);}
        }
        let mut handoff=abi::Boot{magic:abi::MAGIC,role:index as u64,generation:1,entry:layout.entry,kernel_probe:info.arena,peer_probe:private_region(info.arena,0)+USER_STACK,
            framebuffer:0,width:0,height:0,pitch:0,format:0,caps:[0;11]};
        if model::receives(index){handoff.caps[abi::SELF_RECV]=grant(process,abi::SELF_RECV,endpoint(index),model::RECEIVE);}
        if [0,1,3].contains(&index){
            process.queue=Queue::with_sender_limit(2).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=QUEUE-QUOTA"));
        }
        for slot in 1..=6 {
            if let Some(target)=model::send_target(index,slot) {
                handoff.caps[slot]=grant(process,slot,endpoint(target),model::SEND);
            }
        }
        if index==2{handoff.caps[abi::INPUT]=grant(process,abi::INPUT,Object::Input,model::PORT_READ);}
        if index==3{
            handoff.caps[abi::FRAMEBUFFER]=grant(process,abi::FRAMEBUFFER,Object::Framebuffer,model::DRAW);
            handoff.framebuffer=0x800000;handoff.width=640;handoff.height=480;
            handoff.pitch=info.platform.pitch;handoff.format=info.platform.format;
        }
        unsafe{((physical+BOOT)as *mut abi::Boot).write(handoff);}
        let mut tables=unsafe{Tables::new(physical)};
        for page in 0..boot::ARENA_PAGES{
            let address=info.arena+page as u64*4096;
            if omit(info.arena,address){continue;}
            unsafe{tables.map(mapping(address,address,1,true,false),info.arena,info.arena+boot::ARENA_PAGES as u64*4096)}
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=PROCESS-KERNEL-MAP"));
        }
        unsafe{boot::map_image(&mut tables,info.platform.image_base,info.platform.image_size);}
        unsafe{add(&mut tables,process,model::pe::BASE,physical+IMAGE,1,false,false,false);}
        for section in &layout.sections[..layout.count]{
            unsafe{add(&mut tables,process,model::pe::BASE+section.virtual_offset as u64,
                physical+IMAGE+section.virtual_offset as u64,section.memory_size.div_ceil(4096)as u64,
                section.writable,section.executable,false);}
        }
        unsafe{
            add(&mut tables,process,STACK_VA,physical+USER_STACK,16,true,false,false);
            add(&mut tables,process,abi::BOOT_ADDRESS as u64,physical+BOOT,1,false,false,false);
        }
        if index==3{unsafe{add(&mut tables,process,0x800000,info.platform.framebuffer,
            info.platform.framebuffer_bytes/4096,true,false,true);}}
        // Initial architectural state contains no kernel register/SIMD bytes.
        let frame=process.kernel_top-720;
        unsafe{
            ptr::write_bytes(frame as *mut u8,0,720);
            (frame as *mut u16).write(0x37f);((frame+24)as *mut u32).write(0x1f80);
            ((frame+512)as *mut arch::Trap).write(arch::Trap{rip:layout.entry,rsp:STACK_END-40,..arch::Trap::EMPTY});
        }
        process.frame=frame;
    }
    // Retire writable bootstrap aliases as well, before executing any user page.
    let mut old=unsafe{Tables::resume(info.arena,info.table_used)};
    for index in 0..TASKS{
        let base=private_region(info.arena,index);
        for offset in (IMAGE..USER_STACK+0x10000).step_by(4096){
            unsafe{old.unmap(base+offset)}.unwrap_or_else(|_|fatal("RAR-PANIC:CODE=ALIAS-RETIRE"));
        }
        unsafe{old.unmap(base+BOOT)}.unwrap_or_else(|_|fatal("RAR-PANIC:CODE=BOOT-ALIAS"));
    }
    unsafe{display::keyboard();arch::install(info.arena);}
    record("RAR-DESKTOP:PROCESSES-READY");
    let first=runtime.processes[0];
    unsafe{arch::activate(first.root,first.kernel_top);arch::first(first.frame)}
}
fn number(error:Error)->u64{
    (match error{Error::Invalid=>-1i64,Error::Denied=>-2,Error::Stale=>-3,Error::Full=>-4,Error::Empty=>-5,Error::Exhausted=>-6})as u64
}
impl Runtime{
    fn resolve(&mut self,handle:u64,right:u8)->Result<Object,Error>{
        self.processes[self.current].caps.resolve(handle,right)
    }
    fn buffer(&mut self,pointer:u64,length:usize,write:bool)->Result<(),Error>{
        self.processes[self.current].buffer(pointer,length,write)
    }
    fn sys(&mut self,frame:&arch::Trap)->Result<u64,Error>{
        let current=self.current;
        match frame.rax{
            abi::YIELD=>Ok(0),
            abi::SEND=>{
                let Object::Endpoint{task,generation}=self.resolve(frame.rdi,model::SEND)? else{return Err(Error::Denied);};
                let target=task as usize;
                if target>=model::ACTIVE{return Err(Error::Invalid);}
                if self.processes[target].state==State::Dead||self.processes[target].generation!=generation{
                    return Err(Error::Stale);
                }
                let length=usize::try_from(frame.rdx).map_err(|_|Error::Invalid)?;
                if length==0||length>128{return Err(Error::Invalid);}
                self.buffer(frame.rsi,length,false)?;
                let bytes=unsafe{core::slice::from_raw_parts(frame.rsi as *const u8,length)};
                let message=Message::from_kernel_sender(current,self.processes[current].generation,bytes)?;
                let result=self.processes[target].queue.push(message);
                result?;
                if self.processes[target].state==State::Blocked{
                    self.processes[target].state=State::Runnable;
                }
                Ok(0)
            }
            abi::RECEIVE=>{
                let Object::Endpoint{task,generation}=self.resolve(frame.rdi,model::RECEIVE)? else{return Err(Error::Denied);};
                if task as usize!=current||generation!=self.processes[current].generation{return Err(Error::Denied);}
                if frame.rdx!=144||frame.r10>1{return Err(Error::Invalid);}
                self.buffer(frame.rsi,144,true)?;
                let message=match self.processes[current].queue.peek(){
                    Ok(value)=>value,
                    Err(Error::Empty)=>{
                        if frame.r10==1{
                            self.processes[current].state=State::Blocked;
                        }
                        return Err(Error::Empty);
                    }
                    Err(error)=>return Err(error),
                };
                let envelope=abi::Envelope{sender:message.sender as u64,generation:message.generation,
                    length:message.length as u32,bytes:message.bytes};
                // Single CPU, IF=0: the validated mappings cannot change mid-copy.
                unsafe{ptr::copy_nonoverlapping((&envelope as *const abi::Envelope).cast::<u8>(),frame.rsi as *mut u8,144);}
                self.processes[current].queue.pop()?;
                Ok(message.length as u64)
            }
            abi::PORT_READ=>{
                if self.resolve(frame.rdi,model::PORT_READ)?!=Object::Input||![0x60,0x64].contains(&frame.rsi){return Err(Error::Denied);}
                Ok(unsafe{crate::input(frame.rsi as u16)}as u64)
            }
            abi::REPORT=>{
                let code=frame.rdi;
                if !matches!((current,code),(2,1)|(3,2)) {
                    record("RAR-DESKTOP:TEST-FAILED");return Err(Error::Denied);
                }
                let bit=1u8<<code;
                if self.proofs&bit!=0{return Err(Error::Denied);}
                self.proofs|=bit;
                if self.proofs==6&&!self.ready{self.ready=true;record("RAR-DESKTOP-READY");}
                Ok(0)
            }
            abi::EXIT=>{self.kill(current);Ok(0)}
            _=>Err(Error::Invalid),
        }
    }
    fn kill(&mut self,index:usize){
        self.processes[index].state=State::Dead;
        self.processes[index].generation=self.processes[index].generation.saturating_add(1);
        self.processes[index].queue=Queue::new();
        for slot in 0..model::CAP_SLOTS{let _=self.processes[index].caps.revoke(slot);}
    }
}
fn user_return_valid(process:&Process,ret:&arch::Trap)->bool{
    ret.cs==0x1b&&ret.ss==0x23&&(STACK_VA..=STACK_END).contains(&ret.rsp)&&
        process.ranges[..process.range_count].iter().any(|r|r.executable&&r.start<=ret.rip&&ret.rip<r.end)&&
        [ret.ds,ret.es,ret.fs,ret.gs].iter().all(|s|[0,0x1b,0x23].contains(s))
}

/// Called only by the assembly trap gate. Kernel faults are fatal. User faults
/// destroy only their process authority; unrelated services remain scheduled.
pub extern "sysv64" fn trap(frame:*mut arch::Trap,saved:u64)->u64{
    let state=unsafe{&mut *ptr::addr_of_mut!(RUNTIME)};
    let current=state.current;
    let f=unsafe{&mut *frame};
    if current>=TASKS||f.cs&3!=3||matches!(f.vector,2|8){fatal("RAR-PANIC:CODE=KERNEL-TRAP");}
    let process=&state.processes[current];
    if saved%16!=0||saved<process.kernel_bottom||saved+720>process.kernel_top||frame as u64!=saved+512{
        fatal("RAR-PANIC:CODE=TRAP-STACK");
    }
    state.processes[current].frame=saved;
    match f.vector{
        32=>{
            unsafe{crate::out(0x20,0x20);}
            state.processes[current].preemptions=state.processes[current].preemptions.saturating_add(1);
        }
        128=>{
            f.rax=match state.sys(f){Ok(value)=>value,Err(error)=>number(error)};
        }
        0..=31=>{
            if current==6&&f.vector==6&&f.error==0{
                record("RAR-DESKTOP:APP-FAULT=6");
            }else{record("RAR-DESKTOP:UNEXPECTED-USER-FAULT");}
            state.kill(current);
        }
        _=>fatal("RAR-PANIC:CODE=TRAP-VECTOR"),
    }
    // Classify the current invalid user frame immediately, before another
    // service can attempt a response to its now-dead endpoint.
    if state.processes[current].state!=State::Dead&&!user_return_valid(&state.processes[current],f){
        record("RAR-DESKTOP:INVALID-USER-RETURN");
        state.kill(current);
    }
    // Invalid user return registers are user faults, never a kernel-wide panic.
    // Kernel-owned frame/root/stack corruption remains a fatal invariant failure.
    let mut cursor=current;
    for _ in 0..TASKS{
        let states=core::array::from_fn(|index|state.processes[index].state);
        let next=model::next(&states,cursor).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=NO-RUNNABLE"));
        let process=&state.processes[next];
        let expected=private_region(state.arena,next);
        if process.root!=expected||process.kernel_bottom!=expected+KERNEL_BOTTOM||
            process.kernel_top!=expected+KERNEL_TOP||process.frame%16!=0||
            process.frame<process.kernel_bottom||process.frame.checked_add(720).is_none_or(|end|end>process.kernel_top){
            fatal("RAR-PANIC:CODE=OWNED-RETURN-FRAME");
        }
        let ret=unsafe{&mut *((process.frame+512)as *mut arch::Trap)};
        let valid=user_return_valid(process,ret);
        if !valid{
            record("RAR-DESKTOP:INVALID-USER-RETURN");
            state.kill(next);cursor=next;continue;
        }
        // Preserve arithmetic condition flags and DF; deny privileged/tracing modes.
        ret.flags=(ret.flags&0xcd5)|0x202;
        let (root,stack,result)=(process.root,process.kernel_top,process.frame);
        state.current=next;
        unsafe{arch::activate(root,stack);}
        return result;
    }
    fatal("RAR-PANIC:CODE=RETURN-SELECTION")
}
