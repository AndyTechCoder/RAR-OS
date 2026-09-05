//! Ring3-owned, explicitly volatile single-level storage and PS/2 decoding.
//! Experimental Platform-v0 test service protocol; not a persistent filesystem.
pub const WIRE: usize = 128;
pub const FILES: usize = 16;
pub const MAX_NAME: usize = 12;
pub const MAX_DATA: usize = 64;
pub const OWNER_FILES: usize = 4;
pub const OWNER_BYTES: usize = 128;
pub const CREATE: u8 = 1;
pub const WRITE: u8 = 2;
pub const READ: u8 = 3;
pub const LIST: u8 = 4;
pub const OK: u8 = 0;
pub const INVALID: u8 = 1;
pub const NOT_FOUND: u8 = 2;
pub const EXISTS: u8 = 3;
pub const QUOTA: u8 = 4;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Owner { pub task: u8, pub generation: u32 }
#[derive(Clone, Copy)]
struct File {
    used: bool, owner: Owner, name_len: u8, name: [u8; MAX_NAME],
    data_len: u8, data: [u8; MAX_DATA],
}
impl File {
    const EMPTY: Self = Self { used: false, owner: Owner { task: 0, generation: 0 },
        name_len: 0, name: [0; MAX_NAME], data_len: 0, data: [0; MAX_DATA] };
}
pub struct Store { files: [File; FILES] }
impl Store {
    pub const fn new() -> Self { Self { files: [File::EMPTY; FILES] } }
    /// Owner is taken only from the kernel's receive envelope, never request bytes.
    pub fn process(&mut self, owner: Owner, request: &[u8]) -> [u8; WIRE] {
        let mut reply=[0;WIRE];
        if owner.generation == 0 || request.len()!=WIRE { reply[0]=INVALID; return reply; }
        let op=request[0]; let name_len=request[1] as usize; let data_len=request[2] as usize;
        if !matches!(op,CREATE|WRITE|READ|LIST) || name_len>MAX_NAME || data_len>MAX_DATA ||
            request[3]!=0 || request[4+name_len..16].iter().any(|&v|v!=0) ||
            request[16+data_len..].iter().any(|&v|v!=0) ||
            (op!=WRITE && data_len!=0) {
            reply[0]=INVALID; return reply;
        }
        let name=&request[4..4+name_len];
        if (op==LIST && name_len!=0) || (op!=LIST && (name.is_empty() ||
            name==b"." || name==b".." || !name.iter().all(|b|b.is_ascii_alphanumeric() || *b==b'.' || *b==b'-'))) {
            reply[0]=INVALID; return reply;
        }
        let matching=self.files.iter().position(|f|f.used && f.owner==owner &&
            f.name_len as usize==name_len && &f.name[..name_len]==name);
        let owner_count=self.files.iter().filter(|f|f.used && f.owner==owner).count();
        let owner_bytes:usize=self.files.iter().filter(|f|f.used && f.owner==owner)
            .map(|f|f.data_len as usize).sum();
        match op {
            CREATE => {
                if matching.is_some() { reply[0]=EXISTS; return reply; }
                if owner_count>=OWNER_FILES { reply[0]=QUOTA; return reply; }
                let Some(slot)=self.files.iter_mut().find(|f|!f.used) else { reply[0]=QUOTA; return reply; };
                *slot=File { used:true, owner, name_len:name_len as u8, ..File::EMPTY };
                slot.name[..name_len].copy_from_slice(name);
            }
            WRITE => {
                let Some(index)=matching else { reply[0]=NOT_FOUND; return reply; };
                let file=&mut self.files[index];
                if owner_bytes-file.data_len as usize+data_len>OWNER_BYTES { reply[0]=QUOTA; return reply; }
                file.data=[0;MAX_DATA]; file.data[..data_len].copy_from_slice(&request[16..16+data_len]);
                file.data_len=data_len as u8;
            }
            READ => {
                let Some(index)=matching else { reply[0]=NOT_FOUND; return reply; };
                let file=&self.files[index]; reply[2]=file.data_len;
                reply[16..16+file.data_len as usize].copy_from_slice(&file.data[..file.data_len as usize]);
            }
            LIST => {
                let mut offset=4;
                for file in self.files.iter().filter(|f|f.used && f.owner==owner) {
                    reply[offset]=file.name_len; offset+=1;
                    let n=file.name_len as usize;
                    reply[offset..offset+n].copy_from_slice(&file.name[..n]); offset+=n;
                    reply[1]+=1;
                }
            }
            _ => unreachable!(),
        }
        reply
    }
}
pub fn request(op:u8,name:&[u8],data:&[u8])->Option<[u8;WIRE]> {
    if name.len()>MAX_NAME || data.len()>MAX_DATA { return None; }
    let mut wire=[0;WIRE]; wire[0]=op; wire[1]=name.len() as u8; wire[2]=data.len() as u8;
    wire[4..4+name.len()].copy_from_slice(name); wire[16..16+data.len()].copy_from_slice(data); Some(wire)
}
/// Set-1 decoder for the fixed translated PS/2 controller. Extended keys do not
/// become A events. Proof requires make followed by break, never a lone byte.
pub struct Keyboard { extended:bool, made:bool, completed:bool }
impl Keyboard {
    pub const fn new()->Self { Self {extended:false,made:false,completed:false} }
    pub fn feed(&mut self,byte:u8)->bool {
        if matches!(byte,0xe0|0xe1) { self.extended=true; return false; }
        if self.extended { self.extended=false; return false; }
        if byte==0x1e { self.made=true; }
        if byte==0x9e && self.made { self.completed=true; self.made=false; }
        self.completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const A:Owner=Owner{task:0,generation:1};
    const B:Owner=Owner{task:10,generation:1};
    fn call(s:&mut Store,owner:Owner,op:u8,name:&[u8],data:&[u8])->[u8;WIRE] {
        s.process(owner,&request(op,name,data).unwrap())
    }
    #[test] fn real_create_write_read_list() {
        let mut s=Store::new();
        assert_eq!(call(&mut s,A,CREATE,b"alpha",b"")[0],OK);
        assert_eq!(call(&mut s,A,WRITE,b"alpha",b"RAR")[0],OK);
        let got=call(&mut s,A,READ,b"alpha",b""); assert_eq!(got[2],3); assert_eq!(&got[16..19],b"RAR");
        let list=call(&mut s,A,LIST,b"",b""); assert_eq!(list[1],1); assert_eq!(&list[5..10],b"alpha");
    }
    #[test] fn namespace_and_generation_are_not_request_controlled() {
        let mut s=Store::new(); call(&mut s,A,CREATE,b"secret",b"");
        assert_eq!(call(&mut s,B,READ,b"secret",b"")[0],NOT_FOUND);
        assert_eq!(call(&mut s,B,LIST,b"",b"")[1],0);
        assert_eq!(call(&mut s,Owner{task:0,generation:2},READ,b"secret",b"")[0],NOT_FOUND);
        assert_eq!(call(&mut s,B,CREATE,b"secret",b"")[0],OK);
        assert_eq!(call(&mut s,A,CREATE,b"secret",b"")[0],EXISTS);
    }
    #[test] fn quotas_fail_without_modifying_existing_bytes() {
        let mut s=Store::new();
        for name in [b"a",b"b",b"c",b"d"] { assert_eq!(call(&mut s,A,CREATE,name,b"")[0],OK); }
        assert_eq!(call(&mut s,A,CREATE,b"e",b"")[0],QUOTA);
        assert_eq!(call(&mut s,A,WRITE,b"a",&[1;64])[0],OK);
        assert_eq!(call(&mut s,A,WRITE,b"b",&[2;64])[0],OK);
        assert_eq!(call(&mut s,A,WRITE,b"c",&[3])[0],QUOTA);
        assert_eq!(call(&mut s,A,READ,b"a",b"")[16],1);
        assert_eq!(call(&mut s,A,WRITE,b"a",&[4;32])[0],OK);
        assert_eq!(call(&mut s,A,WRITE,b"c",&[3;32])[0],OK);
    }
    #[test] fn malformed_wire_and_paths_rejected() {
        let mut s=Store::new();
        assert_eq!(s.process(A,&[])[0],INVALID);
        for name in [b"/".as_slice(),b"..".as_slice(),b"a/b".as_slice()] {
            assert_eq!(call(&mut s,A,CREATE,name,b"")[0],INVALID);
        }
        let mut r=request(CREATE,b"a",b"").unwrap(); r[127]=1;
        assert_eq!(s.process(A,&r)[0],INVALID);
        r[127]=0; r[1]=255; assert_eq!(s.process(A,&r)[0],INVALID);
        r[1]=1; r[0]=255; assert_eq!(s.process(A,&r)[0],INVALID);
    }
    #[test] fn physical_set1_make_break_decoder() {
        let mut k=Keyboard::new(); assert!(!k.feed(0x9e));
        assert!(!k.feed(0xe0)); assert!(!k.feed(0x1e)); assert!(!k.feed(0x9e));
        assert!(!k.feed(0x1e)); assert!(k.feed(0x9e));
    }
}
