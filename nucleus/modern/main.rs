//! Modern kernel entry: protected processes with Modern policy and fixed PIO.
//! Compiled only by a distinct, not-yet-activated cloud Modern composition.
mod model;
mod support;
mod retirement;
mod loader;
mod lab_images;
#[path="../../core/modern/lab_input.rs"] mod lab_input;
pub(crate) mod staging;
mod native_pio;
#[path="../platform/arch.rs"] mod arch;
#[path="../platform/display.rs"] pub(crate) mod display;
#[path="../../core/modern/abi.rs"] mod abi;
#[path="../platform/pe.rs"] mod pe;
use core::{mem,ptr};
use crate::{boot,fatal,record,paging::Tables,model::Mapping};
use model::Error;
use support::{CpuState as State,UserRange,TASKS};
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
#[cfg(not(rar_modern_compile_only))]
static SERVICE:&[u8]=include_bytes!("/tmp/modern-service.efi");
// Object-only Linux cloud check: no target image and no execution. The UEFI
// build rejects this cfg at the Foundation root. Runtime PE parsing rejects [].
#[cfg(rar_modern_compile_only)]
static SERVICE:&[u8]=&[];
#[derive(Clone,Copy)]
struct Process{
    memory:retirement::Memory,aperture:u64,table_used:usize,
    state:State,generation:u64,root:u64,kernel_bottom:u64,kernel_top:u64,frame:u64,
    ranges:[UserRange;24],range_count:usize,preemptions:u64,entry:u64,stack_end:u64,
}
impl Process{
    const EMPTY:Self=Self{memory:retirement::Memory::Clean,aperture:0,table_used:0,state:State::Dead,generation:1,root:0,kernel_bottom:0,kernel_top:0,frame:0,
        ranges:[EMPTY_RANGE;24],range_count:0,preemptions:0,entry:0,stack_end:0};
    fn range(&mut self,start:u64,end:u64,writable:bool,executable:bool){
        if self.range_count>=self.ranges.len()||start>=end||writable&&executable{fatal("RAR-PANIC:CODE=USER-RANGE");}
        self.ranges[self.range_count]=UserRange{start,end,writable,executable};self.range_count+=1;
    }
    fn buffer(&self,pointer:u64,length:usize,write:bool)->Result<(),Error>{
        support::user_buffer(&self.ranges[..self.range_count],pointer,length,write)
    }
}
struct NativeDesktop{plan:model::DesktopHandover,boot:abi::Boot,seal:u64,token:u64}
struct Runtime{
    processes:[Process;TASKS],current:usize,arena:u64,proofs:u8,ready:bool,
    image_base:u64,image_size:u64,hardware:BootHardware,desktop:Option<NativeDesktop>,
    policy:Option<model::Runtime>,device:Option<native_pio::Adapter>,ticks:Option<u64>,
    handover:Option<(model::Handover,abi::Boot,u64)>,
    staging:Option<staging::Buffer<'static>>,bootstrap_tables:usize,stage_readonly:bool,stage_view:bool,
}
static mut RUNTIME:Runtime=Runtime{processes:[Process::EMPTY;TASKS],current:0,arena:0,proofs:0,ready:false,image_base:0,image_size:0,hardware:BootHardware::EMPTY,desktop:None,
    policy:None,device:None,ticks:Some(0),handover:None,staging:None,bootstrap_tables:0,stage_readonly:false,stage_view:false};
fn private_region(arena:u64,index:usize)->u64{
    retirement::region(arena,boot::ARENA_PAGES,index)
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=PRIVATE-GEOMETRY"))
}
fn omit(arena:u64,address:u64)->bool{
    let offset=address-arena;
    if staging::guard_offset(offset){return true;}
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
/// Safety: tables owns the current process's kernel-only page-table allocation;
/// p/pages designate validated private arena or validated GOP bytes. Virtual and
/// physical spans are aligned and non-overflowing, W+X is forbidden by map_user.
/// The caller alone selects device=true for compositor's fixed framebuffer.
/// Mapping and range registration finish before any user instruction executes.
/// Modern full_envelope_checked_without_desktop_truncation and
/// fixed_geometry_and_framebuffer_bounds plus Foundation map-user tests exercise
/// rejection boundaries without dereferencing invalid host pointers.
unsafe fn add(tables:&mut Tables,process:&mut Process,v:u64,p:u64,pages:u64,w:bool,x:bool,device:bool){
    unsafe{tables.map_user(mapping(v,p,pages,w,x),p,p+pages*4096,device)}
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=USER-MAP"));
    process.range(v,v+pages*4096,w,x);
}
/// # Safety
/// Execute only inside the independently certified Modern cloud VM, never an
/// existing Desktop profile or physical device. The two fixed synthetic PIO
/// devices must match support::DATA/SYSTEM and the adapter invariants. This
/// source/build flag is NOT such certification or authorization.
/// All process destinations are exclusively owned zeroed arena regions, not user
/// data. Copying finishes before roots omit kernel aliases and expose RX pages.
pub unsafe fn start(info:&boot::BootInfo)->!{
    if mem::size_of::<arch::Trap>()!=208||mem::size_of::<abi::Envelope>()!=152||mem::size_of::<abi::Boot>()!=368||
        info.arena<0x2000000||info.platform.image_base<0x2000000{
        fatal("RAR-PANIC:CODE=PLATFORM-LAYOUT");
    }
    let layout=pe::parse(SERVICE).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=SERVICE-PE"));
    let runtime=unsafe{&mut *ptr::addr_of_mut!(RUNTIME)};
    runtime.arena=info.arena;
    runtime.hardware=info.platform;
    runtime.image_base=info.platform.image_base;runtime.image_size=info.platform.image_size;
    runtime.bootstrap_tables=info.table_used;
    let stage_region=staging::region(info.arena,boot::ARENA_PAGES)
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGING-GEOMETRY"));
    if runtime.staging.is_some(){fatal("RAR-PANIC:CODE=STAGING-REINITIALIZE");}
    // SAFETY: UEFI allocated/zeroed this exact Modern arena before entering.
    // Checked513-page region is disjoint from all16 process strides, stack/
    // table/Boot allocations and both unmapped guards. No user mappings name
    // it. Kernel start runs once with IF=0 before any process. This sole static
    // mutable borrow remains owned by Runtime for the whole boot session;
    // no recovery path may reconstruct Buffer and reset the seal sequence.
    runtime.staging=Some(staging::Buffer::new(unsafe{
        core::slice::from_raw_parts_mut(stage_region.start as *mut u8,staging::BUFFER_BYTES)
    }).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGING-BUFFER")));
    runtime.policy=Some(model::Runtime::new());
    for index in support::INITIAL{
        let handoff=support::bootstrap(runtime.policy.as_ref().unwrap(),index,layout.entry,
            info.platform.pitch,info.platform.format).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=MODERN-BOOT"));
        let physical=private_region(info.arena,index);
        let process=&mut runtime.processes[index];
        process.state=State::Runnable;process.root=physical;process.entry=layout.entry;process.stack_end=STACK_END;
        process.generation=handoff.generation;
        process.kernel_bottom=physical+KERNEL_BOTTOM;process.kernel_top=physical+KERNEL_TOP;
        unsafe{ptr::copy_nonoverlapping(SERVICE.as_ptr(),(physical+IMAGE)as *mut u8,layout.header_size);}
        for section in &layout.sections[..layout.count]{
            unsafe{ptr::copy_nonoverlapping(SERVICE.as_ptr().add(section.file_offset),
                (physical+IMAGE+section.virtual_offset as u64)as *mut u8,section.file_size);}
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
        unsafe{add(&mut tables,process,pe::BASE,physical+IMAGE,1,false,false,false);}
        for section in &layout.sections[..layout.count]{
            unsafe{add(&mut tables,process,pe::BASE+section.virtual_offset as u64,
                physical+IMAGE+section.virtual_offset as u64,section.memory_size.div_ceil(4096)as u64,
                section.writable,section.executable,false);}
        }
        unsafe{
            add(&mut tables,process,STACK_VA,physical+USER_STACK,16,true,false,false);
            add(&mut tables,process,abi::BOOT_ADDRESS as u64,physical+BOOT,1,false,false,false);
        }
        if index==3{unsafe{add(&mut tables,process,0x800000,info.platform.framebuffer,
            info.platform.framebuffer_bytes/4096,true,false,true);}}
        if index==9{
            for (index,bank)in lab_images::all().into_iter().enumerate(){
                let Some((bytes,length))=bank else{continue;};
                let window=lab_input::window(index,length,bytes.len(),bytes.as_ptr()as u64,
                    (runtime.image_base,runtime.image_size),(runtime.arena,boot::ARENA_PAGES as u64*4096))
                    .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=LAB-INPUT-BOUNDS"));
                if bytes[length..].iter().any(|&b|b!=0){fatal("RAR-PANIC:CODE=LAB-INPUT-PADDING");}
                // SAFETY: page-aligned dedicated immutable static object, exact
                // image-contained extent, disjoint from arena/other windows;
                // padding is initialized. Sole System mapping, always RO/NX.
                unsafe{add(&mut tables,process,window.address,window.physical,window.pages,false,false,false);}
            }
        }
        // Initial architectural state contains no kernel register/SIMD bytes.
        let frame=process.kernel_top-720;
        unsafe{
            ptr::write_bytes(frame as *mut u8,0,720);
            (frame as *mut u16).write(0x37f);((frame+24)as *mut u32).write(0x1f80);
            ((frame+512)as *mut arch::Trap).write(arch::Trap{rip:layout.entry,rsp:STACK_END-40,..arch::Trap::EMPTY});
        }
        process.frame=frame;
        process.aperture=unsafe{tables.reserve_modern_aperture()}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=RETIRE-APERTURE"));
        if index==8{unsafe{tables.reserve_modern_verifier()}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=VERIFIER-RESERVE"));}
        process.table_used=tables.used();
        process.memory=retirement::Memory::Live;
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
    // SAFETY: this entry may run only after independent certification of the
    // distinct Modern cloud profile described in modern-runtime-v1.md. UEFI has
    // exited, PIC IRQ14/15 are masked, sole CPU/IF=0, no storage user has run.
    // The adapter never retries or resets; any failed initialization halts.
    runtime.device=Some(unsafe{native_pio::Adapter::initialize()}
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=MODERN-PIO-INIT")));
    record("RAR-MODERN:PROCESSES-READY");
    let first=runtime.processes[0];
    unsafe{arch::activate(first.root,first.kernel_top);arch::first(first.frame)}
}
fn number(error:Error)->u64{
    (match error{Error::Invalid=>-1i64,Error::Denied=>-2,Error::Stale=>-3,
        Error::Full=>-4,Error::Empty=>-5,Error::Exhausted=>-6,Error::Busy=>-7})as u64
}
fn stage_error(error:staging::Error)->Error{
    match error{
        staging::Error::Busy=>Error::Busy,staging::Error::Stale=>Error::Stale,
        staging::Error::Exhausted=>Error::Exhausted,
        staging::Error::Bounds|staging::Error::Order|staging::Error::Incomplete|staging::Error::State=>Error::Invalid,
    }
}
impl Runtime{
    fn buffer(&self,pointer:u64,length:usize,write:bool)->Result<(),Error>{
        self.processes[self.current].buffer(pointer,length,write)
    }
    /// Called only by validated ring3 int80 trap with IF=0. Current identity,
    /// page ranges, policy and adapter belong to the kernel, never request bytes.
    fn sys(&mut self,frame:&arch::Trap)->Result<u64,Error>{
        let current=self.current;
        // Dedicated idle CPU context is not a logical principal.
        if current==15 && !matches!(frame.rax,abi::YIELD|abi::TICKS|abi::EXIT){
            return Err(Error::Denied);
        }
        match frame.rax{
            abi::YIELD=>Ok(0),
            abi::SEND=>{
                let length=usize::try_from(frame.rdx).map_err(|_|Error::Invalid)?;
                if length==0||length>128{return Err(Error::Invalid);}
                self.buffer(frame.rsi,length,false)?;
                // SAFETY: current root's complete readable span validated above;
                // no scheduling/revocation is possible during this IF=0 copy.
                let bytes=unsafe{core::slice::from_raw_parts(frame.rsi as *const u8,length)};
                self.policy.as_mut().ok_or(Error::Denied)?.send(current,frame.rdi,bytes)?;
                // Bounded spurious wakeups avoid a second endpoint resolution:
                // only logically active blocked contexts may become runnable.
                for i in 0..TASKS{
                    if self.processes[i].state==State::Blocked &&
                        self.policy.as_ref().unwrap().state(i)?==model::State::Active{
                        self.processes[i].state=State::Runnable;
                    }
                }
                Ok(0)
            }
            abi::RECEIVE=>{
                // Shared tested boundary validates all 152 bytes before pop.
                let process=&self.processes[current];
                let m=match support::receive(self.policy.as_mut().ok_or(Error::Denied)?,
                    current,frame.rdi,&process.ranges[..process.range_count],
                    frame.rsi,frame.rdx,frame.r10){
                    Ok(m)=>m,
                    Err(Error::Empty)=>{
                        if frame.r10==1{self.processes[current].state=State::Blocked;}
                        return Err(Error::Empty);
                    },
                    Err(e)=>return Err(e),
                };
                let envelope=abi::Envelope{sender:m.principal as u64,generation:m.incarnation,
                    length:m.length as u64,bytes:m.bytes};
                // SAFETY: complete owned writable range checked before pop;
                // source is fully initialized/padding-free, sole CPU and IF=0.
                unsafe{ptr::copy_nonoverlapping((&envelope as *const abi::Envelope).cast::<u8>(),
                    frame.rsi as *mut u8,abi::ENVELOPE_BYTES as usize);}
                Ok(m.length as u64)
            }
            abi::PORT_READ=>{
                self.policy.as_ref().ok_or(Error::Denied)?.input(current,frame.rdi)?;
                if ![0x60,0x64].contains(&frame.rsi){return Err(Error::Denied);}
                // SAFETY: caller-local keyboard authority and exact fixed PS/2
                // status/data port allowlist, sole CPU/IF=0 certified VM only.
                let value:u8;
                unsafe{core::arch::asm!("in al, dx",in("dx")frame.rsi as u16,
                    out("al")value,options(nostack,preserves_flags));}
                Ok(value as u64)
            }
            abi::REPORT=>{
                if !matches!((current,frame.rdi),(2,1)|(3,2)){return Err(Error::Denied);}
                let bit=1u8<<frame.rdi;
                if self.proofs&bit!=0{return Err(Error::Denied);}
                self.proofs|=bit;
                if self.proofs==6&&!self.ready{
                    self.ready=true;record("RAR-MODERN:GUI-READY");
                }
                Ok(0)
            }
            abi::EXIT=>{self.kill(current);Ok(0)}
            abi::TICKS=>self.ticks.ok_or(Error::Exhausted),
            abi::DEVICE=>{
                let policy=self.policy.as_ref().ok_or(Error::Denied)?;
                let device=self.device.as_mut().ok_or(Error::Denied)?;
                // SAFETY: profile/initialization invariants above remain true.
                // Current is saved CPU ownership; policy and device borrow are
                // serialized with revocation under the trap's IF=0 invariant.
                unsafe{device.execute(policy,current,frame.rdi,frame.rsi,frame.rdx,frame.r10)}
            }
            abi::LAB_INPUT=>{
                self.policy.as_ref().ok_or(Error::Denied)?.stage_copy(current,frame.rdi)?;
                if frame.r10!=16{return Err(Error::Invalid);}
                let index=usize::try_from(frame.rsi).map_err(|_|Error::Invalid)?;
                let bank=lab_images::all().get(index).copied().flatten().ok_or(Error::Invalid)?;
                let address=lab_input::BASE+index as u64*lab_input::STRIDE;
                self.buffer(frame.rdx,16,true)?;
                let mut reply=[0u8;16];
                reply[..8].copy_from_slice(&address.to_le_bytes());
                reply[8..].copy_from_slice(&(bank.1 as u64).to_le_bytes());
                // SAFETY: System-owned exact output validated; immutable input
                // windows were checked and installed once before user execution.
                unsafe{ptr::copy_nonoverlapping(reply.as_ptr(),frame.rdx as *mut u8,16);}
                Ok(0)
            }
            abi::STAGE_COPY=>{
                self.policy.as_ref().ok_or(Error::Denied)?.stage_copy(current,frame.rdi)?;
                if frame.rdx!=abi::STAGE_REQUEST_BYTES as u64||frame.r10!=0{return Err(Error::Invalid);}
                self.buffer(frame.rsi,abi::STAGE_REQUEST_BYTES,false)?;
                let mut raw=[0u8;abi::STAGE_REQUEST_BYTES];
                // SAFETY: entire current-owned readable request validated; IF=0,
                // no scheduling or device DMA; destination is private kernel stack.
                unsafe{ptr::copy_nonoverlapping(frame.rsi as *const u8,
                    raw.as_mut_ptr(),raw.len());}
                let request=abi::stage_request(&raw).ok_or(Error::Invalid)?;
                let (seal,accepted,reply)=match request{
                    abi::StageRequest::Begin{length,reply}=>{
                        // Validate the entire result BEFORE reserving or consuming
                        // a seal. Invalid output pointers cannot strand a stage.
                        self.buffer(reply,abi::STAGE_REPLY_BYTES,true)?;
                        let clean=[5usize,7].map(|i|
                            self.processes[i].memory==retirement::Memory::Clean&&
                            self.processes[i].state==State::Dead&&self.processes[i].root==0);
                        let slot=self.policy.as_ref().unwrap().staging_slot(current,frame.rdi,clean)?;
                        let id=self.staging.as_mut().ok_or(Error::Denied)?
                            .begin(slot,length).map_err(stage_error)?;
                        (id.seal(),0,reply)
                    },
                    abi::StageRequest::Abort{seal,reply}=>{
                        self.buffer(reply,abi::STAGE_REPLY_BYTES,true)?;
                        self.staging.as_ref().ok_or(Error::Denied)?.copying(seal).map_err(stage_error)?;
                        if self.stage_readonly||self.stage_view{return Err(Error::Busy);}
                        self.scrub_stage(seal);
                        (seal,0,reply)
                    },
                    abi::StageRequest::Finish{seal,reply}=>{
                        self.buffer(reply,abi::STAGE_REPLY_BYTES,true)?;
                        let id=self.staging.as_mut().ok_or(Error::Denied)?
                            .finish(seal).map_err(stage_error)?;
                        self.seal_stage();
                        (seal,id.length(),reply)
                    },
                    abi::StageRequest::Append{seal,offset,pointer,length,reply}=>{
                        self.buffer(reply,abi::STAGE_REPLY_BYTES,true)?;
                        support::staging_buffer(&self.processes[current].ranges[..self.processes[current].range_count],
                            pointer,length)?;
                        let mut chunk=[0u8;512];
                        // SAFETY: exact1..512 byte span validated within one
                        // readable current-owned user range. Copy to private stack
                        // before mutation/reply, even if request/reply overlap it.
                        unsafe{ptr::copy_nonoverlapping(pointer as *const u8,chunk.as_mut_ptr(),length);}
                        self.staging.as_mut().ok_or(Error::Denied)?
                            .append(seal,offset,&chunk[..length]).map_err(stage_error)?;
                        (seal,offset+length,reply)
                    },
                };
                let response=abi::stage_reply(seal,accepted);
                // SAFETY: writable complete result span prevalidated before any
                // state change. Same live current root/IF=0; no intervening yield.
                unsafe{ptr::copy_nonoverlapping(response.as_ptr(),reply as *mut u8,response.len());}
                Ok(0)
            }
            abi::SETTINGS_BINDING=>{
                let e=self.policy.as_ref().ok_or(Error::Denied)?.settings_binding(current,frame.rdi)?;
                if frame.rdx!=8||frame.r10!=0{return Err(Error::Invalid);}
                self.buffer(frame.rsi,8,true)?;
                let bytes=e.map_or(0,|e|e.incarnation).to_le_bytes();
                // SAFETY: current-owned checked output, fixed binding only.
                unsafe{ptr::copy_nonoverlapping(bytes.as_ptr(),frame.rsi as *mut u8,8);}
                Ok(0)
            }
            abi::STAGE_VIEW=>{
                self.policy.as_ref().ok_or(Error::Denied)?.stage_view(current,frame.rdi)?;
                match frame.rsi{
                    0=>{
                        if frame.r10!=abi::STAGE_VIEW_BYTES as u64{return Err(Error::Invalid);}
                        self.buffer(frame.rdx,abi::STAGE_VIEW_BYTES,true)?;
                        let stage=self.staging.as_ref().ok_or(Error::Denied)?;
                        let id=stage.reserved().ok_or(Error::Stale)?;
                        stage.view(id.seal()).map_err(stage_error)?;
                        if !self.stage_readonly||!self.stage_view{return Err(Error::Busy);}
                        let mut response=[0u8;abi::STAGE_VIEW_BYTES];
                        for (i,value) in [id.seal(),id.length() as u64,abi::STAGE_VIEW_ADDRESS,id.slot() as u64].into_iter().enumerate(){
                            response[i*8..i*8+8].copy_from_slice(&value.to_le_bytes());
                        }
                        // SAFETY: current manager owns the fully checked writable
                        // reply; IF=0 prevents changes while copying initialized bytes.
                        unsafe{ptr::copy_nonoverlapping(response.as_ptr(),frame.rdx as *mut u8,response.len());}
                        Ok(0)
                    },
                    1=>{
                        if frame.r10!=0{return Err(Error::Invalid);}
                        self.policy.as_ref().unwrap().stage_reject(current,frame.rdi)?;
                        let stage=self.staging.as_ref().ok_or(Error::Denied)?;
                        stage.view(frame.rdx).map_err(stage_error)?;
                        self.reject_stage(frame.rdx);Ok(0)
                    },
                    2=>{
                        self.buffer(frame.r10,32,true)?;
                        let t=self.accept_stage(frame.rdi,frame.rdx)?;
                        let mut response=[0u8;32];
                        for (i,value) in [t.image_seal(),t.token(),t.endpoint().slot as u64,
                            t.endpoint().incarnation].into_iter().enumerate(){
                            response[i*8..i*8+8].copy_from_slice(&value.to_le_bytes());
                        }
                        // SAFETY: prechecked current manager span; no yield or
                        // change to these mappings during the IF=0 construction.
                        unsafe{ptr::copy_nonoverlapping(response.as_ptr(),frame.r10 as *mut u8,32);}
                        Ok(0)
                    },
                    3=>{
                        if frame.r10!=0{return Err(Error::Invalid);}
                        self.release_trial_stage(frame.rdi,frame.rdx)?;Ok(0)
                    },
                    4=>{
                        if frame.r10!=0{return Err(Error::Invalid);}
                        self.policy.as_mut().unwrap().abort(current,frame.rdi,frame.rdx)?;
                        self.handover=None;self.desktop=None;self.synchronize_revocations();Ok(0)
                    },
                    5=>{
                        self.buffer(frame.r10,32,true)?;
                        let policy=self.policy.as_ref().unwrap();
                        let t=policy.trial().ok_or(Error::Stale)?;
                        if t.image_seal()!=frame.rdx{return Err(Error::Stale);}
                        let phase=match policy.state(t.endpoint().slot as usize)?{
                            model::State::Trial=>1,model::State::Healthy=>2,_=>return Err(Error::Stale)
                        };
                        let mut response=[0u8;32];
                        for (i,value) in [t.image_seal(),t.token(),t.endpoint().incarnation,phase]
                            .into_iter().enumerate(){
                            response[i*8..i*8+8].copy_from_slice(&value.to_le_bytes());
                        }
                        // SAFETY: full current manager output checked, IF=0.
                        unsafe{ptr::copy_nonoverlapping(response.as_ptr(),frame.r10 as *mut u8,32);}
                        Ok(0)
                    },
                    6=>{
                        if frame.r10==0{return Err(Error::Invalid);}
                        if self.policy.as_ref().unwrap().bootstrapping(){
                            self.prepare_boot_desktop(frame.rdi,frame.rdx,frame.r10)?;
                        }else{self.prepare_handover(frame.rdi,frame.rdx,frame.r10)?;}
                        Ok(0)
                    },
                    7=>{
                        if frame.r10==0{return Err(Error::Invalid);}
                        if self.policy.as_ref().unwrap().bootstrapping(){
                            self.commit_boot_desktop(frame.rdi,frame.rdx,frame.r10);
                        }else{self.commit_handover(frame.rdi,frame.rdx,frame.r10);}
                        Ok(0)
                    },
                    8=>{
                        if frame.rdx!=0||frame.r10!=0{return Err(Error::Invalid);}
                        fatal("RAR-PANIC:CODE=UPDATE-RECONCILE");
                    },
                    _=>Err(Error::Invalid),
                }
            }
            abi::TRIAL_READY=>{
                if frame.rdx!=0||frame.r10!=0{return Err(Error::Invalid);}
                self.policy.as_mut().ok_or(Error::Denied)?.ready(current,frame.rdi,frame.rsi)?;
                self.processes[current].state=State::Blocked;
                Ok(0)
            }
            _=>Err(Error::Invalid),
        }
    }

    /// Sole-CPU trap context shared with deferred physical retirement.
    fn stage_context(&self)->u64{
        let root:u64;let cr4:u64;let flags:u64;
        // SAFETY: privileged reads only inside the certified guest trap.
        unsafe{
            core::arch::asm!("mov {},cr3",out(reg)root,options(nostack,preserves_flags));
            core::arch::asm!("mov {},cr4",out(reg)cr4,options(nostack,preserves_flags));
            core::arch::asm!("pushfq","pop {}",out(reg)flags);
        }
        retirement::context(root,private_region(self.arena,self.current),cr4,flags)
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-CONTEXT"));
        root
    }
    /// Preflight all bootstrap/live roots before changing any alias. All roots
    /// use the sole kernel-created identity mapping; no DMA/user writer exists.
    /// IF=0 and PGE/PCIDE off make the final CR3 reload plus later root switches
    /// sufficient to retire every cached writable translation on the sole CPU.
    fn stage_permissions(&mut self,was_writable:bool){
        let root=self.stage_context();
        if self.stage_readonly==was_writable||self.stage_view{
            fatal("RAR-PANIC:CODE=STAGE-ALIAS-ORDER");
        }
        let region=staging::region(self.arena,boot::ARENA_PAGES).unwrap();
        let mut roots=[(0u64,0usize);TASKS+1];
        roots[0]=(self.arena,self.bootstrap_tables);
        for (i,p) in self.processes.iter().enumerate(){roots[i+1]=(p.root,p.table_used);}
        for &(base,used) in &roots{
            if base==0{continue;}
            // SAFETY: exclusively owned arena page-table pools, IF=0. This
            // first pass performs no allocation or mutation in any address space.
            unsafe{Tables::resume(base,used).check_modern_staging(region.start,was_writable)}
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-ALIAS-PREFLIGHT"));
        }
        for &(base,used) in &roots{
            if base==0{continue;}
            // SAFETY: identical full-root preflight above, no scheduling or
            // intervening writer. Any impossible failure halts before exposure.
            unsafe{Tables::resume(base,used).set_modern_staging(region.start,was_writable)}
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-ALIAS-CHANGE"));
        }
        // SAFETY: same verified current root; invalidates nonglobal translations.
        unsafe{core::arch::asm!("mov cr3,{}",in(reg)root,options(nostack,preserves_flags));}
        self.stage_readonly=was_writable;
    }
    fn seal_stage(&mut self){
        if self.current!=9||self.stage_readonly||self.stage_view||
            self.policy.as_ref().unwrap().state(8)!=Ok(model::State::Active)||
            self.processes[8].memory!=retirement::Memory::Live{
            fatal("RAR-PANIC:CODE=STAGE-MANAGER");
        }
        let region=staging::region(self.arena,boot::ARENA_PAGES).unwrap();
        let manager=self.processes[8];
        if manager.range_count>=manager.ranges.len(){fatal("RAR-PANIC:CODE=STAGE-RANGE");}
        // SAFETY: inactive manager's fixed paths were preallocated before boot.
        unsafe{Tables::resume(manager.root,manager.table_used).check_modern_verifier(region.start,false)}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-VIEW-PREFLIGHT"));
        self.stage_permissions(true);
        // SAFETY: ALL writable aliases have been removed, TLBs invalidated.
        // Manager's only new mapping is fixed RO/NX with absent guards.
        unsafe{Tables::resume(manager.root,manager.table_used).publish_modern_verifier(region.start)}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-VIEW-PUBLISH"));
        self.processes[8].range(abi::STAGE_VIEW_ADDRESS,
            abi::STAGE_VIEW_ADDRESS+staging::BUFFER_BYTES as u64,false,false);
        self.stage_view=true;
    }
    fn withdraw_stage_view(&mut self){
        let root=self.stage_context();
        if self.current!=8||!self.stage_readonly||!self.stage_view{
            fatal("RAR-PANIC:CODE=STAGE-REJECT-ORDER");
        }
        let region=staging::region(self.arena,boot::ARENA_PAGES).unwrap();
        let manager=self.processes[8];
        // SAFETY: validated current manager root, sole reader, no trial/staged
        // executable exists. Remove all user aliases before enabling any writer.
        unsafe{Tables::resume(manager.root,manager.table_used).retire_modern_verifier(region.start)}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-VIEW-RETIRE"));
        unsafe{core::arch::asm!("mov cr3,{}",in(reg)root,options(nostack,preserves_flags));}
        let p=&mut self.processes[8];
        if p.range_count==0||p.ranges[p.range_count-1].start!=abi::STAGE_VIEW_ADDRESS{
            fatal("RAR-PANIC:CODE=STAGE-RANGE-RETIRE");
        }
        p.range_count-=1;p.ranges[p.range_count]=EMPTY_RANGE;self.stage_view=false;
    }
    fn reject_stage(&mut self,seal:u64){
        self.withdraw_stage_view();self.stage_permissions(false);self.scrub_stage(seal);
    }
    fn release_trial_stage(&mut self,handle:u64,seal:u64)->Result<(),Error>{
        self.policy.as_ref().ok_or(Error::Denied)?.stage_reject(self.current,handle)?;
        let stage=self.staging.as_ref().ok_or(Error::Denied)?;
        stage.view(seal).map_err(stage_error)?;
        let id=stage.reserved().ok_or(Error::Stale)?;
        if self.stage_view||!self.stage_readonly||
            self.processes[id.slot()].memory==retirement::Memory::Retiring{return Err(Error::Busy);}
        self.stage_permissions(false);self.scrub_stage(seal);Ok(())
    }
    fn accept_stage(&mut self,handle:u64,seal:u64)->Result<model::Trial,Error>{
        self.policy.as_ref().ok_or(Error::Denied)?.stage_view(self.current,handle)?;
        if self.current!=8||!self.stage_readonly||!self.stage_view{return Err(Error::Busy);}
        let stage=self.staging.as_ref().ok_or(Error::Denied)?;
        let bytes=stage.view(seal).map_err(stage_error)?;
        let id=stage.reserved().ok_or(Error::Stale)?;
        let metadata=support::stage_metadata(bytes)?;
        let layout=pe::parse(&bytes[384..])?;
        if layout.image_size>metadata.image_bytes{return Err(Error::Invalid);}
        let p=self.processes[id.slot()];
        if p.memory!=retirement::Memory::Clean||p.state!=State::Dead||p.root!=0{
            return Err(Error::Busy);
        }
        let policy=self.policy.as_mut().unwrap();
        policy.authenticated_stage(self.current,handle,seal,id.slot(),metadata.digest,
            metadata.generation,metadata.budget)?;
        let trial=match policy.begin_trial(self.current,handle,seal){
            Ok(t)=>t,Err(e)=>{policy.discard_staged(self.current,handle,seal)?;return Err(e);}
        };
        self.withdraw_stage_view();
        // Move the sole buffer owner out temporarily: the immutable payload and
        // mutable Runtime construction borrow are disjoint. IF=0; no reentry.
        let stage=self.staging.take().unwrap();
        let bytes=stage.view(seal).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=TRIAL-SEALED-BYTES"));
        let result=self.construct_trial(&bytes[384..],trial);
        self.staging=Some(stage);
        if let Err(e)=result{
            self.policy.as_mut().unwrap().abort(self.current,handle,trial.token())
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=TRIAL-CONSTRUCT-ABORT"));
            self.synchronize_revocations();return Err(e);
        }
        Ok(trial)
    }
    fn scrub_stage(&mut self,seal:u64){
        self.stage_context();
        if self.stage_readonly||self.stage_view{fatal("RAR-PANIC:CODE=STAGE-SCRUB-ORDER");}
        self.staging.as_mut().unwrap().clear_with(seal,|owned|{
            // SAFETY: pointers derive from the exclusive buffer borrow. The
            // caller removed every user view and restored supervisor writes,
            // flushing translations first. Full non-elidable scrub/readback;
            // no allocator, arbitrary address or owner data is involved.
            unsafe{
                let bytes=owned.as_mut_ptr().cast::<u64>();
                for i in 0..owned.len()/8{bytes.add(i).write_volatile(0);}
                for i in 0..owned.len()/8{
                    if bytes.add(i).read_volatile()!=0{fatal("RAR-PANIC:CODE=STAGE-SCRUB");}
                }
            }
        }).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=STAGE-CLEAR"));
    }

    fn kill(&mut self,index:usize){
        if self.processes[index].state==State::Dead{return;}
        if index!=15{
            let endpoint=model::Endpoint{slot:index as u8,incarnation:self.processes[index].generation};
            self.policy.as_mut().unwrap().fault(endpoint)
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=MODERN-FAULT-IDENTITY"));
        }
        self.processes[index].state=State::Dead;
        self.processes[index].memory=retirement::retire(self.processes[index].memory);
        self.synchronize_revocations();
    }
    /// Invoked only on a validated ring3 trap in a surviving context. The last
    /// trap may have switched CR3 while still unwinding on an outgoing stack;
    /// therefore never erase current here or call this after selecting a root.
    fn retire_pending(&mut self){
        let current=self.current;
        let owner=self.processes[current];
        let expected=private_region(self.arena,current);
        if owner.memory!=retirement::Memory::Live||owner.root!=expected||
            owner.kernel_bottom!=expected+KERNEL_BOTTOM||owner.kernel_top!=expected+KERNEL_TOP||
            !(1..=256).contains(&owner.table_used){
            fatal("RAR-PANIC:CODE=RETIRE-CURRENT");
        }
        let root:u64;let cr4:u64;let flags:u64;
        // SAFETY: privileged current-CPU reads, sole certified CPU, kernel trap.
        unsafe{
            core::arch::asm!("mov {},cr3",out(reg)root,options(nostack,preserves_flags));
            core::arch::asm!("mov {},cr4",out(reg)cr4,options(nostack,preserves_flags));
            core::arch::asm!("pushfq","pop {}",out(reg)flags);
        }
        for index in 0..TASKS{
            if index==current||self.processes[index].memory!=retirement::Memory::Retiring{continue;}
            let victim=self.processes[index];
            if victim.state!=State::Dead||
                (index!=15&&self.policy.as_ref().unwrap().state(index)!=Ok(model::State::Vacant)){
                fatal("RAR-PANIC:CODE=RETIRE-AUTHORITY");
            }
            let plan=retirement::plan(self.arena,boot::ARENA_PAGES,current,index,victim.memory,
                victim.root,victim.kernel_bottom,victim.kernel_top,self.processes[current].aperture)
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=RETIRE-GEOMETRY"));
            retirement::context(root,plan.current_root,cr4,flags)
                .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=RETIRE-CONTEXT"));
            // SAFETY: geometry binds the PTE page to current's preallocated
            // private table pool and the victim to another exact owned stride.
            // No user or executable alias exists outside the victim root.
            // PGE/PCIDE are off, so activating this survivor flushed all former
            // user translations. IF=0 prevents scheduling during the operation.
            // The empty aperture is supervisor-only RW/NX, never exported.
            unsafe{
                let mut tables=Tables::resume(owner.root,owner.table_used);
                if tables.modern_aperture()!=Ok(plan.leaf){
                    fatal("RAR-PANIC:CODE=RETIRE-TABLE-PATH");
                }
                let leaf=plan.leaf as *mut u64;
                for i in 0..retirement::APERTURE_PAGES{
                    if leaf.add(i).read_volatile()!=0{fatal("RAR-PANIC:CODE=RETIRE-ALIAS");}
                }
                // Inactive victim root is supervisor identity-mapped here.
                // Destroy every user/executable mapping before writable scrub.
                for i in 0..512{(plan.victim as *mut u64).add(i).write_volatile(0);}
                for i in 0..512{
                    if (plan.victim as *const u64).add(i).read_volatile()!=0{
                        fatal("RAR-PANIC:CODE=RETIRE-ROOT");
                    }
                }
                for i in 0..retirement::APERTURE_PAGES{
                    leaf.add(i).write_volatile((plan.victim+i as u64*4096)|3|(1<<63));
                }
                for i in 0..retirement::APERTURE_PAGES{
                    let address=retirement::APERTURE+i as u64*4096;
                    core::arch::asm!("invlpg [{}]",in(reg)address,options(nostack,preserves_flags));
                }
                let bytes=retirement::APERTURE as *mut u64;
                for i in 0..(STRIDE/8) as usize{bytes.add(i).write_volatile(0);}
                for i in 0..(STRIDE/8) as usize{
                    if bytes.add(i).read_volatile()!=0{fatal("RAR-PANIC:CODE=RETIRE-ZERO");}
                }
                for i in 0..retirement::APERTURE_PAGES{leaf.add(i).write_volatile(0);}
                for i in 0..retirement::APERTURE_PAGES{
                    let address=retirement::APERTURE+i as u64*4096;
                    core::arch::asm!("invlpg [{}]",in(reg)address,options(nostack,preserves_flags));
                }
                if tables.modern_aperture()!=Ok(plan.leaf){
                    fatal("RAR-PANIC:CODE=RETIRE-ALIAS-REMOVAL");
                }
                for i in 0..retirement::APERTURE_PAGES{
                    if leaf.add(i).read_volatile()!=0{fatal("RAR-PANIC:CODE=RETIRE-ALIAS-REMOVAL");}
                }
            }
            // No fallible work after this clean publication; a logical vacancy
            // alone never authorizes future construction in a dirty stride.
            self.processes[index]=Process::EMPTY;
            record("RAR-MODERN:PRIVATE-MEMORY-RETIRED");
        }
    }
    fn synchronize_revocations(&mut self){
        if self.desktop.as_ref().is_some_and(|d|{
            let policy=self.policy.as_ref().unwrap();
            policy.trial().is_none_or(|t|t.token()!=d.token||t.image_seal()!=d.seal)||
                policy.state(8)!=Ok(model::State::Active)||policy.state(9)!=Ok(model::State::Active)
        }){self.desktop=None;}
        if self.handover.is_some_and(|(h,_,_)|self.policy.as_ref().unwrap().trial().is_none_or(|t|t.token()!=h.token())){self.handover=None;}
        for i in 0..TASKS{
            let prepared=self.desktop.is_some()&&[0usize,1,2,3,4,6].contains(&i);
            if i!=15&&!prepared&&self.policy.as_ref().unwrap().state(i)==Ok(model::State::Vacant){
                self.processes[i].state=State::Dead;
                self.processes[i].memory=retirement::retire(self.processes[i].memory);
            }
        }
    }
}
fn user_return_valid(process:&Process,ret:&arch::Trap)->bool{
    support::user_return(&process.ranges[..process.range_count],process.stack_end,
        ret.rsp,ret.rip,ret.cs,ret.ss,[ret.ds,ret.es,ret.fs,ret.gs])
}

/// Called only by the assembly trap gate. Kernel faults are fatal. User faults
/// destroy only their process authority; unrelated services remain scheduled.
pub extern "sysv64" fn trap(frame:*mut arch::Trap,saved:u64)->u64{
    let state=unsafe{&mut *ptr::addr_of_mut!(RUNTIME)};
    let current=state.current;
    if current>=TASKS{fatal("RAR-PANIC:CODE=KERNEL-TRAP");}
    let process=&state.processes[current];
    if saved%16!=0||saved<process.kernel_bottom||
        saved.checked_add(720).is_none_or(|end|end>process.kernel_top)||
        saved.checked_add(512)!=Some(frame as u64){
        fatal("RAR-PANIC:CODE=TRAP-STACK");
    }
    // SAFETY: saved/frame were range/alignment checked before dereference.
    let f=unsafe{&mut *frame};
    if f.cs&3!=3||matches!(f.vector,2|8){fatal("RAR-PANIC:CODE=KERNEL-TRAP");}
    state.retire_pending();
    state.processes[current].frame=saved;
    match f.vector{
        32=>{
            unsafe{crate::out(0x20,0x20);}
            state.processes[current].preemptions=state.processes[current].preemptions.saturating_add(1);
            state.ticks=support::tick(state.ticks);
            // IRQ0 owns current/generation under IF=0. Never take a trial token
            // or budget from registers. Idle has no logical endpoint.
            if current!=15{
                let endpoint=model::Endpoint{slot:current as u8,
                    incarnation:state.processes[current].generation};
                let expired=state.policy.as_mut().unwrap().delivered_preemption(endpoint)
                    .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=MODERN-TIMER-IDENTITY"));
                if expired{
                    // Reconcile logical death before return validation or
                    // runnable selection. Physical teardown/reuse is a separate
                    // mandatory boundary deferred to the next survivor trap.
                    state.synchronize_revocations();
                }
            }
        }
        128=>{
            f.rax=match state.sys(f){Ok(value)=>value,Err(error)=>number(error)};
        }
        0..=31=>{
            if current==6&&f.vector==6&&f.error==0{
                record("RAR-MODERN:APP-FAULT=6");
            }else{record("RAR-MODERN:UNEXPECTED-USER-FAULT");}
            state.kill(current);
        }
        _=>fatal("RAR-PANIC:CODE=TRAP-VECTOR"),
    }
    // Classify the current invalid user frame immediately, before another
    // service can attempt a response to its now-dead endpoint.
    if state.processes[current].state!=State::Dead&&!user_return_valid(&state.processes[current],f){
        record("RAR-MODERN:INVALID-USER-RETURN");
        state.kill(current);
    }
    // Invalid user return registers are user faults, never a kernel-wide panic.
    // Kernel-owned frame/root/stack corruption remains a fatal invariant failure.
    let mut cursor=current;
    for _ in 0..TASKS{
        let states=core::array::from_fn(|index|state.processes[index].state);
        let next=support::next(&states,cursor).unwrap_or_else(|_|fatal("RAR-PANIC:CODE=NO-RUNNABLE"));
        let process=&state.processes[next];
        let expected=private_region(state.arena,next);
        if process.memory!=retirement::Memory::Live||process.root!=expected||process.kernel_bottom!=expected+KERNEL_BOTTOM||
            process.kernel_top!=expected+KERNEL_TOP||process.frame%16!=0||
            process.frame<process.kernel_bottom||process.frame.checked_add(720).is_none_or(|end|end>process.kernel_top){
            fatal("RAR-PANIC:CODE=OWNED-RETURN-FRAME");
        }
        let ret=unsafe{&mut *((process.frame+512)as *mut arch::Trap)};
        let valid=user_return_valid(process,ret);
        if !valid{
            record("RAR-MODERN:INVALID-USER-RETURN");
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
