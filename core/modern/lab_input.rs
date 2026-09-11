//! Immutable laboratory package window geometry, not a disk or network API.
#![forbid(unsafe_code)]
pub const COUNT:usize=5;
pub const BASE:u64=0x40_0000_0000;
pub const STRIDE:u64=0x40_0000;
pub const MAX_BYTES:usize=2_097_536;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Window{pub address:u64,pub physical:u64,pub pages:u64,pub bytes:usize}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]pub struct Invalid;
fn overlap(a:u64,b:u64,c:u64,d:u64)->bool{a<d&&c<b}
pub fn window(index:usize,bytes:usize,padded:usize,physical:u64,
    image:(u64,u64),arena:(u64,u64))->Result<Window,Invalid>{
    if index>=COUNT||!(896..=MAX_BYTES).contains(&bytes)||padded!=bytes.div_ceil(4096)*4096||
        physical%4096!=0||image.1==0||arena.1==0{return Err(Invalid);}
    let end=physical.checked_add(padded as u64).ok_or(Invalid)?;
    let image_end=image.0.checked_add(image.1).ok_or(Invalid)?;
    let arena_end=arena.0.checked_add(arena.1).ok_or(Invalid)?;
    if physical<image.0||end>image_end||overlap(physical,end,arena.0,arena_end){return Err(Invalid);}
    let address=BASE+index as u64*STRIDE;
    let virtual_end=address.checked_add(padded as u64).ok_or(Invalid)?;
    if virtual_end>=1u64<<47||overlap(address,virtual_end,image.0,image_end)||
        overlap(address,virtual_end,arena.0,arena_end){return Err(Invalid);}
    Ok(Window{address,physical,pages:(padded/4096)as u64,bytes})
}
pub fn response(index:usize,address:u64,bytes:u64)->Result<usize,Invalid>{
    let length=usize::try_from(bytes).map_err(|_|Invalid)?;
    if index>=COUNT||address!=BASE+index as u64*STRIDE||!(896..=MAX_BYTES).contains(&length){
        return Err(Invalid);
    }
    Ok(length)
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn immutable_windows_are_bounded_disjoint_and_not_kernel_aliases(){
        let image=(0x1000_0000,0x1000000);let arena=(0x2000_0000,0x2800000);
        for index in 0..COUNT{for bytes in [896,4096,4097,MAX_BYTES]{
            let padded=bytes.div_ceil(4096)*4096;
            let w=window(index,bytes,padded,image.0,image,arena).unwrap();
            assert_eq!(response(index,w.address,bytes as u64),Ok(bytes));
            assert!(w.address+w.pages*4096<BASE+(index+1)as u64*STRIDE);
            assert!(window(index,bytes,padded,image.0+1,image,arena).is_err());
            assert!(window(index,bytes,padded+4096,image.0,image,arena).is_err());
            assert!(response(index,w.address+1,bytes as u64).is_err());
        }}
        for bytes in [0,895,MAX_BYTES+1,usize::MAX]{
            assert!(window(0,bytes,4096,image.0,image,arena).is_err());
        }
        assert!(window(COUNT,896,4096,image.0,image,arena).is_err());
        assert!(window(0,896,4096,image.0-4096,image,arena).is_err());
        assert!(window(0,896,4096,image.0,image,(image.0,4096)).is_err());
        assert!(window(0,896,4096,BASE,(BASE,4096),arena).is_err());
        assert!(window(0,896,4096,image.0,image,(BASE,4096)).is_err());
        assert!(window(0,896,4096,u64::MAX-4095,(u64::MAX-4095,4096),arena).is_err());
    }
}
