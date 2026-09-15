//! Modern-only durable adapter for the existing Terminal/Files request bytes.
//! No VM activation or device authority. Runtime must provide only a Data Block.
#![forbid(unsafe_code)]
#[path="../expansion/app_sdk.rs"]pub mod app_sdk;

use crate::vault::{Block, Error, Snapshot, Vault};
use crate::desktop_wire as wire;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure { ReadOnly, Unavailable, Indeterminate }

pub struct Store<B: Block> {
    vault: Vault<B>,
    unavailable: bool,
    files_generation: u64,
    terminal_generation: u64,
    private: Option<crate::app_documents::Grant>,
    app_sequence:u32,
    private_owner:Option<crate::app_documents::Owner>,private_principal:u32,private_highest:u64,
}
pub(crate) fn name_ok(name: &[u8]) -> bool {
    !name.is_empty() && name.len() <= 12 && name != b"." && name != b".." &&
        name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-')
}

pub(crate) fn decode(request: &[u8]) -> Option<(u8, &[u8], &[u8])> {
    if request.len() != 128 { return None; }
    let op=request[0]; let names=request[1] as usize; let data=request[2] as usize;
    if !matches!(op,wire::CREATE|wire::WRITE|wire::READ|wire::LIST) ||
        names>12 || data>64 || request[3]!=0 ||
        request[4+names..16].iter().any(|b|*b!=0) ||
        request[16+data..].iter().any(|b|*b!=0) || (op!=wire::WRITE && data!=0) {
        return None;
    }
    let name=&request[4..4+names];
    if (op==wire::LIST && names!=0) || (op!=wire::LIST && !name_ok(name)) { return None; }
    Some((op,name,&request[16..16+data]))
}

fn status(value: u8) -> [u8; 128] { let mut out = [0;128]; out[0] = value; out }

impl<B: Block> Store<B> {
    /// Mount only: no welcome-file creation, formatting or write on boot.
    pub fn mount(block: B, capacity: u32, files_generation: u64, terminal_generation: u64)
        -> Result<Self, Error> {
        if files_generation == 0 || terminal_generation == 0 { return Err(Error::Invalid); }
        let vault = Vault::mount(block, capacity)?;
        // Preserve the existing app namespace; never silently omit unsupported
        // entries or rewrite a valid but incompatible snapshot.
        if vault.snapshot().entries().any(|(name, _)| !name_ok(name)) {
            return Err(Error::Invalid);
        }
        Ok(Self { vault, unavailable: false, files_generation, terminal_generation, private: None,app_sequence:0,private_owner:None,private_principal:0,private_highest:0 })
    }
    /// Candidate private adapter. Grant comes from trusted installer/kernel
    /// identity, never app bytes. Reads and validates; never installs on mount.
    pub fn mount_private(block:B,capacity:u32,files_generation:u64,terminal_generation:u64,
        grant:crate::app_documents::Grant)->Result<Self,Error>{
        if files_generation==0||terminal_generation==0{return Err(Error::Invalid);}
        let vault=Vault::mount(block,capacity)?;
        grant.validate_binding(vault.snapshot()).map_err(|_|Error::Invalid)?;
        // Check the complete future installation without publishing it.
        grant.installation(vault.snapshot()).map_err(|_|Error::Bounds)?;
        Ok(Self{vault,unavailable:false,files_generation,terminal_generation,private:Some(grant),app_sequence:0,
            private_owner:Some(grant.owner()),private_principal:grant.principal(),private_highest:grant.incarnation()})
    }
    /// App-aware native composition can mount before Notes exists. Owner is
    /// fixed verified package identity, not an IPC field. No live app grant,
    /// private record install, write, format or sequence reset occurs on mount.
    pub fn mount_applications(block:B,capacity:u32,files_generation:u64,terminal_generation:u64,
        owner:crate::app_documents::Owner)->Result<Self,Error>{
        if files_generation==0||terminal_generation==0{return Err(Error::Invalid);}
        let vault=Vault::mount(block,capacity)?;
        owner.validate_binding(vault.snapshot()).map_err(|_|Error::Invalid)?;
        crate::app_documents::install(vault.snapshot(),owner).map_err(|_|Error::Bounds)?;
        Ok(Self{vault,unavailable:false,files_generation,terminal_generation,private:None,app_sequence:0,
            private_owner:Some(owner),private_principal:10,private_highest:0})
    }
    /// Trusted kernel-binding transition only. Revoke first; validate exact
    /// owner/principal and a strictly newer full incarnation before any mutation.
    /// Old queued replies must be rechecked with app_reply_current before send.
    pub fn rebind_private(&mut self,grant:crate::app_documents::Grant)->Result<(),Failure>{
        if self.unavailable||self.private.is_some()||self.private_owner!=Some(grant.owner())||
            self.private_principal!=grant.principal()||grant.incarnation()<=self.private_highest{
            return Err(Failure::Unavailable);
        }
        grant.validate_binding(self.vault.snapshot()).map_err(|_|Failure::Unavailable)?;
        grant.installation(self.vault.snapshot()).map_err(|_|Failure::Unavailable)?;
        self.private=Some(grant);self.private_highest=grant.incarnation();self.app_sequence=0;Ok(())
    }
    /// Envelope identity must have been retained with the reply, not supplied
    /// by its recipient. The native sender must also compare the live kernel
    /// binding immediately before using its fixed app-send handle.
    pub fn app_reply_current(&self,principal:u64,incarnation:u64)->bool{
        u32::try_from(principal).ok().is_some_and(|principal|
            self.private.is_some_and(|g|g.allows(principal,incarnation)))
    }
    /// Trusted explicit installer call; no caller-controlled owner or filesystem
    /// selector. A future IPC adapter must authenticate installer authority.
    pub fn install_private(&mut self)->Result<(),Failure>{
        let grant=self.private.ok_or(Failure::Unavailable)?;
        if self.unavailable{return Err(Failure::Unavailable);}
        let candidate=grant.installation(self.vault.snapshot())
            .map_err(|_|Failure::Unavailable)?;
        if candidate==*self.vault.snapshot(){return Ok(());}
        self.publish_private(candidate)
    }
    fn publish_private(&mut self,candidate:Snapshot)->Result<(),Failure>{
        if self.unavailable{return Err(Failure::Unavailable);}
        if self.vault.is_readonly(){return Err(Failure::ReadOnly);}
        match self.vault.publish(candidate){
            Ok(_)=>Ok(()),
            Err(Error::ReadOnly)=>Err(Failure::ReadOnly),
            Err(error)=>{
                self.unavailable=true;
                Err(if error==Error::Indeterminate{Failure::Indeterminate}else{Failure::Unavailable})
            }
        }
    }
    pub fn read_private(&self,principal:u32,incarnation:u64)->Result<&[u8],Failure>{
        if self.unavailable{return Err(Failure::Unavailable);}
        let grant=self.private.ok_or(Failure::Unavailable)?;
        crate::app_documents::read(self.vault.snapshot(),grant,principal,incarnation)
            .map_err(|_|Failure::Unavailable)
    }
    pub fn write_private(&mut self,principal:u32,incarnation:u64,value:&[u8])->Result<(),Failure>{
        if self.unavailable{return Err(Failure::Unavailable);}
        let grant=self.private.ok_or(Failure::Unavailable)?;
        let candidate=crate::app_documents::write(self.vault.snapshot(),grant,principal,incarnation,value)
            .map_err(|_|Failure::Unavailable)?;
        if candidate==*self.vault.snapshot(){return Ok(());}
        self.publish_private(candidate)
    }
    /// Candidate private app endpoint. The runtime must supply actual kernel
    /// sender/incarnation, then route the returned frame using its own fixed
    /// app-send grant. No app-selected owner, path, handle or install operation.
    pub fn process_app(&mut self,sender:u64,incarnation:u64,request:&[u8])->Option<[u8;128]>{
        use app_sdk::{wire::Message,protocol::{self,Document}};
        let principal=u32::try_from(sender).ok()?;
        if !self.private?.allows(principal,incarnation){return None;}
        let message=Message::decode(request).ok()?;
        let operation=protocol::document(&message).ok()?;
        if message.sequence()<=self.app_sequence{return None;}
        // Consume before I/O. Replays cannot repeat a possibly committed write.
        self.app_sequence=message.sequence();
        let mut payload=[0u8;64];let mut len=0;
        let result=match operation{
            Document::Read=>self.read_private(principal,incarnation).map(|bytes|{
                len=bytes.len();payload[..len].copy_from_slice(bytes);
            }),
            Document::Write(value)=>self.write_private(principal,incarnation,value),
        };
        let status=match result{
            Ok(())=>protocol::OK,Err(Failure::ReadOnly)=>protocol::READ_ONLY,
            Err(Failure::Unavailable)=>protocol::UNAVAILABLE,
            Err(Failure::Indeterminate)=>protocol::INDETERMINATE,
        };
        Message::new(message.operation(),status,message.sequence(),&payload[..len]).ok().map(|m|m.encode())
    }
    pub fn revoke_private(&mut self){self.private=None;}
    pub fn into_block(self) -> B { self.vault.into_block() }
    pub fn revision(&self) -> u64 { self.vault.revision() }
    pub(crate) fn authorized(&self,sender:u64,generation:u64)->bool {
        match sender {4=>generation==self.files_generation,
            6=>generation==self.terminal_generation,_=>false}
    }

    /// Sender/generation must be copied from the kernel receive envelope.
    /// Mount's expected incarnations come from trusted Modern bootstrap grants,
    /// never request bytes. Recreate this adapter after grant/incarnation changes.
    /// A successful mutation reply is produced ONLY after Vault's durable ACK.
    /// Failures are private typed results, not new values in the old wire ABI;
    /// the Modern runtime must display them explicitly and never synthesize OK.
    pub fn process(&mut self, sender: u64, generation: u64, request: &[u8])
        -> Result<[u8;128], Failure>
    {
        if !self.authorized(sender,generation) { return Ok(status(wire::INVALID)); }
        let Some((op,name,value))=decode(request) else { return Ok(status(wire::INVALID)); };
        if self.unavailable { return Err(Failure::Unavailable); }
        let snapshot = *self.vault.snapshot();
        let visible = if self.private.is_some(){
            crate::app_documents::shared(&snapshot).map_err(|_|Failure::Unavailable)?
        }else{
            // A revoked private grant does not expose its records to Files.
            if snapshot.entries().any(|(name,_)|!name_ok(name)){
                crate::app_documents::shared(&snapshot).map_err(|_|Failure::Unavailable)?
            }else{snapshot}
        };
        match op {
            wire::READ => {
                let Some(value) = visible.get(name) else { return Ok(status(wire::NOT_FOUND)); };
                let mut reply = [0;128];
                reply[2] = value.len() as u8;
                reply[16..16+value.len()].copy_from_slice(value);
                Ok(reply)
            }
            wire::LIST => {
                let mut reply = [0;128];
                let mut at = 4;
                for (name, _) in visible.entries() {
                    reply[at] = name.len() as u8; at += 1;
                    reply[at..at+name.len()].copy_from_slice(name); at += name.len();
                    reply[1] += 1;
                }
                Ok(reply)
            }
            wire::CREATE | wire::WRITE => {
                let exists = snapshot.get(name).is_some();
                if op == wire::CREATE && exists { return Ok(status(wire::EXISTS)); }
                if op == wire::WRITE && !exists { return Ok(status(wire::NOT_FOUND)); }
                // A verified already-committed value needs no new nonce/slot.
                if op == wire::WRITE && snapshot.get(name) == Some(value) {
                    return Ok(status(wire::OK));
                }
                if self.vault.is_readonly() { return Err(Failure::ReadOnly); }
                let mut candidate: Snapshot = snapshot;
                if candidate.put(name, value).is_err() ||
                    self.private_owner.is_some_and(|owner|crate::app_documents::install(&candidate,owner).is_err()) ||
                    (snapshot.entries().any(|(n,_)|!name_ok(n))&&
                     crate::app_documents::shared(&candidate).is_err()) {
                    return Ok(status(wire::QUOTA));
                }
                match self.vault.publish(candidate) {
                    Ok(_) => Ok(status(wire::OK)),
                    Err(Error::ReadOnly) => Err(Failure::ReadOnly),
                    Err(error) => {
                        // Even on a failed request, disk publication can have
                        // taken effect. Do not show cached bytes as authoritative,
                        // retry the mutation, or remount/format automatically.
                        self.unavailable = true;
                        Err(if error == Error::Indeterminate {
                            Failure::Indeterminate
                        } else { Failure::Unavailable })
                    }
                }
            }
            _ => Ok(status(wire::INVALID)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{sha256::sha256, vault::Sector};
    use std::{cell::Cell,rc::Rc};
    #[derive(Clone)]
    struct Disk { live: Vec<Sector>, durable: Vec<Sector>, calls: usize, fail: Option<usize>, writes: Rc<Cell<usize>> }
    impl Disk {
        fn new(slots: u32) -> Self {
            let mut header = [0;512];
            header[..8].copy_from_slice(b"RARVLT00"); header[8..12].copy_from_slice(&[0,0,0,2]);
            header[12..16].copy_from_slice(&1u32.to_le_bytes());
            header[16..20].copy_from_slice(&512u32.to_le_bytes());
            header[20..24].copy_from_slice(&slots.to_le_bytes());
            header[24..32].copy_from_slice(&2u64.to_le_bytes());
            header[32..64].fill(7); header[64..96].fill(9);
            header[96..100].copy_from_slice(&[1,2,3,4]);
            let hash = sha256(&header[..480]).unwrap(); header[480..].copy_from_slice(&hash);
            let mut live = vec![[0;512]; (2+slots*3) as usize];
            live[0] = header; live[1] = header;
            Self { durable: live.clone(), live, calls: 0, fail: None, writes: Rc::new(Cell::new(0)) }
        }
        fn hit(&mut self) -> bool { self.calls += 1; self.fail == Some(self.calls) }
        fn reboot(mut self) -> Self {
            self.live = self.durable.clone(); self.calls = 0; self.fail = None; self
        }
    }
    impl Block for Disk {
        fn read(&mut self, sector: u32) -> Result<Sector, Error> {
            if self.hit() { return Err(Error::Io); }
            self.live.get(sector as usize).copied().ok_or(Error::Bounds)
        }
        fn write(&mut self, sector: u32, bytes: &Sector) -> Result<(), Error> {
            self.writes.set(self.writes.get()+1);
            if self.hit() { return Err(Error::Io); }
            *self.live.get_mut(sector as usize).ok_or(Error::Bounds)? = *bytes; Ok(())
        }
        fn flush(&mut self) -> Result<(), Error> {
            // Failure can occur after bytes became durable.
            let fail = self.hit(); self.durable = self.live.clone();
            if fail { Err(Error::Io) } else { Ok(()) }
        }
    }
    fn call(store: &mut Store<Disk>, sender: u64, op: u8, name: &[u8], data: &[u8])
        -> Result<[u8;128],Failure> {
        store.process(sender,1,&wire::request(op,name,data).unwrap())
    }
    #[test] fn full_width_incarnations_do_not_alias_low_bits() {
        let files=(1u64<<32)|3;let terminal=u64::MAX;
        let mut store=Store::mount(Disk::new(4),14,files,terminal).unwrap();
        let list=wire::request(wire::LIST,b"",b"").unwrap();
        for generation in [3,u32::MAX as u64,files-1] {
            assert_eq!(store.process(4,generation,&list).unwrap()[0],wire::INVALID);
        }
        assert_eq!(store.process(4,files,&list).unwrap()[0],wire::OK);
        assert_eq!(store.process(6,terminal,&list).unwrap()[0],wire::OK);
        assert_eq!(store.process(6,u32::MAX as u64,&list).unwrap()[0],wire::INVALID);
    }
    #[test] fn terminal_write_files_read_after_fresh_mount_model() {
        let disk = Disk::new(4); let before = disk.live.clone(); let writes = disk.writes.clone();
        let mut store = Store::mount(disk,14,1,1).unwrap();
        assert_eq!(writes.get(),0);
        assert_eq!(call(&mut store,4,wire::LIST,b"",b"").unwrap()[1],0);
        assert_eq!(call(&mut store,6,wire::CREATE,b"note",b"").unwrap()[0],wire::OK);
        assert_eq!(call(&mut store,6,wire::WRITE,b"note",b"typed on boot one").unwrap()[0],wire::OK);
        assert_eq!(store.revision(),2);
        let disk = store.into_block();
        assert_eq!(&disk.durable[..2],&before[..2]);
        let mut store = Store::mount(disk.reboot(),14,1,1).unwrap();
        let got = call(&mut store,4,wire::READ,b"note",b"").unwrap();
        assert_eq!(&got[16..16+got[2] as usize],b"typed on boot one");
        assert_eq!(writes.get(),6);
        assert_eq!(call(&mut store,4,wire::LIST,b"",b"").unwrap()[1],1);
    }
    #[test] fn publication_failures_never_ack_or_retry_or_serve_uncertain_cache() {
        for operation in 1..=12 {
            let mut disk = Disk::new(4); disk.fail = Some(14+operation);
            let mut store = Store::mount(disk,14,1,1).unwrap();
            let failed = call(&mut store,6,wire::CREATE,b"note",b"");
            assert_eq!(failed,Err(if operation >= 10 {
                Failure::Indeterminate
            } else { Failure::Unavailable }));
            for op in [wire::CREATE,wire::READ,wire::LIST] {
                assert_eq!(call(&mut store,4,op,if op==wire::LIST{b""}else{b"note"},b""),
                           Err(Failure::Unavailable));
            }
            let disk = store.into_block();
            assert_eq!(disk.calls,14+operation);
            let mut recovered = Store::mount(disk.reboot(),14,1,1).unwrap();
            let got = call(&mut recovered,4,wire::READ,b"note",b"").unwrap();
            assert!(got[0]==wire::OK || got[0]==wire::NOT_FOUND);
            assert_eq!(got[0]==wire::OK,recovered.revision()==1);
        }
    }
    #[test] fn exhaustion_keeps_reads_available_but_denies_new_mutation() {
        let mut store = Store::mount(Disk::new(1),5,1,1).unwrap();
        assert_eq!(call(&mut store,6,wire::CREATE,b"note",b"").unwrap()[0],wire::OK);
        assert_eq!(call(&mut store,6,wire::WRITE,b"note",b"x"),Err(Failure::ReadOnly));
        assert_eq!(call(&mut store,4,wire::READ,b"note",b"").unwrap()[0],wire::OK);
        let mut store = Store::mount(store.into_block().reboot(),5,1,1).unwrap();
        assert_eq!(call(&mut store,4,wire::LIST,b"",b"").unwrap()[1],1);
        assert_eq!(call(&mut store,6,wire::CREATE,b"other",b""),Err(Failure::ReadOnly));
    }
    #[test] fn canonical_requests_and_kernel_sender_identity_are_required() {
        let disk = Disk::new(4); let writes=disk.writes.clone();
        let mut store = Store::mount(disk,14,1,1).unwrap();
        let request=wire::request(wire::CREATE,b"a",b"").unwrap();
        for sender in [0,1,2,3,5,7,u64::MAX] {
            assert_eq!(store.process(sender,1,&request).unwrap()[0],wire::INVALID);
        }
        for generation in [0,2,u32::MAX as u64,u64::MAX] {
            assert_eq!(store.process(4,generation,&request).unwrap()[0],wire::INVALID);
        }
        for length in 0..128 {
            assert_eq!(store.process(4,1,&request[..length]).unwrap()[0],wire::INVALID);
        }
        for (at,byte) in [(0,255),(1,255),(2,255),(3,1),(15,1),(127,1)] {
            let mut bad=request;bad[at]=byte;
            assert_eq!(store.process(4,1,&bad).unwrap()[0],wire::INVALID);
        }
        for name in [b".".as_slice(),b"..",b"a/b",b"a_b",b""] {
            assert_eq!(call(&mut store,4,wire::CREATE,name,b"").unwrap()[0],wire::INVALID);
        }
        for op in [wire::CREATE,wire::READ,wire::LIST] {
            let bad=wire::request(op,if op==wire::LIST{b""}else{b"a"},b"x").unwrap();
            assert_eq!(store.process(4,1,&bad).unwrap()[0],wire::INVALID);
        }
        assert_eq!(writes.get(),0);
    }
    #[test] fn existence_and_quotas_preserve_the_durable_snapshot() {
        let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut store=Store::mount(disk,26,1,1).unwrap();
        for name in [b"d",b"b",b"a",b"c"] {
            assert_eq!(call(&mut store,6,wire::CREATE,name,b"").unwrap()[0],wire::OK);
        }
        let mut expected=[0;128];expected[1]=4;
        expected[4..12].copy_from_slice(&[1,b'a',1,b'b',1,b'c',1,b'd']);
        assert_eq!(call(&mut store,4,wire::LIST,b"",b"").unwrap(),expected);
        assert_eq!(call(&mut store,6,wire::WRITE,b"a",&[1;64]).unwrap()[0],wire::OK);
        assert_eq!(call(&mut store,6,wire::WRITE,b"b",&[2;64]).unwrap()[0],wire::OK);
        let before=writes.get();let revision=store.revision();
        for (op,name,data,wanted) in [(wire::CREATE,b"a".as_slice(),b"".as_slice(),wire::EXISTS),
                (wire::CREATE,b"e",b"",wire::QUOTA),(wire::WRITE,b"e",b"x",wire::NOT_FOUND),
                (wire::WRITE,b"c",b"x",wire::QUOTA)] {
            assert_eq!(call(&mut store,6,op,name,data).unwrap()[0],wanted);
        }
        assert_eq!(writes.get(),before);assert_eq!(store.revision(),revision);
        let mut recovered=Store::mount(store.into_block().reboot(),26,1,1).unwrap();
        assert_eq!(&call(&mut recovered,4,wire::READ,b"a",b"").unwrap()[16..80],&[1;64]);
    }
    #[test] fn corrupt_mount_and_unsupported_namespace_never_format() {
        let mut disk=Disk::new(4);let writes=disk.writes.clone();
        disk.live[0][0]^=1;
        assert!(Store::mount(disk,14,1,1).is_err());assert_eq!(writes.get(),0);
        let disk=Disk::new(4);let writes=disk.writes.clone();
        let mut vault=Vault::mount(disk,14).unwrap();
        let mut snapshot=Snapshot::empty();snapshot.put(b"a_b",b"x").unwrap();
        vault.publish(snapshot).unwrap();let before=writes.get();
        assert!(Store::mount(vault.into_block().reboot(),14,1,1).is_err());
        assert_eq!(writes.get(),before);
    }
    #[test] fn current_incarnations_and_identical_writes_are_explicit() {
        let disk=Disk::new(4);let writes=disk.writes.clone();
        let mut store=Store::mount(disk,14,3,7).unwrap();
        let create=wire::request(wire::CREATE,b"note",b"").unwrap();
        for (sender,generation) in [(4,1),(4,7),(6,1),(6,3)] {
            assert_eq!(store.process(sender,generation,&create).unwrap()[0],wire::INVALID);
        }
        assert_eq!(store.process(6,7,&create).unwrap()[0],wire::OK);
        let empty_write=wire::request(wire::WRITE,b"note",b"").unwrap();
        let before=writes.get();
        assert_eq!(store.process(6,7,&empty_write).unwrap()[0],wire::OK);
        assert_eq!(writes.get(),before);assert_eq!(store.revision(),1);
        let read=wire::request(wire::READ,b"note",b"").unwrap();
        assert_eq!(store.process(4,3,&read).unwrap()[0],wire::OK);
        for (files,terminal) in [(0,1),(1,0)] {
            let disk=Disk::new(4);let writes=disk.writes.clone();
            assert!(Store::mount(disk,14,files,terminal).is_err());
            assert_eq!(writes.get(),0);
        }
    }

    fn private_grant()->crate::app_documents::Grant{
        crate::app_documents::Grant::new(
            crate::app_documents::Owner::from_verified_identity([7;32]).unwrap(),10,0x1_0000_0001).unwrap()
    }
    #[test]fn private_adapter_persists_without_exposing_to_shared_or_revoked_callers(){
        let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();
        assert_eq!(writes.get(),0);assert!(s.read_private(10,0x1_0000_0001).is_err());
        assert_eq!(call(&mut s,6,wire::CREATE,b"note",b"").unwrap()[0],wire::OK);
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",b"shared").unwrap()[0],wire::OK);
        s.install_private().unwrap();s.write_private(10,0x1_0000_0001,b"private").unwrap();
        let rev=s.revision();let count=writes.get();
        s.install_private().unwrap();s.write_private(10,0x1_0000_0001,b"private").unwrap();
        assert_eq!(s.revision(),rev);assert_eq!(writes.get(),count);
        assert_eq!(call(&mut s,4,wire::LIST,b"",b"").unwrap()[1],1);
        assert_eq!(s.read_private(10,1),Err(Failure::Unavailable));
        assert_eq!(s.write_private(4,1,b"attack"),Err(Failure::Unavailable));
        assert_eq!(writes.get(),count);
        let disk=s.into_block().reboot();
        // Old implementation fails closed, without changing valid private data.
        assert!(Store::mount(disk.clone(),26,1,1).is_err());assert_eq!(writes.get(),count);
        let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();
        assert_eq!(writes.get(),count);
        assert_eq!(s.read_private(10,0x1_0000_0001),Ok(&b"private"[..]));
        let got=call(&mut s,4,wire::READ,b"note",b"").unwrap();
        assert_eq!(&got[16..22],b"shared");
        s.revoke_private();
        assert_eq!(s.read_private(10,0x1_0000_0001),Err(Failure::Unavailable));
        assert_eq!(s.write_private(10,0x1_0000_0001,b"attack"),Err(Failure::Unavailable));
        assert_eq!(s.install_private(),Err(Failure::Unavailable));
        assert_eq!(call(&mut s,4,wire::LIST,b"",b"").unwrap()[1],1);
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",b"new shared").unwrap()[0],wire::OK);
        let s=Store::mount_private(s.into_block().reboot(),26,1,1,private_grant()).unwrap();
        assert_eq!(s.read_private(10,0x1_0000_0001),Ok(&b"private"[..]));
    }

    #[test]fn private_publication_failures_never_retry_or_serve_uncertain_cache(){
        for operation in 1..=12{
            let mut disk=Disk::new(8);disk.fail=Some(26+12+operation);
            let writes=disk.writes.clone();
            let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();
            s.install_private().unwrap();
            assert!(s.write_private(10,0x1_0000_0001,b"new").is_err());
            let count=writes.get();
            assert_eq!(s.read_private(10,0x1_0000_0001),Err(Failure::Unavailable));
            assert_eq!(s.write_private(10,0x1_0000_0001,b"retry"),Err(Failure::Unavailable));
            assert_eq!(call(&mut s,4,wire::LIST,b"",b""),Err(Failure::Unavailable));
            assert_eq!(writes.get(),count);
            let s=Store::mount_private(s.into_block().reboot(),26,1,1,private_grant()).unwrap();
            let saved=s.read_private(10,0x1_0000_0001).unwrap();
            assert!(saved==b""||saved==b"new");
            assert_eq!(writes.get(),count);
        }
    }
    #[test]fn private_mount_refuses_other_identity_without_modifying_disk(){
        let disk=Disk::new(4);let writes=disk.writes.clone();
        let mut s=Store::mount_private(disk,14,1,1,private_grant()).unwrap();
        s.install_private().unwrap();let count=writes.get();
        let other=crate::app_documents::Grant::new(
            crate::app_documents::Owner::from_verified_identity([8;32]).unwrap(),10,1).unwrap();
        assert!(Store::mount_private(s.into_block().reboot(),14,1,1,other).is_err());
        assert_eq!(writes.get(),count);
    }

    #[test]fn shared_writes_cannot_spend_reserved_private_capacity(){
        let mut s=Store::mount_private(Disk::new(8),26,1,1,private_grant()).unwrap();
        s.install_private().unwrap();
        assert_eq!(call(&mut s,6,wire::CREATE,b"note",b"").unwrap()[0],wire::OK);
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",&[1;32]).unwrap()[0],wire::OK);
        let revision=s.revision();
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",&[1;33]).unwrap()[0],wire::QUOTA);
        assert_eq!(s.revision(),revision);
        s.write_private(10,0x1_0000_0001,&[2;64]).unwrap();
        s.revoke_private();
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",&[1;33]).unwrap()[0],wire::QUOTA);
    }

    #[test]fn pending_private_install_reserves_slots_and_values_without_writing_metadata(){
        let mut s=Store::mount_private(Disk::new(8),26,1,1,private_grant()).unwrap();
        assert_eq!(s.vault.snapshot().entries().count(),0);
        for name in [b"a",b"b"]{assert_eq!(call(&mut s,6,wire::CREATE,name,b"").unwrap()[0],wire::OK);}
        let revision=s.revision();
        assert_eq!(call(&mut s,6,wire::CREATE,b"c",b"").unwrap()[0],wire::QUOTA);
        assert_eq!(call(&mut s,6,wire::WRITE,b"a",&[1;33]).unwrap()[0],wire::QUOTA);
        assert_eq!(s.revision(),revision);
        assert_eq!(s.vault.snapshot().entries().count(),2);
        s.install_private().unwrap();s.write_private(10,0x1_0000_0001,&[7;64]).unwrap();
        assert_eq!(s.read_private(10,0x1_0000_0001),Ok(&[7;64][..]));
    }
    #[test]fn private_install_faults_are_atomic_sticky_and_recoverable(){
        for operation in 1..=12{
            let mut disk=Disk::new(8);disk.fail=Some(26+operation);
            let writes=disk.writes.clone();
            let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();
            assert!(s.install_private().is_err());let count=writes.get();
            assert_eq!(s.install_private(),Err(Failure::Unavailable));
            assert_eq!(s.write_private(10,0x1_0000_0001,b"retry"),Err(Failure::Unavailable));
            assert_eq!(s.read_private(10,0x1_0000_0001),Err(Failure::Unavailable));
            assert_eq!(call(&mut s,4,wire::LIST,b"",b""),Err(Failure::Unavailable));
            assert_eq!(writes.get(),count);
            let mut s=Store::mount_private(s.into_block().reboot(),26,1,1,private_grant()).unwrap();
            let entries=s.vault.snapshot().entries().count();
            assert!(entries==0||entries==2);
            assert_eq!(entries==2,s.revision()==1);
            assert_eq!(writes.get(),count);
            s.install_private().unwrap();s.write_private(10,0x1_0000_0001,b"recovered").unwrap();
            let s=Store::mount_private(s.into_block().reboot(),26,1,1,private_grant()).unwrap();
            assert_eq!(s.read_private(10,0x1_0000_0001),Ok(&b"recovered"[..]));
        }
    }

    #[test]fn app_endpoint_authenticates_frames_prevents_replay_and_preserves_shared_data(){
        use app_sdk::wire::{Message,READ_DOCUMENT,WRITE_DOCUMENT};
        let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();
        call(&mut s,6,wire::CREATE,b"note",b"").unwrap();
        call(&mut s,6,wire::WRITE,b"note",b"shared").unwrap();s.install_private().unwrap();
        let request=Message::new(WRITE_DOCUMENT,0,1,b"hello").unwrap().encode();
        let count=writes.get();
        for (sender,inc)in [(4,1),(11,0x1_0000_0001),(10,1),(10,0),(0x1_0000_000a,0x1_0000_0001)]{
            assert_eq!(s.process_app(sender,inc,&request),None);
        }
        assert_eq!(writes.get(),count);
        let reply=Message::decode(&s.process_app(10,0x1_0000_0001,&request).unwrap()).unwrap();
        assert_eq!((reply.operation(),reply.sequence(),reply.status(),reply.payload()),(WRITE_DOCUMENT,1,0,&b""[..]));
        let count=writes.get();let revision=s.revision();
        assert_eq!(s.process_app(10,0x1_0000_0001,&request),None);
        assert_eq!(writes.get(),count);assert_eq!(s.revision(),revision);
        let read=Message::new(READ_DOCUMENT,0,2,b"").unwrap().encode();
        let reply=Message::decode(&s.process_app(10,0x1_0000_0001,&read).unwrap()).unwrap();
        assert_eq!(reply.payload(),b"hello");
        let shared=call(&mut s,4,wire::READ,b"note",b"").unwrap();
        assert_eq!(&shared[16..22],b"shared");
        let mut s=Store::mount_private(s.into_block().reboot(),26,1,1,private_grant()).unwrap();
        assert_eq!(s.read_private(10,0x1_0000_0001),Ok(&b"hello"[..]));
        s.revoke_private();assert_eq!(s.process_app(10,0x1_0000_0001,&read),None);
    }
    #[test]fn app_endpoint_malformed_and_failure_paths_never_publish_or_retry(){
        use app_sdk::{wire::{Message,WRITE_DOCUMENT,READ_DOCUMENT},protocol};
        let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();s.install_private().unwrap();
        let count=writes.get();
        for (op,status,data)in [(WRITE_DOCUMENT,1,&b"x"[..]),(READ_DOCUMENT,0,&b"x"[..]),
            (WRITE_DOCUMENT,0,&[1;65][..]),(1,0,&b""[..])]{
            let raw=Message::new(op,status,u32::MAX,data).unwrap().encode();
            assert_eq!(s.process_app(10,0x1_0000_0001,&raw),None);
        }
        assert_eq!(s.app_sequence,0);assert_eq!(writes.get(),count);
        for operation in 1..=12{
            let mut disk=Disk::new(8);disk.fail=Some(26+12+operation);
            let writes=disk.writes.clone();
            let mut s=Store::mount_private(disk,26,1,1,private_grant()).unwrap();s.install_private().unwrap();
            let raw=Message::new(WRITE_DOCUMENT,0,1,b"new").unwrap().encode();
            let reply=Message::decode(&s.process_app(10,0x1_0000_0001,&raw).unwrap()).unwrap();
            assert!(matches!(reply.status(),protocol::UNAVAILABLE|protocol::INDETERMINATE));
            assert!(reply.payload().is_empty());let count=writes.get();
            assert_eq!(s.process_app(10,0x1_0000_0001,&raw),None);
            let read=Message::new(READ_DOCUMENT,0,2,b"").unwrap().encode();
            let reply=Message::decode(&s.process_app(10,0x1_0000_0001,&read).unwrap()).unwrap();
            assert_eq!(reply.status(),protocol::UNAVAILABLE);assert_eq!(writes.get(),count);
        }
    }

    #[test]fn application_mount_is_inactive_rebind_is_monotonic_and_preserves_pending_reply_identity(){
        use app_sdk::wire::{Message,READ_DOCUMENT,WRITE_DOCUMENT};
        let owner=private_grant().owner();let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut s=Store::mount_applications(disk,26,1,1,owner).unwrap();
        assert_eq!(writes.get(),0);assert!(s.private.is_none());assert_eq!(s.private_highest,0);
        assert!(!s.app_reply_current(10,1));
        call(&mut s,6,wire::CREATE,b"note",b"").unwrap();
        assert_eq!(call(&mut s,6,wire::WRITE,b"note",&[1;33]).unwrap()[0],wire::QUOTA);
        let old=private_grant();s.rebind_private(old).unwrap();s.install_private().unwrap();
        let write=Message::new(WRITE_DOCUMENT,0,100,b"retained").unwrap().encode();
        let pending=s.process_app(10,old.incarnation(),&write).unwrap();
        assert!(s.app_reply_current(10,old.incarnation()));
        let count=writes.get();s.revoke_private();
        assert!(!s.app_reply_current(10,old.incarnation()));
        let next=crate::app_documents::Grant::new(owner,10,old.incarnation()+1).unwrap();
        for bad in [
            old,crate::app_documents::Grant::new(owner,11,old.incarnation()+1).unwrap(),
            crate::app_documents::Grant::new(crate::app_documents::Owner::from_verified_identity([8;32]).unwrap(),
                10,old.incarnation()+1).unwrap(),
        ]{
            assert!(s.rebind_private(bad).is_err());assert!(s.private.is_none());
            assert_eq!(s.private_highest,old.incarnation());assert_eq!(s.app_sequence,100);
            assert_eq!(writes.get(),count);
        }
        s.rebind_private(next).unwrap();assert_eq!(s.app_sequence,0);
        assert!(!s.app_reply_current(10,old.incarnation()));
        assert_eq!(Message::decode(&pending).unwrap().sequence(),100);
        assert!(s.app_reply_current(10,next.incarnation()));
        assert_eq!(s.process_app(10,old.incarnation(),&write),None);
        let read=Message::new(READ_DOCUMENT,0,1,b"").unwrap().encode();
        let reply=Message::decode(&s.process_app(10,next.incarnation(),&read).unwrap()).unwrap();
        assert_eq!(reply.payload(),b"retained");assert_eq!(writes.get(),count);
        assert!(s.rebind_private(crate::app_documents::Grant::new(owner,10,next.incarnation()+1).unwrap()).is_err());
        let mut s=Store::mount_applications(s.into_block().reboot(),26,1,1,owner).unwrap();
        assert_eq!(writes.get(),count);assert!(s.private.is_none());
        s.rebind_private(crate::app_documents::Grant::new(owner,10,1).unwrap()).unwrap();
        assert_eq!(s.read_private(10,1),Ok(&b"retained"[..]));
    }
}
