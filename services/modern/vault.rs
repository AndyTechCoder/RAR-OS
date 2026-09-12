//! Experimental bounded DataVault; no device authority or runtime activation.
//! Public laboratory keys only. See the exact DataVault-v0 contract.
#![forbid(unsafe_code)]
use crate::{chacha20poly1305 as aead, sha256::sha256};
pub type Sector = [u8; 512];
const MAX_SLOTS: u32 = 64;
const RESERVE: Sector = [0xa5; 512];
const COMMIT: Sector = [0xc3; 512];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Bounds, Io, Authentication, ReadOnly, Indeterminate }
pub trait Block {
    fn read(&mut self, sector: u32) -> Result<Sector, Error>;
    fn write(&mut self, sector: u32, bytes: &Sector) -> Result<(), Error>;
    fn flush(&mut self) -> Result<(), Error>;
}
fn hash(bytes: &[u8]) -> Result<[u8; 32], Error> { sha256(bytes).map_err(|_| Error::Bounds) }
fn u64_at(bytes: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(bytes[at..at+8].try_into().unwrap())
}
fn zero(bytes: &[u8]) -> bool { bytes.iter().all(|b| *b == 0) }
fn marker(bytes: &Sector, value: u8) -> bool {
    bytes.iter().all(|b| *b == 0 || *b == value) && bytes.contains(&value)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct File { name: [u8;12], name_len: u8, data: [u8;64], data_len: u8 }
impl File {
    const EMPTY: Self = Self { name:[0;12], name_len:0, data:[0;64], data_len:0 };
    fn name(&self) -> &[u8] { &self.name[..self.name_len as usize] }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Snapshot { files: [File;4], count: u8 }
impl Snapshot {
    pub const fn empty() -> Self { Self { files:[File::EMPTY;4], count:0 } }
    /// Bounded canonical order; exposes no mutable file internals.
    pub fn entries(&self) -> impl Iterator<Item=(&[u8], &[u8])> {
        self.files[..self.count as usize].iter().map(|f|
            (f.name(), &f.data[..f.data_len as usize]))
    }
    pub fn get(&self, name: &[u8]) -> Option<&[u8]> {
        self.files[..self.count as usize].iter().find(|f| f.name() == name)
            .map(|f| &f.data[..f.data_len as usize])
    }
    pub fn put(&mut self, name: &[u8], value: &[u8]) -> Result<(), Error> {
        if name.is_empty() || name.len()>12 || value.len()>64 ||
            !name.iter().all(|b| b.is_ascii_alphanumeric() || b"._-".contains(b)) {
            return Err(Error::Bounds);
        }
        let mut next = *self;
        let count = next.count as usize;
        let index = next.files[..count].iter().position(|f| f.name()==name).unwrap_or(count);
        if index >= 4 { return Err(Error::Bounds); }
        let mut file = File::EMPTY;
        file.name[..name.len()].copy_from_slice(name); file.name_len = name.len() as u8;
        file.data[..value.len()].copy_from_slice(value); file.data_len = value.len() as u8;
        next.files[index] = file;
        if index == count { next.count += 1; }
        let count = next.count as usize;
        if next.files[..count].iter().map(|f| f.data_len as usize).sum::<usize>()>128 {
            return Err(Error::Bounds);
        }
        next.files[..count].sort_unstable_by(|a,b| a.name().cmp(b.name()));
        *self = next; Ok(())
    }
    fn encode(&self) -> [u8;256] {
        let mut out=[0;256]; out[..8].copy_from_slice(b"RARDAT00"); out[8]=self.count;
        let mut offset=16;
        for file in &self.files[..self.count as usize] {
            out[offset]=file.name_len; out[offset+1]=file.data_len; offset+=2;
            let name=file.name(); out[offset..offset+name.len()].copy_from_slice(name); offset+=name.len();
            let value=&file.data[..file.data_len as usize];
            out[offset..offset+value.len()].copy_from_slice(value); offset+=value.len();
        }
        out
    }
    fn decode(raw: &[u8;256]) -> Result<Self, Error> {
        if &raw[..8]!=b"RARDAT00" || raw[8]>4 || !zero(&raw[9..16]) { return Err(Error::Invalid); }
        let mut result=Self::empty(); let mut at=16usize;
        for _ in 0..raw[8] {
            if at+2>256 { return Err(Error::Invalid); }
            let names=raw[at] as usize; let data=raw[at+1] as usize; at+=2;
            if names==0 || names>12 || data>64 || at+names+data>256 { return Err(Error::Invalid); }
            if result.get(&raw[at..at+names]).is_some() { return Err(Error::Invalid); }
            result.put(&raw[at..at+names], &raw[at+names..at+names+data])?;
            at+=names+data;
        }
        if result.encode()!=*raw { return Err(Error::Invalid); }
        Ok(result)
    }
}
/// Immutable public-lab descriptor, validated from two identical header sectors.
/// Does not grant a Block implementation any authority.
#[derive(Clone, Copy)]
pub struct Descriptor { header: Sector, key:[u8;32], prefix:[u8;4], slots:u32 }
impl Descriptor {
    pub fn decode(first: Sector, second: Sector, capacity:u32) -> Result<Self,Error> {
        if first!=second || &first[..8]!=b"RARVLT00" ||
            first[8..12]!=[0,0,0,2] || first[12..16]!=1u32.to_le_bytes() ||
            first[16..20]!=512u32.to_le_bytes() || first[24..32]!=2u64.to_le_bytes() ||
            zero(&first[32..64]) || zero(&first[64..96]) || !zero(&first[100..480]) ||
            hash(&first[..480])?!=first[480..512] {
            return Err(Error::Invalid);
        }
        let slots=u32::from_le_bytes(first[20..24].try_into().unwrap());
        if slots==0 || slots>MAX_SLOTS || capacity!=2+slots*3 { return Err(Error::Bounds); }
        Ok(Self { key:first[64..96].try_into().unwrap(), prefix:first[96..100].try_into().unwrap(),
                  header:first, slots })
    }
    fn nonce(&self, slot:u32) -> [u8;12] {
        let mut out=[0;12]; out[..4].copy_from_slice(&self.prefix);
        out[4..].copy_from_slice(&(slot as u64).to_le_bytes()); out
    }
    fn aad(&self, prefix:&[u8]) -> Result<[u8;112],Error> {
        let mut out=[0;112]; out[..16].copy_from_slice(b"RAR-VAULT-AAD-V0");
        out[16..48].copy_from_slice(&hash(&self.header)?); out[48..].copy_from_slice(prefix); Ok(out)
    }
}
pub struct Vault<B:Block> {
    block:B, descriptor:Descriptor, next:u32, revision:u64, digest:[u8;32],
    snapshot:Snapshot, readonly:bool,
}
impl<B:Block> Vault<B> {
    /// Reads all slots; never formats, repairs, writes or executes seal on mount.
    pub fn mount(mut block:B, capacity:u32) -> Result<Self,Error> {
        let descriptor=Descriptor::decode(block.read(0)?,block.read(1)?,capacity)?;
        let mut vault=Self { block, descriptor, next:descriptor.slots, revision:0,
            digest:[0;32], snapshot:Snapshot::empty(), readonly:false };
        let mut free=false;
        for slot in 0..descriptor.slots {
            let base=2+slot*3;
            let reserve=vault.block.read(base)?;
            let payload=vault.block.read(base+1)?;
            let commit=vault.block.read(base+2)?;
            if zero(&reserve) && zero(&payload) && zero(&commit) {
                if !free { vault.next=slot; free=true; }
                continue;
            }
            if free { return Err(Error::Invalid); }
            if reserve!=RESERVE {
                if marker(&reserve,0xa5) && zero(&payload) && zero(&commit) { continue; }
                return Err(Error::Invalid);
            }
            if zero(&commit) { continue; } // Slot consumed, never resealed.
            if !marker(&commit,0xc3) { return Err(Error::Invalid); }
            let snapshot=vault.open_payload(slot,&payload)?;
            vault.snapshot=snapshot; vault.revision+=1; vault.digest=hash(&payload)?;
        }
        vault.readonly=vault.next==descriptor.slots;
        Ok(vault)
    }
    fn open_payload(&self,slot:u32,payload:&Sector)->Result<Snapshot,Error> {
        let revision=self.revision.checked_add(1).ok_or(Error::Bounds)?;
        if &payload[..8]!=b"RARVPY00" || u64_at(payload,8)!=slot as u64 ||
            u64_at(payload,16)!=revision || payload[24..56]!=self.digest ||
            !zero(&payload[56..64]) || !zero(&payload[336..]) { return Err(Error::Invalid); }
        let aad=self.descriptor.aad(&payload[..64])?;
        let mut plain:[u8;256]=payload[64..320].try_into().unwrap();
        let tag:[u8;16]=payload[320..336].try_into().unwrap();
        aead::open(&self.descriptor.key,&self.descriptor.nonce(slot),&aad,&mut plain,&tag)
            .map_err(|_|Error::Authentication)?;
        Snapshot::decode(&plain)
    }
    pub fn snapshot(&self)->&Snapshot { &self.snapshot }
    pub fn revision(&self)->u64 { self.revision }
    pub fn is_readonly(&self)->bool { self.readonly }
    pub fn into_block(self)->B { self.block }
    fn durable(&mut self,sector:u32,bytes:&Sector)->Result<(),Error> {
        self.block.write(sector,bytes)?; self.block.flush()?;
        if self.block.read(sector)?!=*bytes { return Err(Error::Io); }
        Ok(())
    }
    /// After any I/O/verification failure, no further writes this boot.
    /// An error during publication may still have committed; never assume not-applied.
    pub fn publish(&mut self,snapshot:Snapshot)->Result<u64,Error> {
        if self.readonly { return Err(Error::ReadOnly); }
        // Lock before I/O. Only completely verified success unlocks the session.
        self.readonly=true;
        let slot=self.next; let base=2+slot*3;
        let revision=self.revision.checked_add(1).ok_or(Error::Bounds)?;
        for sector in base..base+3 {
            if !zero(&self.block.read(sector)?) { return Err(Error::Invalid); }
        }
        self.durable(base,&RESERVE)?;
        // Exactly one seal per physical slot, after durable all-byte reservation.
        // Recovery consumes reservations without ever invoking this operation.
        let mut payload=[0;512]; payload[..8].copy_from_slice(b"RARVPY00");
        payload[8..16].copy_from_slice(&(slot as u64).to_le_bytes());
        payload[16..24].copy_from_slice(&revision.to_le_bytes());
        payload[24..56].copy_from_slice(&self.digest);
        payload[64..320].copy_from_slice(&snapshot.encode());
        let aad=self.descriptor.aad(&payload[..64])?;
        let tag=aead::seal(&self.descriptor.key,&self.descriptor.nonce(slot),&aad,&mut payload[64..320])
            .map_err(|_|Error::Bounds)?;
        payload[320..336].copy_from_slice(&tag);
        self.durable(base+1,&payload)?;
        self.durable(base+2,&COMMIT).map_err(|_|Error::Indeterminate)?;
        self.snapshot=snapshot; self.revision=revision; self.digest=hash(&payload)?;
        self.next+=1; self.readonly=self.next==self.descriptor.slots;
        Ok(revision)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone)]
    struct Disk {
        live:Vec<Sector>, stable:Vec<Sector>, calls:usize,
        fail:Option<usize>, tear:usize, flush_after:bool,
    }
    impl Disk {
        fn fixture()->Self {
            let mut h=[0;512]; h[..8].copy_from_slice(b"RARVLT00");
            h[8..12].copy_from_slice(&[0,0,0,2]); h[12..16].copy_from_slice(&1u32.to_le_bytes());
            h[16..20].copy_from_slice(&512u32.to_le_bytes()); h[20..24].copy_from_slice(&4u32.to_le_bytes());
            h[24..32].copy_from_slice(&2u64.to_le_bytes()); h[32..64].fill(7);
            h[64..96].fill(9); h[96..100].copy_from_slice(&[1,2,3,4]);
            let digest=hash(&h[..480]).unwrap(); h[480..].copy_from_slice(&digest);
            let mut sectors=vec![[0;512];14]; sectors[0]=h; sectors[1]=h;
            Self { live:sectors.clone(),stable:sectors,calls:0,fail:None,tear:0,flush_after:false }
        }
        fn hit(&mut self)->bool { self.calls+=1; self.fail==Some(self.calls) }
        fn crash(mut self)->Self {
            self.live=self.stable.clone(); self.fail=None; self.calls=0; self
        }
    }
    impl Block for Disk {
        fn read(&mut self,sector:u32)->Result<Sector,Error> {
            if self.hit(){return Err(Error::Io);}
            self.live.get(sector as usize).copied().ok_or(Error::Bounds)
        }
        fn write(&mut self,sector:u32,bytes:&Sector)->Result<(),Error> {
            let fail=self.hit();
            let at=sector as usize;
            if at>=self.live.len(){return Err(Error::Bounds);}
            if fail {
                self.live[at][..self.tear].copy_from_slice(&bytes[..self.tear]);
                self.stable[at][..self.tear].copy_from_slice(&bytes[..self.tear]);
                Err(Error::Io)
            } else {self.live[at]=*bytes;Ok(())}
        }
        fn flush(&mut self)->Result<(),Error> {
            let fail=self.hit();
            if !fail || self.flush_after {self.stable=self.live.clone();}
            if fail {Err(Error::Io)} else {Ok(())}
        }
    }
    fn sample()->Snapshot {
        let mut s=Snapshot::empty(); s.put(b"note",b"new value").unwrap(); s
    }
    #[test] fn canonical_snapshot_and_limits() {
        let mut s=Snapshot::empty(); s.put(b"b",b"2").unwrap();s.put(b"a",b"1").unwrap();
        assert_eq!(Snapshot::decode(&s.encode()).unwrap(),s);
        assert_eq!(s.get(b"a"),Some(&b"1"[..]));
        for (name,data) in [(&b""[..],&b"x"[..]),(&b"../x"[..],&b"x"[..]),
                            (&b"a"[..],&[0;65][..])] {
            let before=s; assert!(s.put(name,data).is_err()); assert_eq!(s,before);
        }
        let mut large=Snapshot::empty();
        large.put(b"a",&[1;64]).unwrap();large.put(b"b",&[2;64]).unwrap();
        let before=large;assert!(large.put(b"c",b"x").is_err());assert_eq!(large,before);
        let mut raw=s.encode();raw[255]=1;assert!(Snapshot::decode(&raw).is_err());
        for byte in 0..256 {
            let mut raw=s.encode();raw[byte]^=0x80;
            let _=Snapshot::decode(&raw); // Bounded malformed corpus must not panic.
        }
    }
    #[test] fn durable_roundtrip_and_exhaustion() {
        let disk=Disk::fixture(); let header=[disk.live[0],disk.live[1]];
        let mut vault=Vault::mount(disk,14).unwrap();
        for revision in 1..=4 {
            assert_eq!(vault.publish(sample()).unwrap(),revision);
            let disk=vault.into_block().crash();
            assert_eq!([disk.live[0],disk.live[1]],header);
            vault=Vault::mount(disk,14).unwrap();
            assert_eq!(vault.revision(),revision);assert_eq!(*vault.snapshot(),sample());
        }
        assert!(vault.is_readonly()); assert_eq!(vault.publish(sample()),Err(Error::ReadOnly));
    }
    fn fault_case(operation:usize,tear:usize,after:bool) {
        let mut vault=Vault::mount(Disk::fixture(),14).unwrap();
        vault.block.calls=0;vault.block.fail=Some(operation);
        vault.block.tear=tear;vault.block.flush_after=after;
        assert!(vault.publish(sample()).is_err());
        assert!(vault.is_readonly());
        let calls=vault.block.calls;let bytes=vault.block.live.clone();
        assert_eq!(vault.publish(sample()),Err(Error::ReadOnly));
        assert_eq!(vault.block.calls,calls);assert_eq!(vault.block.live,bytes);
        let mut recovered=Vault::mount(vault.into_block().crash(),14).unwrap();
        assert!(recovered.revision()==0 || recovered.revision()==1);
        assert_eq!(*recovered.snapshot(),if recovered.revision()==0 {Snapshot::empty()} else {sample()});
        // Every consumed reservation stays untouched on the subsequent write.
        let consumed=recovered.next;
        let prefix=recovered.block.live[..(2+consumed*3) as usize].to_vec();
        let expected_revision=recovered.revision()+1;
        let mut following=sample();
        following.put(b"note",b"after recovery").unwrap();
        following.put(b"next",b"second boot").unwrap();
        assert_eq!(recovered.publish(following),Ok(expected_revision));
        assert_eq!(recovered.block.live[..prefix.len()],prefix);
        // Crash again after appending beyond any burned slot. Old/new selection
        // from the first recovery is not sufficient proof of a valid new chain.
        let again=Vault::mount(recovered.into_block().crash(),14).unwrap();
        assert_eq!(again.revision(),expected_revision);
        assert_eq!(*again.snapshot(),following);
        assert_eq!(again.block.live[..prefix.len()],prefix);
    }
    #[test] fn every_io_failure_and_flush_outcome_recovers_old_or_new() {
        for operation in 1..=12 {
            for tear in [0,1,255,511,512] {
                for after in [false,true] { fault_case(operation,tear,after); }
            }
        }
    }
    #[test] fn every_prefix_of_each_sector_write_recovers_without_slot_reuse() {
        for operation in [4,7,10] {
            for tear in 0..=512 { fault_case(operation,tear,false); }
        }
    }
    #[test] fn corruption_holes_forks_and_header_mismatch_fail_closed() {
        let mut vault=Vault::mount(Disk::fixture(),14).unwrap();
        vault.publish(sample()).unwrap();let disk=vault.into_block().crash();
        for (sector,offset) in [(0,100),(2,2),(3,100),(4,2)] {
            let mut bad=disk.clone();bad.live[sector][offset]^=0x40;
            assert!(Vault::mount(bad,14).is_err());
        }
        let mut hole=Disk::fixture();hole.live[5]=RESERVE;
        assert!(Vault::mount(hole,14).is_err());
        let mut conflict=Disk::fixture();conflict.live[2][0]=0xa5;conflict.live[3][0]=1;
        assert!(Vault::mount(conflict,14).is_err());
        let mut fork=disk.clone();
        fork.live[5]=RESERVE;fork.live[6]=fork.live[3];fork.live[7]=COMMIT;
        assert!(Vault::mount(fork,14).is_err());
        assert!(Vault::mount(Disk::fixture(),13).is_err());
    }
}
