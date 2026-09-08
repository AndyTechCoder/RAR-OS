//! Modern-only durable adapter for the existing Terminal/Files request bytes.
//! No VM activation or device authority. Runtime must provide only a Data Block.
#![forbid(unsafe_code)]
use crate::vault::{Block, Error, Snapshot, Vault};
use crate::desktop_wire as wire;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Failure { ReadOnly, Unavailable, Indeterminate }

pub struct Store<B: Block> {
    vault: Vault<B>,
    unavailable: bool,
    files_generation: u32,
    terminal_generation: u32,
}
fn name_ok(name: &[u8]) -> bool {
    !name.is_empty() && name.len() <= 12 && name != b"." && name != b".." &&
        name.iter().all(|b| b.is_ascii_alphanumeric() || *b == b'.' || *b == b'-')
}
fn status(value: u8) -> [u8; 128] { let mut out = [0;128]; out[0] = value; out }

impl<B: Block> Store<B> {
    /// Mount only: no welcome-file creation, formatting or write on boot.
    pub fn mount(block: B, capacity: u32, files_generation: u32, terminal_generation: u32)
        -> Result<Self, Error> {
        if files_generation == 0 || terminal_generation == 0 { return Err(Error::Invalid); }
        let vault = Vault::mount(block, capacity)?;
        // Preserve the existing app namespace; never silently omit unsupported
        // entries or rewrite a valid but incompatible snapshot.
        if vault.snapshot().entries().any(|(name, _)| !name_ok(name)) {
            return Err(Error::Invalid);
        }
        Ok(Self { vault, unavailable: false, files_generation, terminal_generation })
    }
    pub fn into_block(self) -> B { self.vault.into_block() }
    pub fn revision(&self) -> u64 { self.vault.revision() }

    /// Sender/generation must be copied from the kernel receive envelope.
    /// Mount's expected incarnations come from trusted Modern bootstrap grants,
    /// never request bytes. Recreate this adapter after grant/incarnation changes.
    /// A successful mutation reply is produced ONLY after Vault's durable ACK.
    /// Failures are private typed results, not new values in the old wire ABI;
    /// the Modern runtime must display them explicitly and never synthesize OK.
    pub fn process(&mut self, sender: u64, generation: u32, request: &[u8])
        -> Result<[u8;128], Failure>
    {
        let authorized = match sender {
            4 => generation == self.files_generation,
            6 => generation == self.terminal_generation,
            _ => false,
        };
        if !authorized || request.len() != 128 {
            return Ok(status(wire::INVALID));
        }
        let op = request[0];
        let names = request[1] as usize;
        let data = request[2] as usize;
        if !matches!(op, wire::CREATE|wire::WRITE|wire::READ|wire::LIST) ||
            names > 12 || data > 64 || request[3] != 0 ||
            request[4+names..16].iter().any(|b| *b != 0) ||
            request[16+data..].iter().any(|b| *b != 0) ||
            (op != wire::WRITE && data != 0) {
            return Ok(status(wire::INVALID));
        }
        let name = &request[4..4+names];
        if (op == wire::LIST && names != 0) || (op != wire::LIST && !name_ok(name)) {
            return Ok(status(wire::INVALID));
        }
        if self.unavailable { return Err(Failure::Unavailable); }
        let snapshot = *self.vault.snapshot();
        match op {
            wire::READ => {
                let Some(value) = snapshot.get(name) else { return Ok(status(wire::NOT_FOUND)); };
                let mut reply = [0;128];
                reply[2] = value.len() as u8;
                reply[16..16+value.len()].copy_from_slice(value);
                Ok(reply)
            }
            wire::LIST => {
                let mut reply = [0;128];
                let mut at = 4;
                for (name, _) in snapshot.entries() {
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
                if op == wire::WRITE && snapshot.get(name) == Some(&request[16..16+data]) {
                    return Ok(status(wire::OK));
                }
                if self.vault.is_readonly() { return Err(Failure::ReadOnly); }
                let mut candidate: Snapshot = snapshot;
                if candidate.put(name, &request[16..16+data]).is_err() {
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
            assert!(failed.is_err());
            if operation >= 10 { assert_eq!(failed,Err(Failure::Indeterminate)); }
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
        for generation in [0,2,u32::MAX] {
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
        assert_eq!(writes.get(),0);
    }
    #[test] fn existence_and_quotas_preserve_the_durable_snapshot() {
        let disk=Disk::new(8);let writes=disk.writes.clone();
        let mut store=Store::mount(disk,26,1,1).unwrap();
        for name in [b"a",b"b",b"c",b"d"] {
            assert_eq!(call(&mut store,6,wire::CREATE,name,b"").unwrap()[0],wire::OK);
        }
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
}
