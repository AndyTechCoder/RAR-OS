//! Cloud-only tests of actual page-table reservation; no privileged operation.
#![deny(unsafe_op_in_unsafe_fn)]
#[path="../../../nucleus/foundation/model.rs"] mod model;
#[path="../../../nucleus/foundation/paging.rs"] mod paging;
#[path="../../../nucleus/modern/staging.rs"] mod staging;
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
    fn entry(pool:&Pool,tables:&paging::Tables,address:u64)->Option<u64>{
        let base=pool.address();let end=base+tables.used() as u64*4096;
        let mut table=base;
        for shift in [39,30,21]{
            assert!(table>=base&&table+4096<=end&&table%4096==0);
            // SAFETY: walker validates every dereferenced test-owned table page.
            let e=unsafe{(table as *const u64).add(((address>>shift)&511) as usize).read()};
            if e&1==0{return None;}assert_eq!(e&0x80,0);
            table=e&0x000f_ffff_ffff_f000;
        }
        assert!(table>=base&&table+4096<=end&&table%4096==0);
        let e=unsafe{(table as *const u64).add(((address>>12)&511) as usize).read()};
        if e&1==0{None}else{Some(e)}
    }
    #[test]fn aperture_staging_actual_tables_keep_guards_absent_and_bytes_supervisor_nx(){
        let arena=0x2000000;let region=staging::region(arena,staging::ARENA_PAGES).unwrap();
        // Bootstrap-style root and each initial process-style root. User mapping
        // promotes intermediate U/S bits but must never promote staging leaves.
        for process in 0..=16{
            let pool=Pool::new();let mut tables=unsafe{paging::Tables::new(pool.address())};
            for page in 0..staging::ARENA_PAGES{
                let offset=page as u64*4096;
                if staging::guard_offset(offset){continue;}
                let address=arena+offset;
                unsafe{tables.map(model::Mapping{virtual_start:address,physical_start:address,
                    pages:1,writable:true,executable:false},arena,region.arena_end).unwrap();}
            }
            if process<16{
                let physical=arena+0x400000+process as u64*0x200000+0x100000;
                unsafe{tables.map_user(model::Mapping{virtual_start:0x400000,
                    physical_start:physical,pages:1,writable:false,executable:true},
                    physical,physical+4096,false).unwrap();}
                assert_eq!(entry(&pool,&tables,0x400000).unwrap()&7,5);
            }
            assert_eq!(entry(&pool,&tables,region.lower_guard),None);
            assert_eq!(entry(&pool,&tables,region.end),None);
            for address in (region.start..region.end).step_by(4096){
                let e=entry(&pool,&tables,address).unwrap();
                assert_eq!(e&0x000f_ffff_ffff_f000,address);
                assert_eq!(e&7,3);assert_ne!(e&(1<<63),0);
            }
            assert!(tables.used()<256);
        }
        // Tests page-table bytes only: no CR3, INVLPG, guest code or VM startup.
    }
    #[test]fn aperture_actual_capacity_failure_never_escapes_pool(){
        let pool=Pool::new();let base=pool.address();
        let mut tables=unsafe{paging::Tables::resume(base,256)};
        assert!(unsafe{tables.reserve_modern_aperture()}.is_err());
        assert_eq!(tables.used(),256);
        assert_eq!(unsafe{(base as *const u64).read()},0);
    }
}
