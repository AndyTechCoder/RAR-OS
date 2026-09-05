//! Modern-v0 System selector codec/transition model. No block I/O or Data handle.
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
enum Kind { Factory, Install, Fallback }
impl Kind {
    fn byte(self) -> u8 { match self { Self::Factory => 0, Self::Install => 1, Self::Fallback => 2 } }
    fn parse(v: u8) -> Result<Self, Reject> {
        match v { 0 => Ok(Self::Factory), 1 => Ok(Self::Install), 2 => Ok(Self::Fallback), _ => Err(Reject::State) }
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
        let (a,b,c)=chain();
        // Sector0 is older A; B in sector1 remains untouched during C publication.
        for cut in 0..=SIZE {
            let mut sector=a.encode(); sector[..cut].copy_from_slice(&c.encode()[..cut]);
            let selected=select([&sector,&b.encode()]).unwrap();
            assert!(selected.record==b || selected.record==c,"cut {cut}");
            if cut==SIZE {assert_eq!(selected.record,c);}
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
}
