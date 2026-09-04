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
}
