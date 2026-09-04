//! UEFI 2.10 adapter, implemented from the standard without a boot library.
use crate::{fatal,model::{Region,Mapping,MAX_REGIONS},paging::Tables};
use core::{arch::asm,mem,ptr};
pub const MAGIC:u64=0x5241525f424f4f31;
pub const ARENA_PAGES:usize=1024;
pub const STACK_GUARD:u64=0x100000;
pub const STACK_TOP:u64=0x121000;
pub const HEAP_OFFSET:u64=0x140000;
pub const EMERGENCY_TOP:u64=0x171000;
#[repr(C)]
pub struct Header {signature:u64,revision:u32,size:u32,crc:u32,reserved:u32}
#[repr(C)]
pub struct SystemTable {
    header:Header,vendor:usize,revision:u32,conin_handle:usize,conin:usize,
    conout_handle:usize,conout:usize,stderr_handle:usize,stderr:usize,
    runtime:usize,boot:*const BootServices,entries:usize,config:usize,
}
#[repr(C)]
struct BootServices {header:Header, functions:[usize;44]}
#[repr(C)]
struct Guid {a:u32,b:u16,c:u16,d:[u8;8]}
const LOADED_IMAGE:Guid=Guid{a:0x5b1b31a1,b:0x9562,c:0x11d2,d:[0x8e,0x3f,0,0xa0,0xc9,0x69,0x72,0x3b]};
#[repr(C)]
struct LoadedImage {
    revision:u32,parent:usize,system:usize,device:usize,path:usize,reserved:usize,
    options_size:u32,options:usize,base:u64,size:u64,code_type:u32,data_type:u32,unload:usize,
}
#[repr(C)]
pub struct BootInfo {
    pub magic:u64,pub arena:u64,pub table_used:usize,pub count:usize,
    pub regions:[Region;MAX_REGIONS],
}
static mut INFO:BootInfo=BootInfo{magic:MAGIC,arena:0,table_used:0,count:0,
    regions:[Region{start:0,pages:0,kind:0};MAX_REGIONS]};
#[repr(C,align(8))]
struct MapBuffer([u8;65536]);
static mut MAP:MapBuffer=MapBuffer([0;65536]);

/// Firmware supplies readable table storage. We check bounded structural shape,
/// revision/signature/CRC before using function pointers; malicious firmware is
/// outside the certified profile's software trust boundary.
unsafe fn header(pointer:*const Header,signature:u64,minimum:usize) {
    let address=pointer as u64;
    if address==0 || address%8!=0 || address>0xffff_f000 {fatal("RAR-PANIC:CODE=UEFI-POINTER");}
    let size=unsafe {(*pointer).size} as usize;
    if size<minimum || size>4096 || address+size as u64>0x1_0000_0000 {fatal("RAR-PANIC:CODE=UEFI-SIZE");}
    let bytes=unsafe {core::slice::from_raw_parts(pointer.cast::<u8>(),size)};
    if crate::model::validate_table(bytes,signature,minimum).is_err() {fatal("RAR-PANIC:CODE=UEFI-HEADER");}
}
fn value32(bytes:&[u8],offset:usize)->u32 {
    u32::from_le_bytes(bytes.get(offset..offset+4).unwrap_or_else(||fatal("RAR-PANIC:CODE=PE-BOUNDS")).try_into().unwrap())
}
unsafe fn map_image(tables:&mut Tables,base:u64,size:u64) {
    if base==0 || base%4096!=0 || size<4096 || size>8*1024*1024 ||
        base.checked_add(size).is_none_or(|e|e>0x1_0000_0000) {fatal("RAR-PANIC:CODE=IMAGE-RANGE");}
    let bytes=unsafe {core::slice::from_raw_parts(base as *const u8,size as usize)};
    if &bytes[..2]!=b"MZ" {fatal("RAR-PANIC:CODE=IMAGE-DOS");}
    let pe=value32(bytes,60) as usize;
    if pe>4096 || bytes.get(pe..pe+4)!=Some(b"PE\0\0") {fatal("RAR-PANIC:CODE=IMAGE-PE");}
    let sections=u16::from_le_bytes(bytes[pe+6..pe+8].try_into().unwrap()) as usize;
    let optional=u16::from_le_bytes(bytes[pe+20..pe+22].try_into().unwrap()) as usize;
    if sections==0 || sections>32 || optional<112 {fatal("RAR-PANIC:CODE=IMAGE-SECTIONS");}
    let header_pages=(value32(bytes,pe+24+60) as u64).div_ceil(4096);
    let end=base+size.div_ceil(4096)*4096;
    unsafe {tables.map(Mapping{virtual_start:base,physical_start:base,pages:header_pages,writable:false,executable:false},base,end)}
        .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=IMAGE-HEADER-MAP"));
    for i in 0..sections {
        let s=pe+24+optional+i*40;
        let length=value32(bytes,s+8) as u64;
        let offset=value32(bytes,s+12) as u64;
        let flags=value32(bytes,s+36);
        if length==0 {continue;}
        if offset%4096!=0 || offset+length>size {fatal("RAR-PANIC:CODE=IMAGE-SECTION-RANGE");}
        unsafe {tables.map(Mapping{virtual_start:base+offset,physical_start:base+offset,
            pages:length.div_ceil(4096),writable:flags&0x8000_0000!=0,executable:flags&0x2000_0000!=0},base,end)}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=IMAGE-WX"));
    }
}
pub unsafe fn start(image:usize,system:*const SystemTable)->! {
    unsafe {header(system.cast(),0x5453595320494249,mem::size_of::<SystemTable>());}
    let services=unsafe {(*system).boot};
    unsafe {header(services.cast(),0x56524553544f4f42,mem::size_of::<BootServices>());}
    let f=unsafe {&(*services).functions};
    for &i in &[2,4,16,26] {
        if f[i]==0 || f[i] as u64>=0x1_0000_0000 {fatal("RAR-PANIC:CODE=UEFI-SERVICE");}
    }
    type Allocate=unsafe extern "efiapi" fn(u32,u32,usize,*mut u64)->usize;
    type GetMap=unsafe extern "efiapi" fn(*mut usize,*mut u8,*mut usize,*mut usize,*mut u32)->usize;
    type Handle=unsafe extern "efiapi" fn(usize,*const Guid,*mut *const LoadedImage)->usize;
    type Exit=unsafe extern "efiapi" fn(usize,usize)->usize;
    let allocate:Allocate=unsafe {mem::transmute(f[2])};
    let get_map:GetMap=unsafe {mem::transmute(f[4])};
    let handle:Handle=unsafe {mem::transmute(f[16])};
    let exit:Exit=unsafe {mem::transmute(f[26])};
    let mut loaded=ptr::null();
    if unsafe {handle(image,&LOADED_IMAGE,&mut loaded)}!=0 || loaded.is_null() {
        fatal("RAR-PANIC:CODE=LOADED-IMAGE");
    }
    let (base,size)=unsafe {((*loaded).base,(*loaded).size)};
    let mut arena=0xffff_ffff;
    if unsafe {allocate(1,2,ARENA_PAGES,&mut arena)}!=0 || arena%4096!=0 {
        fatal("RAR-PANIC:CODE=ARENA");
    }
    unsafe {ptr::write_bytes(arena as *mut u8,0,ARENA_PAGES*4096);}
    let mut tables=unsafe {Tables::new(arena)};
    // Only explicit arena regions are mapped. Kernel/emergency guard pages stay absent.
    for page in 0..ARENA_PAGES {
        let offset=page as u64*4096;
        if [STACK_GUARD,STACK_TOP,0x160000,EMERGENCY_TOP].contains(&offset) {continue;}
        unsafe {tables.map(Mapping{virtual_start:arena+offset,physical_start:arena+offset,
            pages:1,writable:true,executable:false},arena,arena+ARENA_PAGES as u64*4096)}
            .unwrap_or_else(|_|fatal("RAR-PANIC:CODE=ARENA-MAP"));
    }
    unsafe {map_image(&mut tables,base,size);}
    let info=ptr::addr_of_mut!(INFO);
    unsafe {(*info).arena=arena;(*info).table_used=tables.used();}
    let buffer=ptr::addr_of_mut!(MAP).cast::<u8>();
    let mut exited=false;
    for _ in 0..3 {
        let mut length=65536;let mut key=0;let mut stride=0;let mut version=0;
        if unsafe {get_map(&mut length,buffer,&mut key,&mut stride,&mut version)}!=0 ||
            !(40..=256).contains(&stride) || length>65536 || length%stride!=0 ||
            length/stride>MAX_REGIONS || version!=1 {fatal("RAR-PANIC:CODE=UEFI-MAP");}
        unsafe {(*info).count=length/stride;}
        for i in 0..length/stride {
            let descriptor=unsafe {buffer.add(i*stride)};
            let kind=unsafe {ptr::read_unaligned(descriptor.cast::<u32>())};
            let start=unsafe {ptr::read_unaligned(descriptor.add(8).cast::<u64>())};
            let pages=unsafe {ptr::read_unaligned(descriptor.add(24).cast::<u64>())};
            unsafe {(*info).regions[i]=Region{start,pages,kind};}
        }
        let regions=unsafe {core::slice::from_raw_parts(ptr::addr_of!((*info).regions).cast::<Region>(),(*info).count)};
        if crate::model::validate_regions(regions).is_err() {fatal("RAR-PANIC:CODE=MEMORY-MAP");}
        if unsafe {exit(image,key)}==0 {exited=true;break;}
        // UEFI allows retrying GetMemoryMap/ExitBootServices after a stale key.
    }
    if !exited {fatal("RAR-PANIC:CODE=EXIT-BOOT");}
    unsafe {
        asm!("cli",options(nomem,nostack));
        // NX is required by the fixed qemu64 profile. Set EFER.NXE and CR0.WP
        // before activating RAR's W^X tables; no firmware call occurs afterward.
        let lo:u32;let hi:u32;
        asm!("rdmsr",in("ecx")0xc0000080u32,out("eax")lo,out("edx")hi,options(nostack));
        asm!("wrmsr",in("ecx")0xc0000080u32,in("eax")(lo|1<<11),in("edx")hi,options(nostack));
        // Remove inherited global translations before switching to private tables.
        let cr4:u64;asm!("mov {}, cr4",out(reg)cr4,options(nostack));
        asm!("mov cr4, {}",in(reg)(cr4&!(1<<7)),options(nostack));
        let cr0:u64;asm!("mov {}, cr0",out(reg)cr0,options(nostack));
        asm!("mov cr0, {}",in(reg)(cr0|1<<16),options(nostack));
        crate::enter(tables.root(),arena+STACK_TOP,info)
    }
}
