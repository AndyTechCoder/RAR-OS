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
pub struct Requests{incarnation:u64,last:u64}
impl Requests{
    pub const fn new()->Self{Self{incarnation:0,last:0}}
    /// Identity must come from the kernel envelope and the current manager-only
    /// binding query. Rejected input never consumes replay state.
    pub fn accept(&mut self,sender:u64,incarnation:u64,current:u64,length:u64,
        bytes:&[u8;128])->Option<u64>{
        if sender!=6||current==0||incarnation!=current||length!=128||
            incarnation<self.incarnation||bytes[..8]!=MAGIC||
            bytes[24..].iter().any(|&b|b!=0){return None;}
        let id=u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let index=u64::from_le_bytes(bytes[16..24].try_into().ok()?);
        if frame(id,index)?!=*bytes||(incarnation==self.incarnation&&id<=self.last){return None;}
        self.incarnation=incarnation;self.last=id;Some(index)
    }
}
/// Bound automatic recovery attempts between successful installations.
pub struct Recovery{used:bool}
impl Recovery{
    pub const fn new()->Self{Self{used:false}}
    pub fn lost(&mut self)->bool{
        if self.used{return false;}self.used=true;true
    }
    pub fn installed(&mut self){self.used=false;}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn fallback_is_once_until_an_explicit_successful_install(){
        let mut r=Recovery::new();assert!(r.lost());assert!(!r.lost());assert!(!r.lost());
        r.installed();assert!(r.lost());assert!(!r.lost());
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
        let mut r=Requests::new();let n=(1u64<<40)+3;let b=frame(u64::MAX,0).unwrap();
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
        assert_eq!(r.accept(6,n,n,128,&b),Some(0));
        assert_eq!(r.accept(6,n,n,128,&b),None);
        assert_eq!(r.accept(6,n,n,128,&frame(1,0).unwrap()),None);
        assert_eq!(r.accept(6,n+1,n+1,128,&frame(1,3).unwrap()),Some(3));
        assert_eq!(r.accept(6,n,n,128,&b),None);
    }
}
