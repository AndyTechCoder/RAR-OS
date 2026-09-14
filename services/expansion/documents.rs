//! Candidate private-document policy over unchanged DataVault-v0 snapshots.
//! No I/O, caller authentication, mount mutation, or execution grant.
#![forbid(unsafe_code)]
use crate::vault::Snapshot;
pub const MAX_DOCUMENT:usize=64;
const OWNER_KEY:&[u8]=b"_rar.app0";
const DOCUMENT_KEY:&[u8]=b"_rar.doc0";
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Invalid,Denied,Quota }
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Owner([u8;32]);
impl Owner {
    /// Trusted installer input only, derived from a verified publisher/app ID.
    /// This value is not an unforgeable capability and must never come from IPC.
    pub fn from_verified_identity(digest:[u8;32])->Result<Self,Error>{
        if digest==[0;32]{Err(Error::Invalid)}else{Ok(Self(digest))}
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Grant{owner:Owner,principal:u32,incarnation:u64}
impl Grant{
    /// Only the trusted policy broker may construct grants; service callers
    /// cannot supply this structure or choose their own stamped identity.
    pub fn new(owner:Owner,principal:u32,incarnation:u64)->Result<Self,Error>{
        if !(10..=13).contains(&principal)||incarnation==0{return Err(Error::Invalid);}
        Ok(Self{owner,principal,incarnation})
    }
    pub fn validate_binding(self,snapshot:&Snapshot)->Result<(),Error>{
        match inspect(snapshot)?{
            None=>Ok(()),Some(owner)if owner==self.owner=>Ok(()),Some(_)=>Err(Error::Denied)
        }
    }
    pub fn installation(self,snapshot:&Snapshot)->Result<Snapshot,Error>{install(snapshot,self.owner)}
    fn authorize(self,principal:u32,incarnation:u64)->Result<Owner,Error>{
        if principal!=self.principal||incarnation!=self.incarnation{return Err(Error::Denied);}
        Ok(self.owner)
    }
}
fn shared_name(name:&[u8])->bool{
    !name.is_empty()&&name!=b"."&&name!=b".."&&
        name.iter().all(|b|b.is_ascii_alphanumeric()||b".-".contains(b))
}
fn inspect(snapshot:&Snapshot)->Result<Option<Owner>,Error>{
    for(name,_)in snapshot.entries(){
        if name!=OWNER_KEY&&name!=DOCUMENT_KEY&&!shared_name(name){return Err(Error::Invalid);}
    }
    match(snapshot.get(OWNER_KEY),snapshot.get(DOCUMENT_KEY)){
        (None,None)=>Ok(None),
        (Some(raw),Some(doc))if raw.len()==32&&doc.len()<=MAX_DOCUMENT&&
            snapshot.entries().filter(|(n,_)|shared_name(n)).map(|(_,v)|v.len()).sum::<usize>()<=32=>{
            Ok(Some(Owner::from_verified_identity(raw.try_into().map_err(|_|Error::Invalid)?)?))
        },
        _=>Err(Error::Invalid),
    }
}
/// One candidate snapshot preserves all shared records. The caller must publish
/// exactly this snapshot through the existing Vault transaction, not two writes.
pub fn install(snapshot:&Snapshot,owner:Owner)->Result<Snapshot,Error>{
    match inspect(snapshot)?{
        Some(current)if current==owner=>return Ok(*snapshot),
        Some(_)=>return Err(Error::Denied),
        None=>(),
    }
    if snapshot.entries().map(|(_,v)|v.len()).sum::<usize>()>32{return Err(Error::Quota);}
    let mut next=*snapshot;
    next.put(OWNER_KEY,&owner.0).map_err(|_|Error::Quota)?;
    next.put(DOCUMENT_KEY,b"").map_err(|_|Error::Quota)?;
    Ok(next)
}
pub fn read(snapshot:&Snapshot,grant:Grant,principal:u32,incarnation:u64)->Result<&[u8],Error>{
    let owner=grant.authorize(principal,incarnation)?;
    if inspect(snapshot)?!=Some(owner){return Err(Error::Denied);}
    snapshot.get(DOCUMENT_KEY).ok_or(Error::Invalid)
}
pub fn write(snapshot:&Snapshot,grant:Grant,principal:u32,incarnation:u64,data:&[u8])->Result<Snapshot,Error>{
    read(snapshot,grant,principal,incarnation)?;
    if data.len()>MAX_DOCUMENT{return Err(Error::Quota);}
    let mut next=*snapshot;next.put(DOCUMENT_KEY,data).map_err(|_|Error::Quota)?;Ok(next)
}
/// Shared Files projection must be used by the future integrated service.
pub fn shared(snapshot:&Snapshot)->Result<Snapshot,Error>{
    inspect(snapshot)?;
    let mut out=Snapshot::empty();
    for(name,data)in snapshot.entries(){
        if shared_name(name){out.put(name,data).map_err(|_|Error::Invalid)?;}
    }
    Ok(out)
}
#[cfg(test)]
mod tests{
    use super::*;
    fn owner()->Owner{Owner::from_verified_identity([7;32]).unwrap()}
    fn grant()->Grant{Grant::new(owner(),10,0x1_0000_0001).unwrap()}
    #[test]fn install_is_atomic_preserves_shared_and_separates_ownership(){
        let mut old=Snapshot::empty();old.put(b"note",b"shared").unwrap();
        let installed=install(&old,owner()).unwrap();
        assert_eq!(shared(&installed),Ok(old));assert_eq!(old.get(OWNER_KEY),None);
        assert_eq!(read(&installed,grant(),10,0x1_0000_0001),Ok(&b""[..]));
        assert_eq!(install(&installed,owner()),Ok(installed));
        assert_eq!(install(&installed,Owner([8;32])),Err(Error::Denied));
        let edited=write(&installed,grant(),10,0x1_0000_0001,b"private").unwrap();
        assert_eq!(shared(&edited),Ok(old));
        assert_eq!(read(&edited,grant(),10,0x1_0000_0001),Ok(&b"private"[..]));
    }
    #[test]fn caller_cannot_select_other_owner_or_truncate_incarnation(){
        let snapshot=install(&Snapshot::empty(),owner()).unwrap();
        for principal in 0..16{
            if principal!=10{assert_eq!(read(&snapshot,grant(),principal,0x1_0000_0001),Err(Error::Denied));}
        }
        for inc in [0,1,0x1_0000_0000,u64::MAX]{
            assert_eq!(write(&snapshot,grant(),10,inc,b"x"),Err(Error::Denied));
        }
        let other=Grant::new(Owner([8;32]),10,0x1_0000_0001).unwrap();
        assert_eq!(read(&snapshot,other,10,0x1_0000_0001),Err(Error::Denied));
        assert!(Owner::from_verified_identity([0;32]).is_err());
        for principal in [0,9,14,u32::MAX]{assert!(Grant::new(owner(),principal,1).is_err());}
        assert!(Grant::new(owner(),10,0).is_err());
    }
    #[test]fn quotas_refuse_without_changing_source(){
        let mut old=Snapshot::empty();old.put(b"a",&[1;64]).unwrap();old.put(b"b",&[2;40]).unwrap();
        let before=old;assert_eq!(install(&old,owner()),Err(Error::Quota));assert_eq!(old,before);
        let empty=install(&Snapshot::empty(),owner()).unwrap();
        assert_eq!(write(&empty,grant(),10,0x1_0000_0001,&[0;65]),Err(Error::Quota));
        let full=write(&empty,grant(),10,0x1_0000_0001,&[1;64]).unwrap();
        assert_eq!(read(&full,grant(),10,0x1_0000_0001),Ok(&[1;64][..]));
        assert_eq!(read(&empty,grant(),10,0x1_0000_0001),Ok(&b""[..]));
        let mut slots=Snapshot::empty();
        for name in [b"a",b"b",b"c"]{slots.put(name,b"").unwrap();}
        assert_eq!(install(&slots,owner()),Err(Error::Quota));
    }
    #[test]fn corrupt_or_unknown_private_namespace_is_never_projected(){
        for name in [&b"_unknown"[..],&b"."[..],&b".."[..]]{
            let mut s=Snapshot::empty();s.put(name,b"x").unwrap();
            assert_eq!(shared(&s),Err(Error::Invalid));assert_eq!(install(&s,owner()),Err(Error::Invalid));
        }
        for(name,value)in [(OWNER_KEY,&[7;32][..]),(DOCUMENT_KEY,&b""[..])]{
            let mut s=Snapshot::empty();s.put(name,value).unwrap();
            assert_eq!(shared(&s),Err(Error::Invalid));
        }
        for len in 0..=64{
            if len==32{continue;}
            let mut s=Snapshot::empty();s.put(OWNER_KEY,&[7;64][..len]).unwrap();s.put(DOCUMENT_KEY,b"").unwrap();
            assert_eq!(shared(&s),Err(Error::Invalid));
        }
    }

    #[test]fn installation_reserves_the_full_document_at_shared_32_33_boundary(){
        let mut fits=Snapshot::empty();fits.put(b"shared",&[1;32]).unwrap();
        let installed=install(&fits,owner()).unwrap();
        let full=write(&installed,grant(),10,0x1_0000_0001,&[2;64]).unwrap();
        assert_eq!(shared(&full),Ok(fits));
        let mut too_much=Snapshot::empty();too_much.put(b"shared",&[1;33]).unwrap();
        let before=too_much;assert_eq!(install(&too_much,owner()),Err(Error::Quota));
        assert_eq!(too_much,before);
        let mut incompatible=installed;incompatible.put(b"shared",&[1;33]).unwrap();
        assert_eq!(shared(&incompatible),Err(Error::Invalid));
    }
}
