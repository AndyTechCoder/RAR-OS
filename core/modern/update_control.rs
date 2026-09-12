//! Private laboratory request policy. No device, lifecycle or signing authority.
#![forbid(unsafe_code)]
pub const MAGIC:[u8;8]=*b"RARCTL01";
pub const CAP:usize=4;
pub fn command(bytes:&[u8])->Option<u64>{
    match bytes{b"update"=>Some(0),b"update badhealth"=>Some(1),
        b"update badsig"=>Some(2),b"update badabi"=>Some(3),_=>None}
}
pub fn frame(id:u64,index:u64)->Option<[u8;128]>{
    if id==0||index>3{return None;}
    let mut bytes=[0;128];bytes[..8].copy_from_slice(&MAGIC);
    bytes[8..16].copy_from_slice(&id.to_le_bytes());
    bytes[16..24].copy_from_slice(&index.to_le_bytes());Some(bytes)
}
pub struct Requests{used:bool}
impl Requests{
    pub const fn new()->Self{Self{used:false}}
    /// Identity must come from the kernel envelope and the current manager-only
    /// binding query. Rejected input never consumes replay state.
    pub fn accept(&mut self,sender:u64,incarnation:u64,current:u64,length:u64,
        bytes:&[u8;128])->Option<u64>{
        if self.used||sender!=6||current==0||incarnation!=current||length!=128||
            bytes[..8]!=MAGIC||
            bytes[24..].iter().any(|&b|b!=0){return None;}
        let id=u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let index=u64::from_le_bytes(bytes[16..24].try_into().ok()?);
        if id!=1||frame(id,index)?!=*bytes{return None;}
        self.used=true;Some(index)
    }
}
/// One successful install may authorize one exact-prior automatic recovery.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Action{Observe,Fallback,Stop}
pub struct Recovery{expected:u64,installed:bool,used:bool,pending:bool}
impl Recovery{
    pub fn new(expected:u64)->Option<Self>{
        if expected==0{return None;}Some(Self{expected,installed:false,used:false,pending:false})
    }
    pub fn observe(&mut self,current:u64)->Action{
        if current==self.expected{return Action::Observe;}
        if current!=0||!self.installed||self.used{return Action::Stop;}
        self.used=true;self.pending=true;Action::Fallback
    }
    pub fn installed(&mut self,committed:u64,current:u64)->bool{
        if current!=committed||self.installed||committed<=self.expected{return false;}
        self.expected=current;self.installed=true;true
    }
    pub fn restored(&mut self,committed:u64,current:u64)->bool{
        if current!=committed||!self.pending||committed<=self.expected{return false;}
        self.expected=current;self.pending=false;true
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn fallback_requires_install_and_never_oscillates(){
        assert!(Recovery::new(0).is_none());
        let mut r=Recovery::new(9).unwrap();
        assert_eq!(r.observe(9),Action::Observe);
        assert_eq!(r.observe(0),Action::Stop);
        assert_eq!(r.observe(10),Action::Stop);
        assert!(!r.installed(0,0));assert!(!r.installed(9,9));assert!(!r.restored(10,10));
        for current in [0,8,9,11,u64::MAX]{assert!(!r.installed(10,current));}
        assert!(r.installed(10,10));assert!(!r.installed(11,11));
        assert_eq!(r.observe(10),Action::Observe);
        assert_eq!(r.observe(11),Action::Stop);
        assert_eq!(r.observe(0),Action::Fallback);
        assert!(!r.restored(10,10));assert!(!r.restored(0,0));
        for current in [0,9,10,12,u64::MAX]{assert!(!r.restored(11,current));}
        assert!(r.restored(11,11));assert!(!r.restored(12,12));assert_eq!(r.observe(11),Action::Observe);
        assert_eq!(r.observe(0),Action::Stop);
    }
    #[test]fn fixed_commands_and_canonical_frames(){
        for (command,index)in [(b"update".as_slice(),0),(b"update badhealth",1),
            (b"update badsig",2),(b"update badabi",3)]{
            assert_eq!(super::command(command),Some(index));assert!(frame(1,index).is_some());
        }
        for command in [b"UPDATE".as_slice(),b"update factory",b"update /disk",b"update 4",b"update "]{
            assert_eq!(super::command(command),None);
        }
        assert!(frame(0,0).is_none());assert!(frame(1,4).is_none());
    }
    #[test]fn authenticate_before_consuming_full_width_replay_state(){
        let mut r=Requests::new();let n=(1u64<<40)+3;let b=frame(1,0).unwrap();
        for sender in 0..16{if sender!=6{assert_eq!(r.accept(sender,n,n,128,&b),None);}}
        for (inc,current,length)in [(n-1,n,128),(n,n-1,128),(0,0,128),(n,n,127),(n,n,129)]{
            assert_eq!(r.accept(6,inc,current,length,&b),None);
        }
        for at in 0..128{
            let mut bad=b;bad[at]^=0x80;
            if at>=24||at<8||at>=16{
                assert_eq!(r.accept(6,n,n,128,&bad),None);
            }
        }
        for at in 8..16{for bit in 0..8{
            let mut bad=b;bad[at]^=1<<bit;
            assert_eq!(r.accept(6,n,n,128,&bad),None);
        }}
        assert_eq!(r.accept(6,n,n,128,&frame(u64::MAX,0).unwrap()),None);
        assert_eq!(r.accept(6,n,n,128,&frame(2,0).unwrap()),None);
        assert_eq!(r.accept(6,n,n,128,&b),Some(0));
        assert_eq!(r.accept(6,n,n,128,&b),None);
        assert_eq!(r.accept(6,n,n,128,&frame(1,0).unwrap()),None);
        assert_eq!(r.accept(6,n+1,n+1,128,&frame(1,3).unwrap()),None);
        assert_eq!(r.accept(6,n,n,128,&b),None);
    }
}
