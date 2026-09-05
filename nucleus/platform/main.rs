//! Experimental Platform runtime: mechanisms in ring0, service policy in ring3.
mod model;
mod arch;
pub(crate) mod display;
#[path="../../core/platform/abi.rs"]
mod abi;
use core::{arch::asm,mem,ptr};
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
const FAULT_MASK:u16=(1<<6)|(1<<7)|(1<<8)|(1<<9)|(1<<11)|(1<<12)|(1<<13);
const EMPTY_RANGE:UserRange=UserRange{start:0,end:0,writable:false,executable:false};
static SERVICE:&[u8]=include_bytes!("/tmp/platform-service.efi");
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
    processes:[Process;TASKS],current:usize,arena:u64,faults:u16,proofs:u16,phase:u8,
    bad_caps:u32,bad_buffers:u32,full_queues:u32,self_received:u32,survivor:bool,peer_death:bool,failed:bool,
}
static mut RUNTIME:Runtime=Runtime{processes:[Process::EMPTY;TASKS],current:0,arena:0,faults:0,proofs:0,phase:0,
    bad_caps:0,bad_buffers:0,full_queues:0,self_received:0,survivor:false,peer_death:false,failed:false};
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
/// All fixture destinations are exclusively owned zeroed arena regions, not user
/// data. Copying finishes before roots omit kernel aliases and expose RX pages.
pub unsafe fn start(info:&boot::BootInfo)->!{
    if mem::size_of::<arch::Trap>()!=208||mem::size_of::<abi::Envelope>()!=144||
        info.arena<0x2000000||info.platform.image_base<0x2000000{
        fatal("RAR-PANIC:CODE=PLATFORM-LAYOUT");
    }
    let layout=model::pe::parse(SERVICE).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=SERVICE-PE"));
    let runtime=unsafe{&mut *ptr::addr_of_mut!(RUNTIME)};
    runtime.arena=info.arena;
    for index in 0..TASKS{
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
            framebuffer:0,width:0,height:0,pitch:0,format:0,caps:[0;10]};
        if [0,1,10].contains(&index){handoff.caps[abi::SELF_RECV]=grant(process,abi::SELF_RECV,endpoint(index),model::RECEIVE);}
        if [0,10].contains(&index){handoff.caps[abi::STORAGE]=grant(process,abi::STORAGE,endpoint(1),model::SEND);}
        if index==1{
            handoff.caps[abi::CLIENT]=grant(process,abi::CLIENT,endpoint(0),model::SEND);
            handoff.caps[abi::SECOND_CLIENT]=grant(process,abi::SECOND_CLIENT,endpoint(10),model::SEND);
        }
        if index==0{
            handoff.caps[abi::DEAD_PEER]=grant(process,abi::DEAD_PEER,endpoint(6),model::SEND);
            handoff.caps[abi::SELF_SEND]=grant(process,abi::SELF_SEND,endpoint(0),model::SEND);
            handoff.caps[abi::STALE]=grant(process,abi::STALE,Object::Input,model::PORT_READ);
            process.caps.revoke(abi::STALE).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=REVOKE"));
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
    record("RAR-PLATFORM:PROCESSES-READY");
    let first=runtime.processes[0];
    unsafe{arch::activate(first.root,first.kernel_top);arch::first(first.frame)}
}
fn number(error:Error)->u64{
    (match error{Error::Invalid=>-1i64,Error::Denied=>-2,Error::Stale=>-3,Error::Full=>-4,Error::Empty=>-5,Error::Exhausted=>-6})as u64
}
impl Runtime{
    fn resolve(&mut self,handle:u64,right:u8)->Result<Object,Error>{
        let result=self.processes[self.current].caps.resolve(handle,right);
        if result.is_err()&&self.current==0{self.bad_caps=self.bad_caps.saturating_add(1);}
        result
    }
    fn buffer(&mut self,pointer:u64,length:usize,write:bool)->Result<(),Error>{
        let result=self.processes[self.current].buffer(pointer,length,write);
        if result.is_err()&&self.current==0{self.bad_buffers=self.bad_buffers.saturating_add(1);}
        result
    }
    fn sys(&mut self,frame:&arch::Trap)->Result<u64,Error>{
        let current=self.current;
        match frame.rax{
            abi::YIELD=>Ok(0),
            abi::SEND=>{
                let Object::Endpoint{task,generation}=self.resolve(frame.rdi,model::SEND)? else{return Err(Error::Denied);};
                let target=task as usize;
                if self.processes[target].state==State::Dead||self.processes[target].generation!=generation{
                    if current==0{self.peer_death=true;}return Err(Error::Stale);
                }
                let length=usize::try_from(frame.rdx).map_err(|_|Error::Invalid)?;
                if length==0||length>128{return Err(Error::Invalid);}
                self.buffer(frame.rsi,length,false)?;
                let bytes=unsafe{core::slice::from_raw_parts(frame.rsi as *const u8,length)};
                let message=Message::from_kernel_sender(current,self.processes[current].generation,bytes)?;
                let result=self.processes[target].queue.push(message);
                if result==Err(Error::Full)&&current==0{self.full_queues=self.full_queues.saturating_add(1);}
                result?;
                if self.processes[target].state==State::Blocked{self.processes[target].state=State::Runnable;}
                Ok(0)
            }
            abi::RECEIVE=>{
                let Object::Endpoint{task,generation}=self.resolve(frame.rdi,model::RECEIVE)? else{return Err(Error::Denied);};
                if task as usize!=current||generation!=self.processes[current].generation{return Err(Error::Denied);}
                if frame.rdx!=144||frame.r10>1{return Err(Error::Invalid);}
                self.buffer(frame.rsi,144,true)?;
                let message=match self.processes[current].queue.peek(){
                    Ok(value)=>value,
                    Err(Error::Empty)=>{if frame.r10==1{self.processes[current].state=State::Blocked;}return Err(Error::Empty);}
                    Err(error)=>return Err(error),
                };
                let envelope=abi::Envelope{sender:message.sender as u64,generation:message.generation,
                    length:message.length as u32,bytes:message.bytes};
                // Single CPU, IF=0: the validated mappings cannot change mid-copy.
                unsafe{ptr::copy_nonoverlapping((&envelope as *const abi::Envelope).cast::<u8>(),frame.rsi as *mut u8,144);}
                self.processes[current].queue.pop()?;
                if current==0&&message.sender==1&&self.faults==FAULT_MASK&&self.processes[5].preemptions>=2{self.survivor=true;}
                if current==0&&message.sender==0{self.self_received=self.self_received.saturating_add(1);}
                Ok(message.length as u64)
            }
            abi::PORT_READ=>{
                if self.resolve(frame.rdi,model::PORT_READ)?!=Object::Input||![0x60,0x64].contains(&frame.rsi){return Err(Error::Denied);}
                Ok(unsafe{crate::input(frame.rsi as u16)}as u64)
            }
            abi::REPORT=>{
                let code=frame.rdi;
                let allowed=match code{
                    1=>current==0&&self.bad_caps>=3&&self.bad_buffers>=3,
                    2=>current==0&&self.full_queues>=1&&self.self_received==4&&self.bad_buffers>=4,
                    3=>current==0,4=>current==4&&self.processes[4].preemptions>=2,
                    5=>current==3,6=>current==2,7=>current==2&&self.phase==1,8=>current==10,
                    _=>false,
                };
                if !allowed{self.failed=true;record("RAR-PLATFORM:TEST-FAILED");return Err(Error::Denied);}
                if self.proofs&(1<<code)!=0{return Err(Error::Denied);}
                self.proofs|=1<<code;
                if code==7{
                    record("RAR-PLATFORM:INPUT-PASS");record("RAR-PLATFORM:CAPTURE");
                    record("RAR-PLATFORM-READY");self.phase=2;
                }
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
    fn progress(&mut self){
        let required=(1<<1)|(1<<2)|(1<<3)|(1<<4)|(1<<5)|(1<<6)|(1<<8);
        if !self.failed&&self.phase==0&&self.proofs&required==required&&self.faults==FAULT_MASK&&
            self.processes[5].preemptions>=2&&self.survivor&&self.peer_death{
            for line in ["RAR-PLATFORM:PREEMPTION-PASS","RAR-PLATFORM:CONTEXT-PASS",
                "RAR-PLATFORM:CAPABILITIES-PASS","RAR-PLATFORM:IPC-PASS","RAR-PLATFORM:STORAGE-PASS",
                "RAR-PLATFORM:FAULT-CONTAINMENT-PASS"]{record(line);}
            self.phase=1;record("RAR-PLATFORM:INPUT-WAIT");
        }
    }
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
            let address:u64;unsafe{asm!("mov {},cr2",out(reg)address,options(nostack));}
            let expected=match current{
                6=>f.vector==14&&f.error==5&&address==state.arena,
                7=>f.vector==14&&f.error==21&&address==STACK_VA,
                8=>f.vector==14&&f.error==6&&address==STACK_VA-4096,
                9=>f.vector==13&&f.error==0,
                11=>f.vector==14&&f.error==7&&address==state.processes[11].entry,
                12=>f.vector==14&&f.error==4&&address==private_region(state.arena,0)+USER_STACK,
                13=>f.vector==13&&f.error==0,
                _=>false,
            };
            if expected{state.faults|=1<<current;}else{state.failed=true;record("RAR-PLATFORM:UNEXPECTED-USER-FAULT");}
            state.kill(current);
        }
        _=>fatal("RAR-PANIC:CODE=TRAP-VECTOR"),
    }
    state.progress();
    let states=core::array::from_fn(|index|state.processes[index].state);
    let next=model::next(&states,current).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=NO-RUNNABLE"));
    let process=&state.processes[next];
    let ret=unsafe{&mut *((process.frame+512)as *mut arch::Trap)};
    if ret.cs!=0x1b||ret.ss!=0x23||!(STACK_VA..=STACK_END).contains(&ret.rsp)||
        !process.ranges[..process.range_count].iter().any(|r|r.executable&&r.start<=ret.rip&&ret.rip<r.end)||
        ![ret.ds,ret.es,ret.fs,ret.gs].iter().all(|s|[0,0x1b,0x23].contains(s)){
        fatal("RAR-PANIC:CODE=USER-RETURN");
    }
    // Preserve arithmetic condition flags and DF; deny privileged/tracing modes.
    ret.flags=(ret.flags&0xcd5)|0x202;
    let (root,stack,result)=(process.root,process.kernel_top,process.frame);
    state.current=next;
    unsafe{arch::activate(root,stack);}
    result
}
