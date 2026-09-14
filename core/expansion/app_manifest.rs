//! Experimental signed application envelope. Public lab key, not production trust.
#![forbid(unsafe_code)]
use crate::{sha256::sha256,ed25519,pe};
pub const SIZE:usize=512;
pub const UI:u32=1;
pub const DOCUMENT:u32=2;
pub const NETWORK:u32=4;
pub const AGENT:u32=8;
pub const LAB_PUBLIC_KEY:[u8;32]=[
0xd7,0x5a,0x98,0x01,0x82,0xb1,0x0a,0xb7,0xd5,0x4b,0xfe,0xd3,0xc9,0x64,0x07,0x3a,
0x0e,0xe1,0x72,0xf3,0xda,0xa6,0x23,0x25,0xaf,0x02,0x1a,0x68,0xf7,0x07,0x51,0x1a];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Framing,Compatibility,Publisher,Digest,Signature,Rollback,Budget,Executable }
fn hash(b:&[u8])->Result<[u8;32],Error>{sha256(b).map_err(|_|Error::Framing)}
fn u32_at(b:&[u8],at:usize)->u32{u32::from_le_bytes(b[at..at+4].try_into().unwrap())}
fn u64_at(b:&[u8],at:usize)->u64{u64::from_le_bytes(b[at..at+8].try_into().unwrap())}
#[derive(Clone,Copy)]
pub struct Manifest<'a>{bytes:&'a[u8;SIZE]}
impl<'a> Manifest<'a>{
    pub fn parse(raw:&'a[u8])->Result<Self,Error>{
        let b:&[u8;SIZE]=raw.try_into().map_err(|_|Error::Framing)?;
        if &b[..8]!=b"RARAPKG0"||u32_at(b,8)!=0||u32_at(b,12)!=512||
            b[128..416].iter().any(|&x|x!=0){return Err(Error::Framing);}
        if b[16..32]==[0;16]||u32_at(b,120)!=0||u32_at(b,124)!=0x8664{
            return Err(Error::Compatibility);
        }
        Ok(Self{bytes:b})
    }
    pub fn application_id(&self)->[u8;16]{self.bytes[16..32].try_into().unwrap()}
    pub fn generation(&self)->u64{u64_at(self.bytes,96)}
    pub fn rights(&self)->u32{u32_at(self.bytes,116)}
    pub fn image_budget(&self)->usize{u32_at(self.bytes,108)as usize}
    pub fn stack_bytes(&self)->usize{u32_at(self.bytes,112)as usize}
    pub fn signed_message(&self)->Result<[u8;49],Error>{
        let mut out=[0;49];out[..17].copy_from_slice(b"RAR-APP-ALPHA-V0\0");
        out[17..].copy_from_slice(&hash(&self.bytes[..416])?);Ok(out)
    }
    fn authenticate(&self)->Result<(),Error>{
        if self.bytes[32..64]!=hash(&LAB_PUBLIC_KEY)?{return Err(Error::Publisher);}
        if self.bytes[416..448]!=hash(&self.bytes[..416])?{return Err(Error::Digest);}
        let signature:[u8;64]=self.bytes[448..].try_into().map_err(|_|Error::Framing)?;
        if !ed25519::verify(&LAB_PUBLIC_KEY,&self.signed_message()?,&signature){return Err(Error::Signature);}
        Ok(())
    }
    fn policy(&self,minimum:u64,allowed:u32)->Result<(),Error>{
        if minimum==0||self.generation()<minimum{return Err(Error::Rollback);}
        if allowed&!15!=0||self.rights()&!allowed!=0||self.rights()&UI==0||
            self.image_budget()==0||self.image_budget()>pe::LIMIT||self.image_budget()%4096!=0||
            !matches!(self.stack_bytes(),16384|65536)||
            !(512..=pe::LIMIT).contains(&(u32_at(self.bytes,104)as usize)){
            return Err(Error::Budget);
        }
        Ok(())
    }
}
pub struct VerifiedApp<'a>{manifest:Manifest<'a>,payload:&'a[u8],layout:pe::Layout,owner:[u8;32]}
impl<'a> VerifiedApp<'a>{
    pub fn manifest(&self)->Manifest<'a>{self.manifest}
    pub fn payload(&self)->&'a[u8]{self.payload}
    pub fn layout(&self)->&pe::Layout{&self.layout}
    pub fn document_owner(&self)->[u8;32]{self.owner}
}
pub fn verify<'a>(raw:&'a[u8],payload:&'a[u8],minimum:u64,allowed:u32)->Result<VerifiedApp<'a>,Error>{
    let manifest=Manifest::parse(raw)?;
    manifest.authenticate()?;manifest.policy(minimum,allowed)?;
    if payload.len()!=u32_at(manifest.bytes,104)as usize||hash(payload)?!=manifest.bytes[64..96]{
        return Err(Error::Digest);
    }
    let layout=pe::parse(payload).map_err(|_|Error::Executable)?;
    if layout.image_size>manifest.image_budget(){return Err(Error::Budget);}
    let mut ownership=[0;65];ownership[..17].copy_from_slice(b"RAR-APP-OWNER-V0\0");
    ownership[17..49].copy_from_slice(&manifest.bytes[32..64]);
    ownership[49..].copy_from_slice(&manifest.application_id());
    Ok(VerifiedApp{manifest,payload,layout,owner:hash(&ownership)?})
}
#[cfg(test)]
mod tests{
    use super::*;
    fn fixture()->[u8;SIZE]{
        let mut b=[0;SIZE];b[..8].copy_from_slice(b"RARAPKG0");
        b[12..16].copy_from_slice(&512u32.to_le_bytes());b[16..32].fill(7);
        b[32..64].copy_from_slice(&hash(&LAB_PUBLIC_KEY).unwrap());
        b[96..104].copy_from_slice(&1u64.to_le_bytes());
        for(at,n)in [(104,1024u32),(108,8192),(112,16384),(116,UI|DOCUMENT),(124,0x8664)]{
            b[at..at+4].copy_from_slice(&n.to_le_bytes());
        }
        let h=hash(&b[..416]).unwrap();b[416..448].copy_from_slice(&h);b
    }
    #[test]fn framing_architecture_and_reserved_bytes(){
        let good=fixture();assert!(Manifest::parse(&good).is_ok());
        for len in 0..512{assert!(Manifest::parse(&good[..len]).is_err());}
        assert!(Manifest::parse(&[0;513]).is_err());
        for at in (0..16).chain(120..416){
            let mut bad=good;bad[at]^=0x80;assert!(Manifest::parse(&bad).is_err(),"{at}");
        }
        let mut bad=good;bad[16..32].fill(0);assert!(Manifest::parse(&bad).is_err());
    }
    #[test]fn no_unsigned_authority_and_separate_signature_domain(){
        let b=fixture();let m=Manifest::parse(&b).unwrap();
        assert_eq!(&m.signed_message().unwrap()[..17],b"RAR-APP-ALPHA-V0\0");
        assert!(matches!(verify(&b,&[0;1024],1,15),Err(Error::Signature)));
        for at in 16..120{
            let mut bad=b;bad[at]^=1;
            if let Ok(m)=Manifest::parse(&bad){assert!(m.authenticate().is_err());}
        }
    }
    #[test]fn owner_grants_generation_and_resource_ceilings(){
        let b=fixture();let m=Manifest::parse(&b).unwrap();
        assert_eq!(m.policy(1,3),Ok(()));
        for minimum in [0,2,u64::MAX]{assert_eq!(m.policy(minimum,15),Err(Error::Rollback));}
        for allowed in [0,UI,DOCUMENT,16,u32::MAX]{assert!(m.policy(1,allowed).is_err());}
        for(at,n)in [(104,0u32),(104,131073),(108,0),(108,4097),(108,131073),
                    (112,0),(112,8192),(112,131072),(116,0),(116,16),(116,u32::MAX)]{
            let mut bad=b;bad[at..at+4].copy_from_slice(&n.to_le_bytes());
            assert!(Manifest::parse(&bad).unwrap().policy(1,15).is_err());
        }
    }
}
