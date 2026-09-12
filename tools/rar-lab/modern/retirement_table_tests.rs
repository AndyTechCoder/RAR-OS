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
    fn leaf_pointer(pool:&Pool,tables:&paging::Tables,address:u64)->*mut u64{
        let base=pool.address();let end=base+tables.used() as u64*4096;
        let mut table=base;
        for shift in [39,30,21]{
            assert!(table>=base&&table+4096<=end&&table%4096==0);
            let e=unsafe{(table as *const u64).add(((address>>shift)&511) as usize).read()};
            assert_ne!(e&1,0);assert_eq!(e&0x80,0);table=e&0x000f_ffff_ffff_f000;
        }
        assert!(table>=base&&table+4096<=end&&table%4096==0);
        unsafe{(table as *mut u64).add(((address>>12)&511) as usize)}
    }
    #[test]fn aperture_staging_seal_alias_and_view_transitions_are_exact(){
        let region=staging::region(0x2000000,staging::ARENA_PAGES).unwrap();
        let pool=Pool::new();let mut tables=unsafe{paging::Tables::new(pool.address())};
        // Only test-owned tables are dereferenced; physical targets are numbers.
        unsafe{
            tables.map(model::Mapping{virtual_start:region.start,physical_start:region.start,
                pages:513,writable:true,executable:false},region.start,region.end).unwrap();
            tables.reserve_modern_verifier().unwrap();
            tables.check_modern_verifier(region.start,false).unwrap();
            tables.check_modern_staging(region.start,true).unwrap();
        }
        let used=tables.used();
        let last=leaf_pointer(&pool,&tables,region.end-4096);
        let original=unsafe{last.read()};
        // Hardware A/D bits do not defeat validation; forbidden writable-user,
        // executable, global/cache or wrong-physical aliases do.
        unsafe{last.write(original|0x60);}
        assert!(unsafe{tables.check_modern_staging(region.start,true)}.is_ok());
        for bad in [original|4,original&!(1<<63),original|0x100,original|0x18,original^4096]{
            unsafe{last.write(bad);}
            assert!(unsafe{tables.set_modern_staging(region.start,true)}.is_err());
            assert_eq!(entry(&pool,&tables,region.start).unwrap()&7,3);
        }
        unsafe{last.write(original);}
        unsafe{
            tables.set_modern_staging(region.start,true).unwrap();
            tables.check_modern_staging(region.start,false).unwrap();
            tables.publish_modern_verifier(region.start).unwrap();
        }
        for i in 0..513{
            assert_eq!(entry(&pool,&tables,region.start+i*4096).unwrap()&7,1);
            let e=entry(&pool,&tables,0x1400000+i*4096).unwrap();
            assert_eq!(e&7,5);assert_ne!(e&(1<<63),0);
            assert_eq!(e&0x000f_ffff_ffff_f000,region.start+i*4096);
        }
        assert_eq!(entry(&pool,&tables,0x13ff000),None);
        assert_eq!(entry(&pool,&tables,0x1601000),None);
        assert!(unsafe{tables.publish_modern_verifier(region.start)}.is_err());
        let first=leaf_pointer(&pool,&tables,0x1400000);let ro=unsafe{first.read()};
        unsafe{first.write(ro|2);}
        assert!(unsafe{tables.retire_modern_verifier(region.start)}.is_err());
        unsafe{
            first.write(ro);tables.retire_modern_verifier(region.start).unwrap();
            tables.check_modern_verifier(region.start,false).unwrap();
            tables.set_modern_staging(region.start,false).unwrap();
            tables.check_modern_staging(region.start,true).unwrap();
        }
        assert_eq!(tables.used(),used);
        // No CR3 reload, INVLPG, OS code or VM runs in this source fixture.
    }

}
