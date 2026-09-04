//! RAR-owned deterministic FAT16 image generator. Cloud host tool only.
use std::{env, fs};
const SECTOR: usize=512;
const DATA: usize=97*SECTOR;
const CLUSTER: usize=2048;
fn u16at(b:&mut [u8], o:usize, n:u16) { b[o..o+2].copy_from_slice(&n.to_le_bytes()); }
fn u32at(b:&mut [u8], o:usize, n:u32) { b[o..o+4].copy_from_slice(&n.to_le_bytes()); }
fn entry(b:&mut [u8], o:usize, name:&[u8;11], attrs:u8, cluster:u16, size:u32) {
    b[o..o+11].copy_from_slice(name); b[o+11]=attrs;
    u16at(b,o+26,cluster); u32at(b,o+28,size);
    u16at(b,o+24,((2026-1980)<<9)|1<<5|1);
}
fn cluster(n:usize)->usize { DATA+(n-2)*CLUSTER }
fn build(efi:&[u8])->Vec<u8> {
    assert!((1024..=2097152).contains(&efi.len()) && &efi[..2]==b"MZ");
    let pe=u32::from_le_bytes(efi[60..64].try_into().unwrap()) as usize;
    assert!(pe+96<=efi.len() && &efi[pe..pe+4]==b"PE\0\0");
    assert_eq!(&efi[pe+4..pe+6], &0x8664u16.to_le_bytes());
    assert_eq!(&efi[pe+24..pe+26], &0x20bu16.to_le_bytes());
    assert_eq!(&efi[pe+92..pe+94], &10u16.to_le_bytes());
    let mut b=vec![0;16777216];
    b[..3].copy_from_slice(&[0xeb,0x3c,0x90]); b[3..11].copy_from_slice(b"RAROS   ");
    u16at(&mut b,11,512); b[13]=4; u16at(&mut b,14,1); b[16]=2;
    u16at(&mut b,17,512); u16at(&mut b,19,32768); b[21]=0xf8;
    u16at(&mut b,22,32); u16at(&mut b,24,32); u16at(&mut b,26,64);
    b[36]=0x80; b[38]=0x29; u32at(&mut b,39,0x52415231);
    b[43..54].copy_from_slice(b"RAR ALPHA  "); b[54..62].copy_from_slice(b"FAT16   ");
    b[510]=0x55; b[511]=0xaa;
    let file_clusters=efi.len().div_ceil(CLUSTER);
    for fat in [512,33*512] {
        u16at(&mut b,fat,0xfff8); u16at(&mut b,fat+2,0xffff);
        u16at(&mut b,fat+4,0xffff); u16at(&mut b,fat+6,0xffff);
        for i in 0..file_clusters {
            u16at(&mut b,fat+(4+i)*2,if i+1==file_clusters {0xffff} else {(5+i) as u16});
        }
    }
    entry(&mut b,65*512,b"EFI        ",0x10,2,0);
    entry(&mut b,cluster(2),b".          ",0x10,2,0);
    entry(&mut b,cluster(2)+32,b"..         ",0x10,0,0);
    entry(&mut b,cluster(2)+64,b"BOOT       ",0x10,3,0);
    entry(&mut b,cluster(3),b".          ",0x10,3,0);
    entry(&mut b,cluster(3)+32,b"..         ",0x10,2,0);
    entry(&mut b,cluster(3)+64,b"BOOTX64 EFI",0x20,4,efi.len() as u32);
    b[cluster(4)..cluster(4)+efi.len()].copy_from_slice(efi);
    b
}
fn main() {
    let args:Vec<_>=env::args().collect(); assert_eq!(args.len(),3);
    let efi=fs::read(&args[1]).unwrap(); let image=build(&efi);
    fs::write(&args[2],image).unwrap();
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn fat_geometry_and_payload() {
        let mut e=vec![0u8;3000]; e[..2].copy_from_slice(b"MZ");
        u32at(&mut e,60,128); e[128..132].copy_from_slice(b"PE\0\0");
        u16at(&mut e,132,0x8664); u16at(&mut e,152,0x20b); u16at(&mut e,220,10);
        let image=build(&e); assert_eq!(image.len(),16777216);
        assert_eq!(&image[512..33*512],&image[33*512..65*512]);
        assert_eq!(&image[cluster(4)..cluster(4)+e.len()],&e);
        assert!((4085..65525).contains(&((32768-97)/4)));
    }
    #[test] #[should_panic] fn invalid_executable_rejected() { build(&[0;1024]); }
}
