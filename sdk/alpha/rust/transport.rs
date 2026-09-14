//! Checked kernel-envelope framing, separate from syscall execution.
#![forbid(unsafe_code)]
use super::wire::{Boot,Error,Message};
pub const ENVELOPE_BYTES:usize=152;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Received{pub principal:u32,pub incarnation:u64,pub message:Message}
impl Received{
    /// Raw bytes must be copied from the actual RECEIVE output. Decoding bytes
    /// supplied by an app does not authenticate the claimed sender.
    pub fn decode(raw:&[u8],returned:i64)->Result<Self,Error>{
        if raw.len()!=ENVELOPE_BYTES||returned!=128{return Err(Error::Invalid);}
        let principal=u64::from_le_bytes(raw[..8].try_into().unwrap());
        let incarnation=u64::from_le_bytes(raw[8..16].try_into().unwrap());
        let length=u64::from_le_bytes(raw[16..24].try_into().unwrap());
        if principal>u32::MAX as u64||incarnation==0||length!=128{return Err(Error::Invalid);}
        Ok(Self{principal:principal as u32,incarnation,message:Message::decode(&raw[24..])?})
    }
    pub fn from_peer(&self,boot:&Boot,index:usize)->bool{
        index<4&&boot.peer_incarnations[index]!=0&&self.principal==boot.peer_principals[index]&&
            self.incarnation==boot.peer_incarnations[index]
    }
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn full_envelope_length_and_identity_are_never_truncated(){
        let mut b=[0;152];b[..8].copy_from_slice(&3u64.to_le_bytes());
        b[8..16].copy_from_slice(&0x1_0000_0001u64.to_le_bytes());
        b[16..24].copy_from_slice(&128u64.to_le_bytes());
        b[24..].copy_from_slice(&Message::new(4,0,1,b"x").unwrap().encode());
        assert_eq!(Received::decode(&b,128).unwrap().incarnation,0x1_0000_0001);
        for len in 0..152{assert!(Received::decode(&b[..len],128).is_err());}
        for returned in [-5,-1,0,1,127,129,i64::MAX]{assert!(Received::decode(&b,returned).is_err());}
        let mut bad=b;bad[4]=1;assert!(Received::decode(&bad,128).is_err());
        let mut bad=b;bad[8..16].fill(0);assert!(Received::decode(&bad,128).is_err());
        let mut bad=b;bad[16]=127;assert!(Received::decode(&bad,128).is_err());
    }
}
