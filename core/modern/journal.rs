//! Modern-v0 System selector codec and bounded publication. No native I/O or Data handle.
//! Checksums detect corruption, not malicious rewrites or wholesale rollback.
use crate::{manifest::VerifiedLayer, sha256::sha256};

pub const SIZE: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Reject { Framing, Checksum, State, Ambiguous, Exhausted, Downgrade, NoFallback }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot { A, B }
impl Slot {
    pub fn other(self) -> Self { match self { Self::A => Self::B, Self::B => Self::A } }
    fn byte(self) -> u8 { match self { Self::A => 0, Self::B => 1 } }
    fn parse(v: u8) -> Result<Self, Reject> {
        match v { 0 => Ok(Self::A), 1 => Ok(Self::B), _ => Err(Reject::State) }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayerId { slot: Slot, generation: u64, digest: [u8; 32] }
impl LayerId {
    pub fn slot(&self) -> Slot { self.slot }
    pub fn generation(&self) -> u64 { self.generation }
    pub fn digest(&self) -> [u8; 32] { self.digest }
    fn valid(&self) -> bool { self.generation != 0 && self.digest != [0;32] }
    fn from_verified(slot: Slot, layer: &VerifiedLayer<'_>) -> Self {
        Self { slot, generation: layer.manifest().generation(), digest: layer.manifest().digest() }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind { Factory, Install, Fallback, Repair }
impl Kind {
    fn byte(self) -> u8 { match self { Self::Factory => 0, Self::Install => 1, Self::Fallback => 2, Self::Repair => 3 } }
    fn parse(v: u8) -> Result<Self, Reject> {
        match v { 0 => Ok(Self::Factory), 1 => Ok(Self::Install), 2 => Ok(Self::Fallback), 3 => Ok(Self::Repair), _ => Err(Reject::State) }
    }
}

/// A structurally valid checksum record; NOT proof that referenced layers are
/// present, signed, healthy, or current relative to a non-rollbackable anchor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Record {
    kind: Kind, sequence: u64, highest: u64, active: LayerId,
    previous: Option<LayerId>, parent_sequence: u64, parent_digest: [u8;32],
}
fn hash(b: &[u8]) -> [u8;32] { sha256(b).expect("all journal callers hash at most 512 bytes") }
fn get64(b: &[u8;SIZE], offset: usize) -> u64 { u64::from_le_bytes(b[offset..offset+8].try_into().unwrap()) }
fn put64(b: &mut [u8;SIZE], offset: usize, value: u64) { b[offset..offset+8].copy_from_slice(&value.to_le_bytes()); }

impl Record {
    pub fn active(&self) -> LayerId { self.active }
    pub fn previous(&self) -> Option<LayerId> { self.previous }
    pub fn sequence(&self) -> u64 { self.sequence }
    pub fn highest_committed_generation(&self) -> u64 { self.highest }
    pub fn minimum_install_generation(&self) -> Result<u64, Reject> {
        self.highest.checked_add(1).ok_or(Reject::Exhausted)
    }
    pub fn factory(layer: &VerifiedLayer<'_>) -> Self {
        Self::factory_id(LayerId::from_verified(Slot::A, layer))
    }
    fn factory_id(active: LayerId) -> Self {
        Self { kind: Kind::Factory, sequence:1, highest:active.generation, active,
            previous:None, parent_sequence:0, parent_digest:[0;32] }
    }
    pub fn install(&self, layer: &VerifiedLayer<'_>) -> Result<Self, Reject> {
        self.install_id(LayerId::from_verified(self.active.slot.other(), layer))
    }
    fn next(&self) -> Result<Self, Reject> {
        let mut n = *self;
        n.sequence = self.sequence.checked_add(1).ok_or(Reject::Exhausted)?;
        n.parent_sequence = self.sequence;
        n.parent_digest = hash(&self.encode());
        Ok(n)
    }
    fn install_id(&self, candidate: LayerId) -> Result<Self, Reject> {
        if !candidate.valid() || candidate.slot != self.active.slot.other() ||
            candidate.generation < self.minimum_install_generation()? {
            return Err(Reject::Downgrade);
        }
        let mut n = self.next()?;
        n.kind = Kind::Install;
        n.previous = Some(self.active);
        n.active = candidate;
        n.highest = candidate.generation;
        Ok(n)
    }
    /// Plans fallback only. Caller must reverify previous manifest/PE and health
    /// before publishing this record; the record cannot grant execution.
    pub fn fallback(&self) -> Result<Self, Reject> {
        let previous = self.previous.ok_or(Reject::NoFallback)?;
        let mut n = self.next()?;
        n.kind = Kind::Fallback;
        n.active = previous;
        n.previous = None;
        // Never lower the install high-water mark when executing authorized fallback.
        Ok(n)
    }
    /// Pure repair planning only. The repair module separately binds the exact
    /// immutable factory and damaged content. No disk/lifecycle authority follows.
    #[cfg(any(test,rar_signed_updates))]
    pub(crate) fn repair_factory(&self, factory:&VerifiedLayer<'_>)->Result<Self,Reject> {
        self.repair_id(LayerId::from_verified(self.active.slot.other(),factory))
    }
    /// Structural binding only; not factory provenance, damage or health proof.
    pub(crate) fn is_repair_successor_of(&self,older:&Self)->bool{
        self.kind==Kind::Repair&&self.valid()&&older.valid()&&self.follows(older)
    }
    #[cfg(any(test,rar_signed_updates))]
    fn repair_id(&self,factory:LayerId)->Result<Self,Reject> {
        if !factory.valid()||factory.generation!=1||factory.slot!=self.active.slot.other(){
            return Err(Reject::State);
        }
        let mut n=self.next()?;
        n.kind=Kind::Repair;n.active=factory;n.previous=None;
        // Preserve the high-water even when the immutable root is generation1.
        if !n.valid(){return Err(Reject::State);}
        Ok(n)
    }
    fn valid(&self) -> bool {
        if self.sequence == 0 || !self.active.valid() || self.highest < self.active.generation { return false; }
        if let Some(p) = self.previous {
            if !p.valid() || p.slot == self.active.slot || p.generation >= self.active.generation { return false; }
        }
        match self.kind {
            Kind::Factory => self.sequence == 1 && self.active.slot == Slot::A && self.previous.is_none() &&
                self.parent_sequence == 0 && self.parent_digest == [0;32] &&
                self.highest == self.active.generation,
            Kind::Install => self.sequence >= 2 && self.previous.is_some() &&
                self.parent_sequence == self.sequence-1 && self.parent_digest != [0;32] &&
                self.highest == self.active.generation,
            Kind::Fallback => self.sequence >= 3 && self.previous.is_none() &&
                self.parent_sequence == self.sequence-1 && self.parent_digest != [0;32] &&
                self.highest > self.active.generation,
            Kind::Repair => self.sequence >= 2 && self.previous.is_none() &&
                self.active.generation == 1 && self.parent_sequence == self.sequence-1 &&
                self.parent_digest != [0;32],
        }
    }
    pub fn encode(&self) -> [u8;SIZE] {
        let mut b = [0;SIZE];
        b[..8].copy_from_slice(b"RARSYS00");
        b[10..12].copy_from_slice(&(SIZE as u16).to_le_bytes());
        b[12]=self.kind.byte(); b[13]=self.active.slot.byte();
        b[14]=self.previous.map_or(255, |p| p.slot.byte());
        put64(&mut b,16,self.sequence); put64(&mut b,24,self.highest);
        put64(&mut b,32,1); // fixed laboratory root floor, not a hardware counter.
        put64(&mut b,40,self.active.generation);
        put64(&mut b,56,self.parent_sequence);
        b[64..96].copy_from_slice(&self.active.digest);
        if let Some(p)=self.previous {
            put64(&mut b,48,p.generation); b[96..128].copy_from_slice(&p.digest);
        }
        b[128..160].copy_from_slice(&self.parent_digest);
        let digest=hash(&b[..480]); b[480..].copy_from_slice(&digest);
        b
    }
    pub fn decode(raw: &[u8]) -> Result<Self, Reject> {
        let b: &[u8;SIZE]=raw.try_into().map_err(|_|Reject::Framing)?;
        if &b[..8]!=b"RARSYS00" || b[8..10]!=[0,0] ||
            b[10..12]!=(SIZE as u16).to_le_bytes() || b[15]!=0 ||
            b[160..480].iter().any(|&x|x!=0) { return Err(Reject::Framing); }
        if b[480..]!=hash(&b[..480]) { return Err(Reject::Checksum); }
        if get64(b,32)!=1 { return Err(Reject::State); }
        let previous=if b[14]==255 {
            if get64(b,48)!=0 || b[96..128]!=[0;32] { return Err(Reject::State); }
            None
        } else {
            Some(LayerId {slot:Slot::parse(b[14])?, generation:get64(b,48), digest:b[96..128].try_into().unwrap()})
        };
        let r=Self {kind:Kind::parse(b[12])?, sequence:get64(b,16), highest:get64(b,24),
            active:LayerId {slot:Slot::parse(b[13])?, generation:get64(b,40),digest:b[64..96].try_into().unwrap()},
            previous, parent_sequence:get64(b,56),parent_digest:b[128..160].try_into().unwrap()};
        if !r.valid() { return Err(Reject::State); }
        Ok(r)
    }
    fn follows(&self, older:&Self)->bool {
        if older.sequence.checked_add(1)!=Some(self.sequence) ||
            self.parent_sequence!=older.sequence || self.parent_digest!=hash(&older.encode()) { return false; }
        match self.kind {
            Kind::Factory=>false,
            Kind::Install=>self.previous==Some(older.active) &&
                self.active.slot==older.active.slot.other() && self.active.generation>older.highest &&
                self.highest==self.active.generation,
            Kind::Fallback=>older.previous==Some(self.active) && self.previous.is_none() &&
                self.highest==older.highest,
            Kind::Repair=>self.active.slot==older.active.slot.other() &&
                self.active.generation==1 && self.previous.is_none() && self.highest==older.highest,
        }
    }
}

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Selection { record:Record, sector:usize }
impl Selection {
    pub fn record(&self)->Record {self.record}
    pub fn sector(&self)->usize {self.sector}
    pub fn next_sector(&self)->usize {1-self.sector}
}
pub fn select(sectors:[&[u8];2])->Result<Selection,Reject> {
    let a=Record::decode(sectors[0]).ok();
    let b=Record::decode(sectors[1]).ok();
    match (a,b) {
        (None,None)=>Err(Reject::Ambiguous),
        (Some(r),None)=>Ok(Selection {record:r,sector:0}),
        (None,Some(r))=>Ok(Selection {record:r,sector:1}),
        (Some(a),Some(b))=>{
            if a.sequence==b.sequence { return Err(Reject::Ambiguous); }
            let (older,newer,sector)=if a.sequence<b.sequence {(a,b,1)}else{(b,a,0)};
            if !newer.follows(&older) { return Err(Reject::Ambiguous); }
            Ok(Selection {record:newer,sector})
        }
    }
}


/// Two selector records only, supplied by the System service's exclusive adapter.
/// This typed interface contains no Data or arbitrary-sector selector. It is not
/// itself a kernel capability boundary; its implementation must enforce ownership.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum SelectorSector { First, Second }
impl SelectorSector {
    fn from_index(index:usize)->Self { if index==0 {Self::First}else{Self::Second} }
}
pub trait SelectorIo {
    fn read(&mut self,sector:SelectorSector)->Result<[u8;SIZE],()>;
    fn write(&mut self,sector:SelectorSector,bytes:&[u8;SIZE])->Result<(),()>;
    fn flush(&mut self)->Result<(),()>;
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum PublicationError { Io, Corrupt, Changed, Transition, ReadOnly, Indeterminate }

/// Durable selector publication, not signature, health, or execution authority.
/// Before commit the caller must durably stage and reverify the inactive signed
/// image and complete lifecycle preparation/health. No API here can grant those
/// rights. Mount and record selection remain provisional until layer verification.
pub struct Journal<I:SelectorIo> {io:I,selected:Selection,readonly:bool}
impl<I:SelectorIo> Journal<I> {
    pub fn mount(mut io:I)->Result<Self,PublicationError> {
        let first=io.read(SelectorSector::First).map_err(|_|PublicationError::Io)?;
        let second=io.read(SelectorSector::Second).map_err(|_|PublicationError::Io)?;
        let selected=select([&first,&second]).map_err(|_|PublicationError::Corrupt)?;
        Ok(Self {io,selected,readonly:false})
    }
    pub fn record(&self)->Record {self.selected.record()}
    pub fn selection(&self)->Selection {self.selected}
    pub fn is_readonly(&self)->bool {self.readonly}
    pub fn into_io(self)->I {self.io}

    pub fn commit(&mut self,next:Record)->Result<(),PublicationError> {
        if self.readonly {return Err(PublicationError::ReadOnly);}
        if !next.valid() || !next.follows(&self.selected.record) {
            return Err(PublicationError::Transition);
        }
        let result=self.commit_inner(next);
        // Any I/O or observed-media failure locks writes in this instance.
        // An indeterminate publication may have committed despite no ACK.
        if result.is_err(){self.readonly=true;}
        result
    }
    fn commit_inner(&mut self,next:Record)->Result<(),PublicationError> {
        let before=[
            self.io.read(SelectorSector::First).map_err(|_|PublicationError::Io)?,
            self.io.read(SelectorSector::Second).map_err(|_|PublicationError::Io)?,
        ];
        let current=select([&before[0],&before[1]]).map_err(|_|PublicationError::Corrupt)?;
        if current!=self.selected {return Err(PublicationError::Changed);}
        let target=self.selected.next_sector();
        let bytes=next.encode();
        // From the first attempted write onward, errors are indeterminate.
        // Never overwrite the selected sector, retry, repair, or autoformat.
        self.io.write(SelectorSector::from_index(target),&bytes)
            .map_err(|_|PublicationError::Indeterminate)?;
        self.io.flush().map_err(|_|PublicationError::Indeterminate)?;
        let after=[
            self.io.read(SelectorSector::First).map_err(|_|PublicationError::Indeterminate)?,
            self.io.read(SelectorSector::Second).map_err(|_|PublicationError::Indeterminate)?,
        ];
        if after[target]!=bytes || after[self.selected.sector()]!=before[self.selected.sector()] {
            return Err(PublicationError::Indeterminate);
        }
        let selected=select([&after[0],&after[1]]).map_err(|_|PublicationError::Indeterminate)?;
        if selected.record()!=next || selected.sector()!=target {
            return Err(PublicationError::Indeterminate);
        }
        self.selected=selected;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn id(slot:Slot,generation:u64)->LayerId {LayerId {slot,generation,digest:[generation as u8;32]}}
    fn base()->Record {Record::factory_id(id(Slot::A,1))}
    fn chain()->(Record,Record,Record) {
        let a=base(); let b=a.install_id(id(Slot::B,2)).unwrap();
        let c=b.install_id(id(Slot::A,3)).unwrap(); (a,b,c)
    }
    #[test] fn canonical_roundtrip_and_alternating_selection() {
        let (a,b,c)=chain();
        for r in [a,b,c,b.fallback().unwrap()] {assert_eq!(Record::decode(&r.encode()),Ok(r));}
        assert_eq!(select([&a.encode(),&[0;SIZE]]).unwrap().record,a);
        let selected=select([&a.encode(),&b.encode()]).unwrap();
        assert_eq!((selected.record,selected.next_sector()),(b,0));
        assert_eq!(select([&c.encode(),&b.encode()]).unwrap().record,c);
    }
    #[test] fn every_single_byte_corruption_and_truncation_fails() {
        let encoded=base().encode();
        for p in 0..SIZE {
            let mut bad=encoded; bad[p]^=1;
            assert!(Record::decode(&bad).is_err(),"{p}");
            assert!(Record::decode(&encoded[..p]).is_err());
        }
        assert!(Record::decode(&[0;SIZE+1]).is_err());
    }
    #[test] fn torn_selector_publication_selects_only_complete_old_or_new() {
        let (a,b,c)=chain(); let fallback=b.fallback().unwrap();
        // First install, subsequent install, and rollback publication all differ.
        for (old_sectors,current,next,target) in [
            ([a.encode(),[0;SIZE]],a,b,1usize),
            ([a.encode(),b.encode()],b,c,0),
            ([a.encode(),b.encode()],b,fallback,0),
        ] {
            let complete=next.encode();
            for cut in 0..=SIZE {
                let mut sectors=old_sectors;
                sectors[target][..cut].copy_from_slice(&complete[..cut]);
                let selected=select([&sectors[0],&sectors[1]]).unwrap();
                // A shorter prefix can already be complete if the untouched
                // suffix equals the successor. Judge actual bytes, not cut count.
                let expected=if sectors[target]==complete {next}else{current};
                assert_eq!(selected.record,expected,"kind {:?}, cut {cut}",next.kind);
            }
        }
    }
    #[test] fn missing_conflicting_forked_and_gapped_records_fail_closed() {
        let (a,b,c)=chain();
        assert_eq!(select([&[0;SIZE],&[0;SIZE]]),Err(Reject::Ambiguous));
        assert_eq!(select([&a.encode(),&a.encode()]),Err(Reject::Ambiguous));
        assert_eq!(select([&a.encode(),&c.encode()]),Err(Reject::Ambiguous));
        let fork=Record::factory_id(id(Slot::A,4)).install_id(id(Slot::B,5)).unwrap();
        assert_eq!(select([&a.encode(),&fork.encode()]),Err(Reject::Ambiguous));
        assert_eq!(select([&b.encode(),&[0;SIZE]]).unwrap().record,b);
    }
    #[test] fn fallback_retains_high_water_and_cannot_loop() {
        let a=base(); let b=a.install_id(id(Slot::B,7)).unwrap();
        let fallback=b.fallback().unwrap();
        assert_eq!(fallback.active,a.active);
        assert_eq!(fallback.highest,7);
        assert_eq!(fallback.minimum_install_generation(),Ok(8));
        assert_eq!(fallback.fallback(),Err(Reject::NoFallback));
        assert_eq!(fallback.install_id(id(Slot::B,7)),Err(Reject::Downgrade));
        let update=fallback.install_id(id(Slot::B,8)).unwrap();
        assert_eq!(select([&fallback.encode(),&update.encode()]).unwrap().record,update);
        assert_eq!(select([&fallback.encode(),&b.encode()]).unwrap().record,fallback);
    }
    #[test] fn integer_exhaustion_retires_without_wraparound() {
        let mut b=base(); b.sequence=u64::MAX;
        assert_eq!(b.install_id(id(Slot::B,2)),Err(Reject::Exhausted));
        b.sequence=1; b.highest=u64::MAX;
        assert_eq!(b.minimum_install_generation(),Err(Reject::Exhausted));
        assert_eq!(b.install_id(id(Slot::B,u64::MAX)),Err(Reject::Exhausted));
    }
    #[test] fn valid_checksums_do_not_override_semantic_constraints() {
        let (_,r,_)=chain();
        for (offset,value) in [(12,8),(13,3),(14,3),(32,2),(40,0),(48,0)] {
            let mut b=r.encode(); b[offset]=value;
            let h=hash(&b[..480]); b[480..].copy_from_slice(&h);
            assert!(Record::decode(&b).is_err(),"{offset}");
        }
    }

    #[derive(Clone)]
    struct Media {live:[[u8;SIZE];2],stable:[[u8;SIZE];2],calls:usize,
        fail:usize,tear:usize,flush_after:bool,writes:Vec<SelectorSector>,corrupt_after_flush:Option<usize>}
    impl Media {
        fn new(sectors:[[u8;SIZE];2])->Self {Self {live:sectors,stable:sectors,
            calls:0,fail:0,tear:0,flush_after:false,writes:vec![],corrupt_after_flush:None}}
        fn hit(&mut self)->bool {self.calls+=1;self.calls==self.fail}
        fn reboot(mut self)->Self {self.live=self.stable;self.calls=0;self.fail=0;self}
    }
    impl SelectorIo for Media {
        fn read(&mut self,sector:SelectorSector)->Result<[u8;SIZE],()> {
            if self.hit(){return Err(());}
            Ok(self.live[if sector==SelectorSector::First {0}else{1}])
        }
        fn write(&mut self,sector:SelectorSector,bytes:&[u8;SIZE])->Result<(),()> {
            self.writes.push(sector);
            let index=if sector==SelectorSector::First {0}else{1};
            if self.hit() {
                self.live[index][..self.tear].copy_from_slice(&bytes[..self.tear]);
                self.stable[index][..self.tear].copy_from_slice(&bytes[..self.tear]);
                return Err(());
            }
            self.live[index]=*bytes;Ok(())
        }
        fn flush(&mut self)->Result<(),()> {
            let fail=self.hit();
            if !fail||self.flush_after {self.stable=self.live;}
            if !fail {if let Some(index)=self.corrupt_after_flush {
                self.live[index][200]^=1;self.stable[index]=self.live[index];
            }}
            if fail {Err(())}else{Ok(())}
        }
    }
    #[test] fn selector_publication_flush_readback_and_fresh_mount() {
        let (a,b,c)=chain();
        let mut journal=Journal::mount(Media::new([a.encode(),[0;SIZE]])).unwrap();
        assert!(journal.io.writes.is_empty());
        journal.commit(b).unwrap();
        assert_eq!(journal.record(),b);
        assert_eq!(journal.io.writes,vec![SelectorSector::Second]);
        let mut journal=Journal::mount(journal.into_io().reboot()).unwrap();
        assert_eq!(journal.record(),b);
        journal.commit(c).unwrap();
        assert_eq!(journal.io.writes,vec![SelectorSector::Second,SelectorSector::First]);
        assert_eq!(Journal::mount(journal.into_io().reboot()).unwrap().record(),c);
    }
    fn publication_fault(sectors:[[u8;SIZE];2],old:Record,next:Record,
                         operation:usize,tear:usize,after:bool) {
        let mut journal=Journal::mount(Media::new(sectors)).unwrap();
        let protected=journal.selected.sector();
        journal.io.calls=0;journal.io.fail=operation;
        journal.io.tear=tear;journal.io.flush_after=after;
        assert_eq!(journal.commit(next),Err(if operation<=2 {PublicationError::Io}
            else {PublicationError::Indeterminate}));
        assert_eq!(journal.record(),old);assert!(journal.is_readonly());
        assert_eq!(journal.io.stable[protected],sectors[protected]);
        let calls=journal.io.calls;let writes=journal.io.writes.clone();
        assert_eq!(journal.commit(next),Err(PublicationError::ReadOnly));
        assert_eq!(journal.io.calls,calls);assert_eq!(journal.io.writes,writes);
        let recovered=Journal::mount(journal.into_io().reboot()).unwrap();
        assert!(recovered.record()==old||recovered.record()==next);
        assert_eq!(recovered.io.stable[protected],sectors[protected]);
    }
    #[test] fn every_publication_io_boundary_and_sector_prefix_recovers_old_or_new() {
        let (a,b,c)=chain();
        for (sectors,old,next) in [
            ([a.encode(),[0;SIZE]],a,b),
            ([a.encode(),b.encode()],b,c),
            ([a.encode(),b.encode()],b,b.fallback().unwrap()),
        ] {
            for operation in 1..=6 {for after in [false,true] {
                publication_fault(sectors,old,next,operation,255,after);
            }}
            for tear in 0..=SIZE {publication_fault(sectors,old,next,3,tear,false);}
        }
    }
    #[test] fn selector_refuses_stale_transition_and_changed_media_without_writes() {
        let (a,b,c)=chain();
        let mut journal=Journal::mount(Media::new([a.encode(),[0;SIZE]])).unwrap();
        let calls=journal.io.calls;
        for bad in [a,c] {
            assert_eq!(journal.commit(bad),Err(PublicationError::Transition));
            assert_eq!(journal.io.calls,calls);assert!(!journal.is_readonly());
        }
        journal.io.live[1]=b.encode();
        assert_eq!(journal.commit(b),Err(PublicationError::Changed));
        assert!(journal.is_readonly());assert!(journal.io.writes.is_empty());
        for fail in [1,2] {
            let mut media=Media::new([a.encode(),[0;SIZE]]);media.fail=fail;
            assert!(matches!(Journal::mount(media),Err(PublicationError::Io)));
        }
        assert!(matches!(Journal::mount(Media::new([[0;SIZE];2])),Err(PublicationError::Corrupt)));
    }

    #[test] fn successful_io_with_wrong_readback_is_indeterminate_and_never_retried() {
        let (a,b,_)=chain();
        // Independently damage the protected current record and written target.
        for damaged in [0,1] {
            let mut journal=Journal::mount(Media::new([a.encode(),[0;SIZE]])).unwrap();
            journal.io.corrupt_after_flush=Some(damaged);
            assert_eq!(journal.commit(b),Err(PublicationError::Indeterminate));
            assert_eq!(journal.record(),a);assert!(journal.is_readonly());
            let calls=journal.io.calls;let writes=journal.io.writes.clone();
            assert_eq!(journal.commit(b),Err(PublicationError::ReadOnly));
            assert_eq!(journal.io.calls,calls);assert_eq!(journal.io.writes,writes);
            assert_eq!(writes,vec![SelectorSector::Second]);
            let recovered=Journal::mount(journal.into_io().reboot()).unwrap();
            assert_eq!(recovered.record(),if damaged==0 {b}else{a});
        }
    }
    #[test] fn successful_fallback_ack_preserves_high_water_across_reboot() {
        let (a,b,_)=chain();let fallback=b.fallback().unwrap();
        let mut journal=Journal::mount(Media::new([a.encode(),b.encode()])).unwrap();
        journal.commit(fallback).unwrap();
        assert_eq!(journal.record(),fallback);assert!(!journal.is_readonly());
        assert_eq!(journal.io.writes,vec![SelectorSector::First]);
        let recovered=Journal::mount(journal.into_io().reboot()).unwrap();
        assert_eq!(recovered.record(),fallback);
        assert_eq!(recovered.record().highest_committed_generation(),b.highest);
        assert_eq!(recovered.record().active(),a.active());
    }

    #[test]fn repair_preserves_highwater_and_has_strict_root_shape(){
        let (a,b,_)=chain();
        for old in [a,b,b.fallback().unwrap()] {
            let repaired=old.repair_id(id(old.active.slot.other(),1)).unwrap();
            assert_eq!(repaired.highest,old.highest);
            assert_eq!(repaired.previous,None);
            assert_eq!(Record::decode(&repaired.encode()),Ok(repaired));
            assert_eq!(select([&old.encode(),&repaired.encode()]).unwrap().record(),repaired);
            assert_eq!(repaired.minimum_install_generation(),old.minimum_install_generation());
            assert_eq!(old.repair_id(id(old.active.slot,1)),Err(Reject::State));
            assert_eq!(old.repair_id(id(old.active.slot.other(),2)),Err(Reject::State));
            let mut wrong=repaired;wrong.highest+=1;
            assert_eq!(select([&old.encode(),&wrong.encode()]),Err(Reject::Ambiguous));
            let mut wrong=repaired;wrong.active.slot=old.active.slot;
            assert_eq!(select([&old.encode(),&wrong.encode()]),Err(Reject::Ambiguous));
            let mut exhausted=old;exhausted.sequence=u64::MAX;
            assert_eq!(exhausted.repair_id(id(old.active.slot.other(),1)),Err(Reject::Exhausted));
        }
    }
    #[test]fn repair_publication_all_io_boundaries_and_513_tears_keep_complete_record(){
        let (a,b,_)=chain();let fallback=b.fallback().unwrap();
        for (sectors,old) in [
            ([a.encode(),[0;SIZE]],a),
            ([a.encode(),b.encode()],b),
            ([fallback.encode(),b.encode()],fallback),
        ] {
            let next=old.repair_id(id(old.active.slot.other(),1)).unwrap();
            for at in 1..=6 {for after in [false,true] {
                publication_fault(sectors,old,next,at,255,after);
            }}
            for tear in 0..=SIZE{publication_fault(sectors,old,next,3,tear,false);}
            let mut journal=Journal::mount(Media::new(sectors)).unwrap();
            journal.commit(next).unwrap();
            let rebooted=Journal::mount(journal.into_io().reboot()).unwrap();
            assert_eq!(rebooted.record(),next);
            assert_eq!(rebooted.record().highest_committed_generation(),old.highest);
        }
    }

    #[test]fn repair_canonical_bytes_and_illegal_semantics_are_rejected(){
        let old=base().install_id(id(Slot::B,2)).unwrap();
        let next=old.repair_id(id(Slot::A,1)).unwrap();
        let encoded=next.encode();
        for p in 0..SIZE{
            let mut bad=encoded;bad[p]^=1;
            assert!(Record::decode(&bad).is_err(),"repair byte {p}");
            assert!(Record::decode(&encoded[..p]).is_err());
        }
        assert!(Record::decode(&[0;SIZE+1]).is_err());
        for offset in [13usize,16,24,56,128] {
            let mut bad=encoded;bad[offset]^=1;
            let checksum=hash(&bad[..480]);bad[480..].copy_from_slice(&checksum);
            if let Ok(selected)=select([&old.encode(),&bad]){
                assert_eq!(selected.record(),old,"illegal repair field {offset}");
            }
        }
        for offset in [12usize,14,32,40,48,64,96] {
            let mut bad=encoded;
            match offset{12=>bad[12]=4,14=>bad[14]=1,64=>bad[64..96].fill(0),_=>bad[offset]^=1}
            let checksum=hash(&bad[..480]);bad[480..].copy_from_slice(&checksum);
            assert!(Record::decode(&bad).is_err(),"repair shape {offset}");
        }
        // Historical decoder's exhaustive kind0..2 match rejects kind3.
        fn legacy_decode(bytes:&[u8;SIZE])->Option<Record>{
            if bytes[12]>2{None}else{Record::decode(bytes).ok()}
        }
        assert_eq!(legacy_decode(&encoded),None);
        assert_eq!(legacy_decode(&old.encode()),Some(old));
        // It may retain old provisionally, but cannot treat Repair as an update
        // or reset high-water. Actual boot must separately authenticate old bytes.
        assert_eq!(legacy_decode(&old.encode()).unwrap().highest,2);
    }

}
