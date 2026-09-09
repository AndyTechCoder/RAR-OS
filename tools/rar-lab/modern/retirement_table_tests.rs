//! Cloud-only tests of actual page-table reservation; no privileged operation.
#![deny(unsafe_op_in_unsafe_fn)]
#[path="../../../nucleus/foundation/model.rs"] mod model;
#[path="../../../nucleus/foundation/paging.rs"] mod paging;
#[cfg(test)]
mod tests{
    use super::*;
    use std::alloc::{alloc_zeroed,dealloc,Layout};
    struct Pool{pointer:*mut u8,layout:Layout}
    impl Pool{
        fn new()->Self{
            let layout=Layout::from_size_align(1024*1024,4096).unwrap();
            // SAFETY: test-owned aligned allocation; never an OS or disk image.
            let pointer=unsafe{alloc_zeroed(layout)};
            assert!(!pointer.is_null());Self{pointer,layout}
        }
        fn address(&self)->u64{self.pointer as u64}
    }
    impl Drop for Pool{
        fn drop(&mut self){
            // SAFETY: exact allocation, no outstanding dereferenced references.
            unsafe{dealloc(self.pointer,self.layout);}
        }
    }
    #[test]fn aperture_actual_reservation_is_empty_aligned_and_owned(){
        let pool=Pool::new();let base=pool.address();
        // SAFETY: only the fresh test-owned 1 MiB pool is accessed; reservation
        // performs no CR3 write, INVLPG, syscall, device I/O or target execution.
        let mut tables=unsafe{paging::Tables::new(base)};
        assert!(unsafe{tables.modern_aperture()}.is_err());
        assert_eq!(tables.used(),1);
        let leaf=unsafe{tables.reserve_modern_aperture().unwrap()};
        assert_eq!(leaf%4096,0);
        assert!(leaf>=base+4096&&leaf<base+tables.used() as u64*4096);
        for i in 0..512{assert_eq!(unsafe{(leaf as *const u64).add(i).read()},0);}
        let used=tables.used();
        assert_eq!(unsafe{tables.reserve_modern_aperture().unwrap()},leaf);
        assert_eq!(tables.used(),used);
        assert_eq!(unsafe{tables.modern_aperture().unwrap()},leaf);
        assert_eq!(tables.used(),used);
        // Any residue, even a nonpresent software value, forbids reservation.
        unsafe{(leaf as *mut u64).add(511).write(2);}
        assert!(unsafe{tables.reserve_modern_aperture()}.is_err());
        assert!(unsafe{tables.modern_aperture()}.is_err());
    }
    #[test]fn aperture_actual_capacity_failure_never_escapes_pool(){
        let pool=Pool::new();let base=pool.address();
        let mut tables=unsafe{paging::Tables::resume(base,256)};
        assert!(unsafe{tables.reserve_modern_aperture()}.is_err());
        assert_eq!(tables.used(),256);
        assert_eq!(unsafe{(base as *const u64).read()},0);
    }
}
