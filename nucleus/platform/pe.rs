//! Bounded private native fixture loader model, not a general application format.
use super::Error;
pub const BASE:u64=0x400000;
pub const LIMIT:usize=128*1024;
pub const SECTIONS:usize=16;
#[derive(Clone,Copy,Debug)]
pub struct Section {
    pub virtual_offset:usize,pub memory_size:usize,pub file_offset:usize,pub file_size:usize,
    pub writable:bool,pub executable:bool,
}
impl Section {
    const EMPTY:Self=Self{virtual_offset:0,memory_size:0,file_offset:0,file_size:0,writable:false,executable:false};
}
pub struct Layout {pub entry:u64,pub image_size:usize,pub header_size:usize,pub count:usize,pub sections:[Section;SECTIONS]}
fn u16at(b:&[u8],p:usize)->Result<u16,Error>{
    Ok(u16::from_le_bytes(b.get(p..p+2).ok_or(Error::Invalid)?.try_into().map_err(|_|Error::Invalid)?))
}
fn u32at(b:&[u8],p:usize)->Result<u32,Error>{
    Ok(u32::from_le_bytes(b.get(p..p+4).ok_or(Error::Invalid)?.try_into().map_err(|_|Error::Invalid)?))
}
fn u64at(b:&[u8],p:usize)->Result<u64,Error>{
    Ok(u64::from_le_bytes(b.get(p..p+8).ok_or(Error::Invalid)?.try_into().map_err(|_|Error::Invalid)?))
}
pub fn parse(bytes:&[u8])->Result<Layout,Error>{
    if bytes.len()<512 || bytes.len()>2*1024*1024 || bytes.get(..2)!=Some(b"MZ") {return Err(Error::Invalid);}
    let pe=u32at(bytes,60)? as usize;
    if pe>4096 || bytes.get(pe..pe+4)!=Some(b"PE\0\0") || u16at(bytes,pe+4)?!=0x8664 {return Err(Error::Invalid);}
    let count=u16at(bytes,pe+6)? as usize;
    let optional=u16at(bytes,pe+20)? as usize;
    let o=pe+24;
    if count==0 || count>SECTIONS || optional<240 || optional>512 ||
        o+optional+count*40>bytes.len() || u16at(bytes,o)?!=0x20b ||
        u64at(bytes,o+24)?!=BASE || u32at(bytes,o+32)?!=4096 ||
        u32at(bytes,o+108)?!=16 {return Err(Error::Invalid);}
    // No imports, dynamic relocations, TLS or delayed imports. The fixture is
    // fully RAR-owned static code at its linked virtual base.
    for index in [1,5,9,13] {
        if u64at(bytes,o+112+index*8)?!=0 {return Err(Error::Denied);}
    }
    let image_size=u32at(bytes,o+56)? as usize;
    let header_size=u32at(bytes,o+60)? as usize;
    let entry=BASE+u32at(bytes,o+16)? as u64;
    if image_size==0 || image_size>LIMIT || image_size%4096!=0 ||
        header_size==0 || header_size>bytes.len() || header_size>4096 ||
        header_size<o+optional+count*40 {return Err(Error::Invalid);}
    let mut layout=Layout{entry,image_size,header_size,count,sections:[Section::EMPTY;SECTIONS]};
    let mut entry_executable=false;
    for index in 0..count {
        let s=o+optional+index*40;
        let memory_size=u32at(bytes,s+8)? as usize;
        let virtual_offset=u32at(bytes,s+12)? as usize;
        let file_size=u32at(bytes,s+16)? as usize;
        let file_offset=u32at(bytes,s+20)? as usize;
        let flags=u32at(bytes,s+36)?;
        let writable=flags&0x8000_0000!=0;
        let executable=flags&0x2000_0000!=0;
        let size=memory_size.max(file_size);
        let pages=size.div_ceil(4096);
        if size==0 || virtual_offset<4096 || virtual_offset%4096!=0 ||
            virtual_offset+pages*4096>image_size || file_offset+file_size>bytes.len() ||
            (file_size!=0 && file_offset<header_size) || writable&&executable ||
            flags&0x4000_0000==0 {return Err(Error::Denied);}
        for previous in &layout.sections[..index] {
            let end=virtual_offset+pages*4096;
            let other_end=previous.virtual_offset+previous.memory_size.div_ceil(4096)*4096;
            if virtual_offset<other_end && previous.virtual_offset<end {return Err(Error::Denied);}
        }
        let start=BASE+virtual_offset as u64;
        if executable && entry>=start && entry<start+memory_size as u64 {entry_executable=true;}
        layout.sections[index]=Section{virtual_offset,memory_size:size,file_offset,file_size,writable,executable};
    }
    if !entry_executable {return Err(Error::Denied);}
    Ok(layout)
}
#[cfg(test)]
mod tests{
    use super::*;
    fn put16(b:&mut[u8],p:usize,x:u16){b[p..p+2].copy_from_slice(&x.to_le_bytes());}
    fn put32(b:&mut[u8],p:usize,x:u32){b[p..p+4].copy_from_slice(&x.to_le_bytes());}
    fn image()->[u8;1024]{
        let mut b=[0;1024];b[..2].copy_from_slice(b"MZ");put32(&mut b,60,64);
        b[64..68].copy_from_slice(b"PE\0\0");put16(&mut b,68,0x8664);put16(&mut b,70,1);put16(&mut b,84,240);
        let o=88;put16(&mut b,o,0x20b);put32(&mut b,o+16,4096);b[o+24..o+32].copy_from_slice(&BASE.to_le_bytes());
        put32(&mut b,o+32,4096);put32(&mut b,o+56,8192);put32(&mut b,o+60,512);put32(&mut b,o+108,16);
        let s=o+240;put32(&mut b,s+8,16);put32(&mut b,s+12,4096);put32(&mut b,s+16,512);put32(&mut b,s+20,512);
        put32(&mut b,s+36,0x6000_0020);b
    }
    #[test]fn valid_static_fixture(){let l=parse(&image()).unwrap();assert_eq!(l.entry,BASE+4096);assert!(!l.sections[0].writable);}
    #[test]fn malformed_and_wx_fail(){
        let good=image();
        for length in [0,511,700]{assert!(parse(&good[..length]).is_err());}
        for (offset,value) in [(60,u32::MAX),(88+56,0x100000),(88+16,0),(328+36,0xe0000020),
            (328+20,u32::MAX),(328+12,0),(88+112+8,1),(88+112+5*8,1),(88+112+9*8,1)]{
            let mut b=good;put32(&mut b,offset,value);assert!(parse(&b).is_err(),"offset {}",offset);
        }
    }
}
