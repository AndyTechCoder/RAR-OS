//! Private trusted-service control for independent app composition.
//! This is not the public application SDK; untrusted apps get no control call.
#![forbid(unsafe_code)]
pub const SYSCALL:u64=14;
pub const SIZE:usize=128;
pub const NOTES:[u8;16]=*b"rar.notes.alpha0";
pub const COUNTER:[u8;16]=*b"rar.counter.v000";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Record {
    pub index:usize,pub incarnation:u64,pub generation:u64,pub rights:u32,
    pub state:u32,pub application:[u8;16],pub owner:[u8;32],pub digest:[u8;32],
}
impl Record {
    pub fn encode(self)->Result<[u8;128],()>{
        if self.index>=2||self.generation==0||self.rights!=if self.index==0{3}else{1}||
            self.application!=if self.index==0{NOTES}else{COUNTER}||
            self.digest==[0;32]||self.state>2||(self.state==0)!=(self.incarnation==0){return Err(());}
        let mut b=[0;128];b[..8].copy_from_slice(b"RARACT00");
        for(at,n)in [(8,self.index as u32),(12,10+self.index as u32),(32,self.rights),(36,self.state)]{
            b[at..at+4].copy_from_slice(&n.to_le_bytes());
        }
        b[16..24].copy_from_slice(&self.incarnation.to_le_bytes());
        b[24..32].copy_from_slice(&self.generation.to_le_bytes());
        b[40..56].copy_from_slice(&self.application);b[56..88].copy_from_slice(&self.owner);
        b[88..120].copy_from_slice(&self.digest);Ok(b)
    }
    pub fn decode(b:&[u8])->Result<Self,()>{
        if b.len()!=128||&b[..8]!=b"RARACT00"||b[120..].iter().any(|x|*x!=0){return Err(());}
        let u32at=|at|u32::from_le_bytes(b[at..at+4].try_into().unwrap());
        let u64at=|at|u64::from_le_bytes(b[at..at+8].try_into().unwrap());
        let r=Self{index:u32at(8)as usize,incarnation:u64at(16),generation:u64at(24),rights:u32at(32),
            state:u32at(36),application:b[40..56].try_into().unwrap(),owner:b[56..88].try_into().unwrap(),
            digest:b[88..120].try_into().unwrap()};
        if r.encode()?.as_slice()!=b{return Err(());}Ok(r)
    }
}
pub fn control(op:u8,index:usize,incarnation:u64)->Result<[u8;128],()>{
    // Launch1/close2/focus3/synchronize4/ready5/failure6. No caller-selected path.
    if !(1..=6).contains(&op)||index>=2||(op==1&&incarnation!=0)||(op!=1&&incarnation==0){return Err(());}
    let mut b=[0;128];b[..8].copy_from_slice(b"RARACM00");b[8]=op;b[9]=index as u8;
    b[16..24].copy_from_slice(&incarnation.to_le_bytes());Ok(b)
}
pub fn parse(b:&[u8])->Result<(u8,usize,u64),()>{
    if b.len()!=128{return Err(());}
    let v=(b[8],b[9]as usize,u64::from_le_bytes(b[16..24].try_into().unwrap()));
    if control(v.0,v.1,v.2)?.as_slice()!=b{return Err(());}Ok(v)
}

pub fn input(index:usize,incarnation:u64,key:u8)->Result<[u8;128],()>{
    if index>=2||incarnation==0||!matches!(key,8|13|27|32..=126){return Err(());}
    let mut b=[0;128];b[..8].copy_from_slice(b"RARAKY00");b[8]=index as u8;b[9]=key;
    b[16..24].copy_from_slice(&incarnation.to_le_bytes());Ok(b)
}
pub fn parse_input(b:&[u8])->Result<(usize,u64,u8),()>{
    if b.len()!=128{return Err(());}
    let v=(b[8]as usize,u64::from_le_bytes(b[16..24].try_into().unwrap()),b[9]);
    if input(v.0,v.1,v.2)?.as_slice()!=b{return Err(());}Ok(v)
}

pub fn profile(compact:bool)->[u8;128]{
    let mut b=[0;128];b[..8].copy_from_slice(b"RARPRF00");b[8]=compact as u8;b
}
pub fn parse_profile(b:&[u8])->Result<bool,()>{
    if b.len()!=128||b[8]>1||profile(b[8]!=0).as_slice()!=b{return Err(());}Ok(b[8]!=0)
}
