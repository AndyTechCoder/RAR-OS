//! Owned four-level x86_64 tables. Only explicit 4 KiB mappings are installed.
use crate::model::{Error,Mapping};
use core::arch::asm;
const ADDRESS:u64=0x000f_ffff_ffff_f000;
const NX:u64=1<<63;
pub struct Tables {base:u64,used:usize}
impl Tables {
    /// Caller owns the zeroed, 1 MiB page-table arena and firmware maps it.
    pub unsafe fn new(base:u64)->Self {Self{base,used:1}}
    /// Caller retains exclusive ownership and identity mapping of this arena.
    pub unsafe fn resume(base:u64,used:usize)->Self {Self{base,used}}
    pub fn root(&self)->u64 {self.base}
    pub fn used(&self)->usize {self.used}
    fn allocate(&mut self)->Result<u64,Error> {
        if self.used>=256 {return Err(Error::Exhausted);}
        let page=self.base+self.used as u64*4096;self.used+=1;
        // SAFETY: page is a fresh, exclusively owned mapped arena page.
        unsafe {core::ptr::write_bytes(page as *mut u8,0,4096);}
        Ok(page)
    }
    unsafe fn leaf(&mut self,address:u64,create:bool)->Result<*mut u64,Error> {
        let mut table=self.base;
        for shift in [39,30,21] {
            let p=(table as *mut u64).wrapping_add(((address>>shift)&511) as usize);
            let mut e=unsafe {p.read()};
            if e&1==0 {
                if !create {return Err(Error::Invalid);}
                e=self.allocate()?|3; unsafe {p.write(e);}
            }
            if e&0x80!=0 {return Err(Error::Invalid);}
            let next=e&ADDRESS;
            if next<self.base || next>=self.base+self.used as u64*4096 {return Err(Error::Permission);}
            table=next;
        }
        Ok((table as *mut u64).wrapping_add(((address>>12)&511) as usize))
    }
    /// Caller grants ownership of the specified physical interval. This private
    /// bootstrap interface is not an application mapping capability.
    pub unsafe fn map(&mut self,m:Mapping,start:u64,end:u64)->Result<(),Error> {
        m.validate(start,end)?;
        // Preflight prevents partial mapping on overlap. Arena exhaustion is a
        // fatal bootstrap error; callers never resume a failed address space.
        for i in 0..m.pages {
            let p=unsafe {self.leaf(m.virtual_start+i*4096,true)?};
            if unsafe {p.read()}&1!=0 {return Err(Error::Overlap);}
        }
        for i in 0..m.pages {
            let v=m.virtual_start+i*4096;
            let p=unsafe {self.leaf(v,false)?};
            let flags=1|if m.writable {2}else{0}|if m.executable {0}else{NX};
            unsafe {p.write((m.physical_start+i*4096)|flags);}
        }
        Ok(())
    }
    /// Caller has stopped all users of this mapping; this profile is uniprocessor.
    pub unsafe fn unmap(&mut self,address:u64)->Result<(),Error> {
        crate::model::validate_virtual_page(address)?;
        let p=unsafe {self.leaf(address,false)?};
        if unsafe {p.read()}&1==0 {return Err(Error::Invalid);}
        unsafe {p.write(0);asm!("invlpg [{}]",in(reg)address,options(nostack,preserves_flags));}
        Ok(())
    }

    /// Reserve the Modern-only 2 MiB supervisor scratch aperture while inactive.
    /// Caller owns this zeroed 1 MiB pool; no user may run until construction
    /// finishes. No leaf becomes present and no TLB operation occurs here.
    /// The returned page stays kernel-private; only the current root's owner
    /// may fill it with supervisor RW/NX entries during deferred retirement.
    #[cfg(rar_modern)]
    pub unsafe fn reserve_modern_aperture(&mut self)->Result<u64,Error>{
        unsafe{self.leaf(0x1000000,true)?;}
        unsafe{self.modern_aperture()}
    }
    /// Validate the already reserved empty aperture without allocating pages.
    /// Caller exclusively owns the live current table pool with IF=0.
    #[cfg(rar_modern)]
    pub unsafe fn modern_aperture(&mut self)->Result<u64,Error>{
        if !(1..=256).contains(&self.used){return Err(Error::Invalid);}
        let first=unsafe{self.leaf(0x1000000,false)?};
        if (first as u64)%4096!=0{return Err(Error::Invalid);}
        for i in 0..512{
            if unsafe{first.add(i).read()}!=0{return Err(Error::Overlap);}
        }
        Ok(first as u64)
    }


    /// Modern fixed manager verifier window, including two nonpresent guards.
    /// Inactive, exclusively owned tables only; reserves paths but grants no
    /// user access and performs no privileged instruction.
    #[cfg(rar_modern)]
    pub unsafe fn reserve_modern_verifier(&mut self)->Result<(),Error>{
        for i in 0..515 {
            let address=0x13ff000+i*4096;
            let p=unsafe{self.leaf(address,true)?};
            if unsafe{p.read()}!=0{return Err(Error::Overlap);}
        }
        Ok(())
    }
    /// Validate all513 supervisor identity aliases before an all-root
    /// permission transition. A/D bits may be set by the processor; all other
    /// flags, identity and NX permissions must match exactly.
    /// Caller owns the entire mapped table pool and the fixed staging region.
    #[cfg(rar_modern)]
    pub unsafe fn check_modern_staging(&mut self,start:u64,writable:bool)->Result<(),Error>{
        if !(1..=256).contains(&self.used)||self.base%4096!=0||
            start%4096!=0||start<0x4401000||start.checked_add(513*4096).is_none_or(|e|e>0x1_0000_0000){
            return Err(Error::Invalid);
        }
        for i in 0..513{
            let address=start+i*4096;let p=unsafe{self.leaf(address,false)?};
            let expected=address|1|NX|if writable{2}else{0};
            if unsafe{p.read()}&!0x60!=expected{return Err(Error::Permission);}
        }
        Ok(())
    }
    /// Page-table byte mutation only. Caller preflights EVERY root, holds IF=0
    /// on the sole CPU, and flushes translations before publishing a user view
    /// or writing bytes. No allocation and no executable/user alias is created.
    #[cfg(rar_modern)]
    pub unsafe fn set_modern_staging(&mut self,start:u64,was_writable:bool)->Result<(),Error>{
        unsafe{self.check_modern_staging(start,was_writable)?;}
        for i in 0..513{
            let p=unsafe{self.leaf(start+i*4096,false)?};
            let e=unsafe{p.read()};
            unsafe{p.write_volatile(if was_writable{e&!2}else{e|2});}
        }
        Ok(())
    }
    /// Caller has reserved the fixed window. This checks exact present RO/NX
    /// user leaves, or strict all-zero absence, plus both guards. No allocation.
    #[cfg(rar_modern)]
    pub unsafe fn check_modern_verifier(&mut self,physical:u64,present:bool)->Result<(),Error>{
        if !(1..=256).contains(&self.used)||self.base%4096!=0||physical%4096!=0||
            physical<0x4401000||physical.checked_add(513*4096).is_none_or(|e|e>0x1_0000_0000){
            return Err(Error::Invalid);
        }
        for i in 0..515{
            let p=unsafe{self.leaf(0x13ff000+i*4096,false)?};
            let e=unsafe{p.read()};
            if i==0||i==514||!present{
                if e!=0{return Err(Error::Overlap);}
            }else if e&!0x60!=((physical+(i-1)*4096)|5|NX){return Err(Error::Permission);}
        }
        Ok(())
    }
    /// Only after every writable alias has been revoked and TLB invalidation
    /// completed. The manager is inactive and the sole permitted user reader.
    #[cfg(rar_modern)]
    pub unsafe fn publish_modern_verifier(&mut self,physical:u64)->Result<(),Error>{
        unsafe{self.check_modern_verifier(physical,false)?;}
        unsafe{self.map_user(Mapping{virtual_start:0x1400000,physical_start:physical,
            pages:513,writable:false,executable:false},physical,physical+513*4096,false)?;}
        unsafe{self.check_modern_verifier(physical,true)}
    }
    /// Removes the user view before any writable alias or buffer reuse. Caller
    /// must flush translations before enabling writes. No privileged instruction
    /// occurs here so exact table transitions can be source-tested in cloud.
    #[cfg(rar_modern)]
    pub unsafe fn retire_modern_verifier(&mut self,physical:u64)->Result<(),Error>{
        unsafe{self.check_modern_verifier(physical,true)?;}
        for i in 0..513{
            let p=unsafe{self.leaf(0x1400000+i*4096,false)?};
            unsafe{p.write_volatile(0);}
        }
        unsafe{self.check_modern_verifier(physical,false)}
    }

    /// Platform-only user mapping, built while this address space is inactive.
    /// The caller owns the physical interval and has removed writable aliases of
    /// executable pages. Intermediate U/S promotion never changes kernel leaves.
    #[cfg(rar_platform)]
    pub unsafe fn map_user(&mut self,m:Mapping,start:u64,end:u64,device:bool)->Result<(),Error>{
        if device && (m.executable || !m.writable) {return Err(Error::Permission);}
        unsafe{self.map(m,start,end)?;}
        for i in 0..m.pages{
            let address=m.virtual_start+i*4096;
            let mut table=self.base;
            for shift in [39,30,21]{
                let p=(table as *mut u64).wrapping_add(((address>>shift)&511) as usize);
                let e=unsafe{p.read()};
                let next=e&ADDRESS;
                if e&1==0 || e&0x80!=0 || next<self.base || next>=self.base+self.used as u64*4096 {
                    return Err(Error::Permission);
                }
                unsafe{p.write(e|4);}
                table=next;
            }
            let p=(table as *mut u64).wrapping_add(((address>>12)&511) as usize);
            let e=unsafe{p.read()};
            unsafe{p.write(e|4|if device{0x18}else{0});}
        }
        Ok(())
    }
}
