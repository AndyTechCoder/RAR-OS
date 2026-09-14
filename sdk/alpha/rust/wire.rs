//! Shared app wire codec. No syscall authority or borrowed raw bootstrap pointers.
#![forbid(unsafe_code)]
include!("constants.rs");
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Invalid,Version}
fn u32_at(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn u64_at(b:&[u8],at:usize)->u64{u64::from_le_bytes(b[at..at+8].try_into().unwrap())}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Boot{
    pub application:[u8;16],pub incarnation:u64,pub principal:u32,pub rights:u32,
    pub entry:u64,pub caps:[u64;5],pub peer_principals:[u32;4],pub peer_incarnations:[u64;4],
}
impl Boot{
    /// Kernel-side byte construction. Revalidate the complete result so public
    /// fields cannot encode a bootstrap the independent decoder would refuse.
    pub fn encode(&self)->Result<[u8;BOOT_BYTES],Error>{
        let mut raw=[0;BOOT_BYTES];raw[..8].copy_from_slice(b"RARAPP00");
        raw[8..12].copy_from_slice(&ABI.to_le_bytes());
        raw[12..16].copy_from_slice(&(BOOT_BYTES as u32).to_le_bytes());
        raw[16..32].copy_from_slice(&self.application);
        raw[32..40].copy_from_slice(&self.incarnation.to_le_bytes());
        raw[40..44].copy_from_slice(&self.principal.to_le_bytes());
        raw[44..48].copy_from_slice(&self.rights.to_le_bytes());
        raw[48..56].copy_from_slice(&self.entry.to_le_bytes());
        for i in 0..5{raw[56+i*8..64+i*8].copy_from_slice(&self.caps[i].to_le_bytes());}
        for i in 0..4{
            raw[96+i*4..100+i*4].copy_from_slice(&self.peer_principals[i].to_le_bytes());
            raw[112+i*8..120+i*8].copy_from_slice(&self.peer_incarnations[i].to_le_bytes());
        }
        if Self::decode(&raw)?!=*self{return Err(Error::Invalid);}Ok(raw)
    }
    /// Decode bytes from the kernel's read-only bootstrap mapping. Byte validity
    /// cannot prove their source: callers must not accept this over IPC.
    pub fn decode(raw:&[u8])->Result<Self,Error>{
        if raw.len()!=BOOT_BYTES||&raw[..8]!=b"RARAPP00"||u32_at(raw,12)!=256||
            raw[144..].iter().any(|&b|b!=0){return Err(Error::Invalid);}
        if u32_at(raw,8)!=ABI{return Err(Error::Version);}
        let mut b=Self{application:raw[16..32].try_into().unwrap(),
            incarnation:u64_at(raw,32),principal:u32_at(raw,40),rights:u32_at(raw,44),
            entry:u64_at(raw,48),caps:[0;5],peer_principals:[0;4],peer_incarnations:[0;4]};
        if b.application==[0;16]||b.incarnation==0||!matches!(b.principal,10|11)||
            b.rights&UI==0||b.rights&!15!=0||!(0x400000..0x420000).contains(&b.entry){
            return Err(Error::Invalid);
        }
        for i in 0..5{
            b.caps[i]=u64_at(raw,56+i*8);
            let enabled=i==0||b.rights&(1<<(i-1))!=0;
            if enabled{
                if b.caps[i]>>32==0||b.caps[i]as u32!=i as u32+1{return Err(Error::Invalid);}
            }else if b.caps[i]!=0{return Err(Error::Invalid);}
        }
        for(i,expected)in [3u32,1,7,12].into_iter().enumerate(){
            b.peer_principals[i]=u32_at(raw,96+i*4);
            b.peer_incarnations[i]=u64_at(raw,112+i*8);
            if b.rights&(1<<i)!=0{
                if b.peer_principals[i]!=expected||b.peer_incarnations[i]==0{return Err(Error::Invalid);}
            }else if b.peer_principals[i]!=0||b.peer_incarnations[i]!=0{return Err(Error::Invalid);}
        }
        Ok(b)
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Message{operation:u8,status:u8,sequence:u32,len:u16,data:[u8;PAYLOAD_BYTES]}
impl Message{
    pub fn operation(&self)->u8{self.operation}
    pub fn status(&self)->u8{self.status}
    pub fn sequence(&self)->u32{self.sequence}
    pub fn new(operation:u8,status:u8,sequence:u32,payload:&[u8])->Result<Self,Error>{
        if !(1..=6).contains(&operation)||status>7||sequence==0||payload.len()>PAYLOAD_BYTES{
            return Err(Error::Invalid);
        }
        let mut m=Self{operation,status,sequence,len:payload.len()as u16,data:[0;PAYLOAD_BYTES]};
        m.data[..payload.len()].copy_from_slice(payload);Ok(m)
    }
    pub fn payload(&self)->&[u8]{&self.data[..self.len as usize]}
    pub fn encode(&self)->[u8;MESSAGE_BYTES]{
        let mut b=[0;MESSAGE_BYTES];b[..4].copy_from_slice(b"RAPP");
        b[5]=self.operation;b[6]=self.status;b[8..12].copy_from_slice(&self.sequence.to_le_bytes());
        b[12..14].copy_from_slice(&self.len.to_le_bytes());b[16..].copy_from_slice(&self.data);b
    }
    pub fn decode(b:&[u8])->Result<Self,Error>{
        if b.len()!=MESSAGE_BYTES||&b[..4]!=b"RAPP"||b[4]!=0||b[7]!=0||b[14..16]!=[0;2]{
            return Err(Error::Invalid);
        }
        let len=u16::from_le_bytes(b[12..14].try_into().unwrap())as usize;
        if len>PAYLOAD_BYTES||b[16+len..].iter().any(|&x|x!=0){return Err(Error::Invalid);}
        Self::new(b[5],b[6],u32_at(b,8),&b[16..16+len])
    }
    /// Reply authority includes full kernel-stamped identity and correlation.
    pub fn reply_to(&self,request:&Self,principal:u32,incarnation:u64,
        expected_principal:u32,expected_incarnation:u64)->bool{
        expected_incarnation!=0&&principal==expected_principal&&incarnation==expected_incarnation&&
            self.sequence==request.sequence&&self.operation==request.operation
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    fn boot()->[u8;256]{
        let mut b=[0;256];b[..8].copy_from_slice(b"RARAPP00");
        b[12..16].copy_from_slice(&256u32.to_le_bytes());b[16..32].fill(7);
        b[32..40].copy_from_slice(&0x1_0000_0001u64.to_le_bytes());
        b[40..44].copy_from_slice(&10u32.to_le_bytes());b[44..48].copy_from_slice(&3u32.to_le_bytes());
        b[48..56].copy_from_slice(&0x401000u64.to_le_bytes());
        for i in 0..3{b[56+i*8..64+i*8].copy_from_slice(&((1u64<<32)|(i as u64+1)).to_le_bytes());}
        b[96..100].copy_from_slice(&3u32.to_le_bytes());b[100..104].copy_from_slice(&1u32.to_le_bytes());
        b[112..120].copy_from_slice(&9u64.to_le_bytes());b[120..128].copy_from_slice(&10u64.to_le_bytes());b
    }
    #[test]fn bootstrap_exact_lengths_reserved_authority_and_full_identities(){
        let good=boot();let b=Boot::decode(&good).unwrap();assert_eq!(b.incarnation,0x1_0000_0001);
        for len in 0..256{assert!(Boot::decode(&good[..len]).is_err());}
        assert!(Boot::decode(&[0;257]).is_err());
        for at in (0..16).chain(80..96).chain(104..112).chain(128..256){
            let mut bad=good;bad[at]^=0x80;assert!(Boot::decode(&bad).is_err(),"{at}");
        }
        for at in [16usize,32,40,44,48,56,64,72,96,100,112,120]{
            let mut bad=good;
            let size=if at==16{16}else if [40,44,96,100].contains(&at){4}else{8};
            bad[at..at+size].fill(0);assert!(Boot::decode(&bad).is_err(),"{at}");
        }
    }
    #[test]fn bootstrap_encoding_revalidates_all_public_authority_fields(){
        let good=boot();let b=Boot::decode(&good).unwrap();
        assert_eq!(b.encode(),Ok(good));
        let mut bad=b;bad.incarnation=0;assert!(bad.encode().is_err());
        let mut bad=b;bad.principal=8;assert!(bad.encode().is_err());
        let mut bad=b;bad.rights=15;assert!(bad.encode().is_err());
        let mut bad=b;bad.caps[3]=0x1_0000_0004;assert!(bad.encode().is_err());
        let mut bad=b;bad.peer_incarnations[2]=1;assert!(bad.encode().is_err());
        let mut bad=b;bad.entry=0x420000;assert!(bad.encode().is_err());
        for rights in [1u32,3,5,7,9,11,13,15]{
            let mut value=b;value.rights=rights;
            for i in 1..5{
                value.caps[i]=if rights&(1<<(i-1))!=0{(1u64<<32)|(i as u64+1)}else{0};
            }
            for(i,principal)in [3u32,1,7,12].into_iter().enumerate(){
                let enabled=rights&(1<<i)!=0;
                value.peer_principals[i]=if enabled{principal}else{0};
                value.peer_incarnations[i]=if enabled{u64::MAX}else{0};
            }
            assert_eq!(Boot::decode(&value.encode().unwrap()),Ok(value));
        }
    }
    #[test]fn messages_are_bounded_canonical_and_correlated(){
        for op in 1..=6{for len in 0..=112{
            let m=Message::new(op,0,u32::MAX,&[7;112][..len]).unwrap();
            assert_eq!(Message::decode(&m.encode()),Ok(m));
        }}
        let request=Message::new(READ_DOCUMENT,0,7,b"").unwrap();
        let reply=Message::new(READ_DOCUMENT,0,7,b"hello").unwrap();
        assert!(reply.reply_to(&request,1,0x1_0000_0001,1,0x1_0000_0001));
        for id in [0,1,u64::MAX]{assert!(!reply.reply_to(&request,1,id,1,0x1_0000_0001));}
        assert!(!reply.reply_to(&request,2,0x1_0000_0001,1,0x1_0000_0001));
        for len in 0..128{assert!(Message::decode(&reply.encode()[..len]).is_err());}
        let good=reply.encode();
        for at in (0..5).chain(7..8).chain(14..16).chain(21..128){
            let mut bad=good;bad[at]^=1;assert!(Message::decode(&bad).is_err(),"{at}");
        }
        for op in [0,7,255]{assert!(Message::new(op,0,1,b"").is_err());}
        assert!(Message::new(1,8,1,b"").is_err());assert!(Message::new(1,0,0,b"").is_err());
        assert!(Message::new(1,0,1,&[0;113]).is_err());
    }
}
