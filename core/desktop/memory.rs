//! Compiler-required RAR-owned freestanding memory intrinsics.
//! Every caller must provide valid allocated readable/writable spans of n bytes;
//! n must fit pointer arithmetic within the allocation. Zero length dereferences
//! nothing. These functions do not validate untrusted pointers; syscall mapping
//! validation belongs to the kernel before copying guest request buffers.
#[cfg_attr(not(test),unsafe(no_mangle))]
/// Safety: d and s span n writable/readable bytes and do not overlap.
pub unsafe extern "C" fn memcpy(d:*mut u8,s:*const u8,n:usize)->*mut u8{for i in 0..n{unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}d}
#[cfg_attr(not(test),unsafe(no_mangle))]
/// Safety: d spans n writable bytes; no source allocation is read.
pub unsafe extern "C" fn memset(d:*mut u8,v:i32,n:usize)->*mut u8{for i in 0..n{unsafe{d.add(i).write_volatile(v as u8);}}d}
#[cfg_attr(not(test),unsafe(no_mangle))]
/// Safety: d and s span n writable/readable bytes; overlap is permitted.
pub unsafe extern "C" fn memmove(d:*mut u8,s:*const u8,n:usize)->*mut u8{
    if (d as usize)<(s as usize){for i in 0..n{unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}}
    else{for i in (0..n).rev(){unsafe{d.add(i).write_volatile(s.add(i).read_volatile());}}}d
}
#[cfg_attr(not(test),unsafe(no_mangle))]
/// Safety: a and b each span n readable bytes; overlap is permitted.
pub unsafe extern "C" fn memcmp(a:*const u8,b:*const u8,n:usize)->i32{for i in 0..n{let x=unsafe{a.add(i).read_volatile()};let y=unsafe{b.add(i).read_volatile()};if x!=y{return x as i32-y as i32;}}0}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn copy_set_compare_keep_canaries() {
        let source=[1u8,2,3,4];let mut destination=[0xa5u8;8];
        unsafe {
            memcpy(destination.as_mut_ptr().add(2),source.as_ptr(),4);
            assert_eq!(memcmp(destination.as_ptr().add(2),source.as_ptr(),4),0);
            memset(destination.as_mut_ptr().add(3),0x123,2);
        }
        assert_eq!(destination,[0xa5,0xa5,1,0x23,0x23,4,0xa5,0xa5]);
        assert!(unsafe{memcmp([1u8].as_ptr(),[2u8].as_ptr(),1)}<0);
    }
    #[test] fn overlapping_move_both_directions() {
        let mut bytes=[0u8,1,2,3,4,5,6,7];
        unsafe{memmove(bytes.as_mut_ptr().add(2),bytes.as_ptr(),6);}
        assert_eq!(bytes,[0,1,0,1,2,3,4,5]);
        unsafe{memmove(bytes.as_mut_ptr(),bytes.as_ptr().add(2),6);}
        assert_eq!(bytes,[0,1,2,3,4,5,4,5]);
    }
    #[test] fn zero_length_and_same_address_touch_nothing() {
        let mut bytes=[7u8;4];
        unsafe {
            let p=bytes.as_mut_ptr();memmove(p,p,4);memcpy(p,p,0);memset(p,0,0);
            assert_eq!(memcmp(p,p,0),0);
        }
        assert_eq!(bytes,[7;4]);
    }
}
