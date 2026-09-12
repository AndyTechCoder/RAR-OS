//! Checked private-memory retirement geometry. No authority comes from callers.
#![forbid(unsafe_code)]
pub const TASKS:usize=16;
pub const PRIVATE_BASE:u64=0x400000;
pub const STRIDE:u64=0x200000;
pub const TABLE_BYTES:u64=0x100000;
pub const APERTURE:u64=0x1000000;
pub const APERTURE_PAGES:usize=512;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Memory {Clean,Live,Retiring}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Plan {pub victim:u64,pub current_root:u64,pub leaf:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Geometry,State,Context}
pub fn region(arena:u64,pages:usize,index:usize)->Result<u64,Error>{
    let bytes=(pages as u64).checked_mul(4096).ok_or(Error::Geometry)?;
    let end=arena.checked_add(bytes).ok_or(Error::Geometry)?;
    if arena<0x2000000||arena%4096!=0||end>0x1_0000_0000||
        bytes<PRIVATE_BASE+TASKS as u64*STRIDE||index>=TASKS{return Err(Error::Geometry);}
    arena.checked_add(PRIVATE_BASE).and_then(|v|v.checked_add(index as u64*STRIDE)).ok_or(Error::Geometry)
}
pub fn retire(memory:Memory)->Memory{
    match memory {Memory::Live=>Memory::Retiring,other=>other}
}
/// A dying current stack remains unavailable until a subsequent survivor trap.
pub fn plan(arena:u64,pages:usize,current:usize,victim:usize,memory:Memory,
    victim_root:u64,bottom:u64,top:u64,leaf:u64)->Result<Plan,Error>{
    if current==victim||memory!=Memory::Retiring{return Err(Error::State);}
    let current_root=region(arena,pages,current)?;
    let victim=region(arena,pages,victim)?;
    if victim_root!=victim||bottom!=victim+0x141000||top!=victim+0x151000||
        leaf%4096!=0||leaf<current_root+4096||leaf>=current_root+TABLE_BYTES {
        return Err(Error::Geometry);
    }
    Ok(Plan{victim,current_root,leaf})
}
/// No stale global/PCID translation may survive an ordinary CR3 switch.
pub fn context(root:u64,expected:u64,cr4:u64,flags:u64)->Result<(),Error>{
    if root!=expected||cr4&((1<<7)|(1<<17))!=0||flags&(1<<9)!=0{
        Err(Error::Context)
    }else{Ok(())}
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn all_private_slots_are_disjoint_bounded_and_not_aperture(){
        for i in 0..TASKS{
            let p=region(0x2000000,9216,i).unwrap();
            assert_eq!(p,0x2400000+i as u64*STRIDE);
            assert!(p>=APERTURE+APERTURE_PAGES as u64*4096);
            if i+1<TASKS{assert_eq!(p+STRIDE,region(0x2000000,9216,i+1).unwrap());}
        }
        for (a,n,i) in [(0,9216,0),(0x2000001,9216,0),(0x2000000,9215,0),
            (0x2000000,9216,16),(u64::MAX-4095,9216,0),(0xfffff000,9216,0)]{
            assert_eq!(region(a,n,i),Err(Error::Geometry));
        }
    }
    #[test]fn current_and_clean_memory_never_retire_or_become_reusable_early(){
        assert_eq!(retire(Memory::Clean),Memory::Clean);
        assert_eq!(retire(Memory::Live),Memory::Retiring);
        assert_eq!(retire(Memory::Retiring),Memory::Retiring);
        let a=0x2000000;let victim=region(a,9216,6).unwrap();
        let leaf=region(a,9216,1).unwrap()+4096;
        let valid=|current,state,root,bottom,top,pte|plan(a,9216,current,6,state,root,bottom,top,pte);
        assert!(valid(1,Memory::Retiring,victim,victim+0x141000,victim+0x151000,leaf).is_ok());
        for state in [Memory::Clean,Memory::Live]{
            assert_eq!(valid(1,state,victim,victim+0x141000,victim+0x151000,leaf),Err(Error::State));
        }
        assert_eq!(valid(6,Memory::Retiring,victim,victim+0x141000,victim+0x151000,leaf),Err(Error::State));
        for (root,bottom,top,pte) in [(victim+4096,victim+0x141000,victim+0x151000,leaf),
            (victim,victim+0x141001,victim+0x151000,leaf),
            (victim,victim+0x141000,victim+0x152000,leaf),
            (victim,victim+0x141000,victim+0x151000,leaf+1),
            (victim,victim+0x141000,victim+0x151000,victim+4096),
            (victim,victim+0x141000,victim+0x151000,leaf-4096),
            (victim,victim+0x141000,victim+0x151000,leaf-4096+TABLE_BYTES)]{
            assert!(valid(1,Memory::Retiring,root,bottom,top,pte).is_err());
        }
    }
    #[test]fn explicit_translation_and_interrupt_context(){
        assert_eq!(context(0x2400000,0x2400000,0,2),Ok(()));
        for (root,cr4,flags) in [(0x2400001,0,2),(0x2400000,1<<7,2),
            (0x2400000,1<<17,2),(0x2400000,0,0x202)]{
            assert_eq!(context(root,0x2400000,cr4,flags),Err(Error::Context));
        }
    }
}
