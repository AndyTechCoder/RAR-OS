//! Pure bounded Foundation algorithms. Shared verbatim by kernel and cloud tests.
#![cfg_attr(not(test), no_std)]

pub const PAGE: u64 = 4096;
pub const MAX_REGIONS: usize = 512;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Overflow, Overlap, Exhausted, Stale, Permission }

#[derive(Clone, Copy, Default)]
pub struct Region { pub start: u64, pub pages: u64, pub kind: u32 }
impl Region {
    pub fn end(self) -> Result<u64, Error> {
        if self.pages == 0 || self.start % PAGE != 0 { return Err(Error::Invalid); }
        self.start.checked_add(self.pages.checked_mul(PAGE).ok_or(Error::Overflow)?)
            .ok_or(Error::Overflow)
    }
}
pub fn validate_regions(regions: &[Region]) -> Result<(), Error> {
    if regions.is_empty() || regions.len() > MAX_REGIONS { return Err(Error::Invalid); }
    for (i, &r) in regions.iter().enumerate() {
        let end = r.end()?;
        for &p in &regions[..i] {
            if r.start < p.end()? && p.start < end { return Err(Error::Overlap); }
        }
    }
    Ok(())
}

/// Single-owner physical page allocation. Only conventional UEFI RAM is eligible.
/// Release is deliberately LIFO for this bootstrap manager; other frees fail.
pub struct Frames {
    regions: [Region; MAX_REGIONS],
    next: [u64; MAX_REGIONS],
    count: usize,
    last: Option<(usize, u64)>,
}
impl Frames {
    pub fn new(regions: &[Region]) -> Result<Self, Error> {
        validate_regions(regions)?;
        let mut value = Self {
            regions: [Region::default(); MAX_REGIONS], next: [0; MAX_REGIONS],
            count: regions.len(), last: None,
        };
        value.regions[..regions.len()].copy_from_slice(regions);
        for (i, r) in regions.iter().enumerate() { value.next[i] = r.start.max(PAGE); }
        Ok(value)
    }
    pub fn allocate(&mut self) -> Result<u64, Error> {
        for i in 0..self.count {
            let r = self.regions[i];
            let p = self.next[i];
            if r.kind == 7 && p.checked_add(PAGE).is_some_and(|e| e <= r.end().unwrap() && e <= 0x1_0000_0000) {
                self.next[i] += PAGE;
                self.last = Some((i,p));
                return Ok(p);
            }
        }
        Err(Error::Exhausted)
    }
    pub fn release_last(&mut self, page: u64) -> Result<(), Error> {
        let (i,p) = self.last.ok_or(Error::Stale)?;
        if p != page { return Err(Error::Stale); }
        self.next[i] = p;
        self.last = None;
        Ok(())
    }
}

pub fn canonical(address: u64) -> bool {
    address < 0x0000_8000_0000_0000 || address >= 0xffff_8000_0000_0000
}
#[derive(Clone, Copy)]
pub struct Mapping { pub virtual_start: u64, pub physical_start: u64, pub pages: u64,
                     pub writable: bool, pub executable: bool }
impl Mapping {
    pub fn validate(self, owned_start: u64, owned_end: u64) -> Result<(), Error> {
        if self.pages == 0 || self.virtual_start == 0 ||
            self.virtual_start % PAGE != 0 || self.physical_start % PAGE != 0 {
            return Err(Error::Invalid);
        }
        if self.writable && self.executable { return Err(Error::Permission); }
        let bytes = self.pages.checked_mul(PAGE).ok_or(Error::Overflow)?;
        let vend = self.virtual_start.checked_add(bytes - 1).ok_or(Error::Overflow)?;
        let pend = self.physical_start.checked_add(bytes).ok_or(Error::Overflow)?;
        if !canonical(self.virtual_start) || !canonical(vend) ||
            (self.virtual_start >> 47) != (vend >> 47) { return Err(Error::Invalid); }
        if pend > (1u64<<52) || self.physical_start < owned_start || pend > owned_end { return Err(Error::Permission); }
        Ok(())
    }
}

/// A 64 KiB heap split into 4096 sixteen-byte units. Allocation records carry
/// monotonic identities so stale, forged and double-free handles are rejected.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Allocation { pub offset: usize, pub size: usize, id: u64 }
#[derive(Clone, Copy)]
struct Slot { allocation: Allocation, units: usize }
pub struct Heap {
    bitmap: [u64; 64],
    slots: [Option<Slot>; 128],
    next_id: u64,
}
impl Heap {
    pub const fn new() -> Self { Self { bitmap: [0; 64], slots: [None; 128], next_id: 1 } }
    fn occupied(&self, i: usize) -> bool { self.bitmap[i / 64] & (1 << (i % 64)) != 0 }
    fn mark(&mut self, i: usize, used: bool) {
        if used { self.bitmap[i/64] |= 1 << (i%64); }
        else { self.bitmap[i/64] &= !(1 << (i%64)); }
    }
    pub fn allocate(&mut self, size: usize, align: usize) -> Result<Allocation, Error> {
        if size == 0 || size > 65536 || !align.is_power_of_two() || align > 4096 {
            return Err(Error::Invalid);
        }
        let slot = self.slots.iter().position(Option::is_none).ok_or(Error::Exhausted)?;
        let units = size.checked_add(15).ok_or(Error::Overflow)? / 16;
        let id = self.next_id;
        let next_id = id.checked_add(1).ok_or(Error::Exhausted)?;
        let step = align.max(16)/16;
        for start in (0..=4096-units).step_by(step) {
            if (start..start+units).all(|i| !self.occupied(i)) {
                let allocation = Allocation { offset: start*16, size, id };
                for i in start..start+units { self.mark(i, true); }
                self.slots[slot] = Some(Slot { allocation, units });
                self.next_id = next_id;
                return Ok(allocation);
            }
        }
        Err(Error::Exhausted)
    }
    pub fn deallocate(&mut self, allocation: Allocation) -> Result<(), Error> {
        let slot = self.slots.iter().position(|s| s.is_some_and(|s| s.allocation == allocation))
            .ok_or(Error::Stale)?;
        let record = self.slots[slot].take().ok_or(Error::Stale)?;
        for i in allocation.offset/16..allocation.offset/16+record.units { self.mark(i, false); }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn invalid_memory_maps_fail() {
        assert!(validate_regions(&[]).is_err());
        assert!(validate_regions(&[Region{start:1,pages:1,kind:7}]).is_err());
        assert!(validate_regions(&[Region{start:4096,pages:u64::MAX,kind:7}]).is_err());
        assert!(validate_regions(&[Region{start:4096,pages:2,kind:7},Region{start:8192,pages:1,kind:7}]).is_err());
    }
    #[test] fn only_conventional_frames_are_allocated() {
        let mut f=Frames::new(&[Region{start:4096,pages:1,kind:2},
                               Region{start:8192,pages:2,kind:7}]).unwrap();
        assert_eq!(f.allocate(),Ok(8192)); assert_eq!(f.allocate(),Ok(12288));
        assert_eq!(f.allocate(),Err(Error::Exhausted));
        assert_eq!(f.release_last(8192),Err(Error::Stale));
        assert_eq!(f.release_last(12288),Ok(()));
        assert_eq!(f.release_last(12288),Err(Error::Stale));
        assert_eq!(f.allocate(),Ok(12288));
    }
    #[test] fn null_and_high_frames_are_not_allocated() {
        let mut f=Frames::new(&[Region{start:0,pages:1,kind:7},
                               Region{start:0x1_0000_0000,pages:1,kind:7}]).unwrap();
        assert_eq!(f.allocate(),Err(Error::Exhausted));
    }
    #[test] fn mapping_permissions_and_bounds() {
        let m=Mapping{virtual_start:0x1000,physical_start:0x2000,pages:1,writable:true,executable:false};
        assert_eq!(m.validate(0x2000,0x3000),Ok(()));
        assert_eq!(Mapping{executable:true,..m}.validate(0x2000,0x3000),Err(Error::Permission));
        assert!(Mapping{virtual_start:0,..m}.validate(0,0x4000).is_err());
        assert!(Mapping{virtual_start:0x0000_8000_0000_0000,..m}.validate(0,0x4000).is_err());
        assert!(Mapping{pages:u64::MAX,..m}.validate(0,u64::MAX).is_err());
        assert!(m.validate(0x3000,0x4000).is_err());
        assert!(Mapping{physical_start:0x2001,..m}.validate(0,0x4000).is_err());
    }
    #[test] fn heap_alignment_exhaustion_and_reuse() {
        let mut h=Heap::new();
        let a=h.allocate(17,4096).unwrap();
        let b=h.allocate(32,4096).unwrap();
        assert_eq!(a.offset,0); assert_eq!(b.offset,4096);
        assert_eq!(h.deallocate(a),Ok(())); assert_eq!(h.deallocate(a),Err(Error::Stale));
        let c=h.allocate(17,4096).unwrap();
        assert_eq!(c.offset,0); assert_ne!(a,c); assert_eq!(h.deallocate(a),Err(Error::Stale));
        h.deallocate(b).unwrap(); h.deallocate(c).unwrap();
        let all=h.allocate(65536,16).unwrap(); assert_eq!(h.allocate(1,1),Err(Error::Exhausted));
        h.deallocate(all).unwrap(); assert!(h.allocate(65536,4096).is_ok());
    }
    #[test] fn heap_rejects_invalid_and_forged_requests() {
        let mut h=Heap::new();
        for (s,a) in [(0,1),(1,0),(1,3),(65537,16),(1,8192),(usize::MAX,16)] {
            assert!(h.allocate(s,a).is_err());
        }
        let a=h.allocate(16,16).unwrap();
        assert_eq!(h.deallocate(Allocation{offset:16,..a}),Err(Error::Stale));
        assert_eq!(h.deallocate(Allocation{size:32,..a}),Err(Error::Stale));
        h.deallocate(a).unwrap();
    }
    #[test] fn heap_adversarial_sequence_never_overlaps() {
        let mut h=Heap::new();
        let mut live=[None;64];
        for tick in 0..4096 {
            let slot=tick%64;
            if let Some(a)=live[slot].take() { h.deallocate(a).unwrap(); }
            let a=h.allocate((tick*71)%511+1,1<<((tick%8)+1)).unwrap();
            for other in live.iter().flatten() {
                assert!(a.offset+a.size<=other.offset || other.offset+other.size<=a.offset);
            }
            live[slot]=Some(a);
        }
    }
}

pub fn validate_table(bytes:&[u8], signature:u64, minimum:usize)->Result<(),Error> {
    if bytes.len()<24 || bytes.len()>4096 || bytes.len()<minimum {return Err(Error::Invalid);}
    let u32at=|i|u32::from_le_bytes(bytes[i..i+4].try_into().unwrap());
    if u64::from_le_bytes(bytes[..8].try_into().unwrap())!=signature ||
        u32at(8)<0x0002_0000 || u32at(12) as usize!=bytes.len() || u32at(20)!=0 {
        return Err(Error::Invalid);
    }
    let mut crc=0xffff_ffffu32;
    for (i, &b) in bytes.iter().enumerate() {
        crc^=if (16..20).contains(&i) {0} else {b as u32};
        for _ in 0..8 {crc=(crc>>1)^if crc&1!=0 {0xedb8_8320} else {0};}
    }
    if !crc!=u32at(16) {return Err(Error::Invalid);}
    Ok(())
}
#[cfg(test)]
mod handoff_tests {
    use super::*;
    #[test] fn rejects_truncated_and_corrupt_headers() {
        assert!(validate_table(&[],0,24).is_err());
        let mut b=[0u8;24];
        b[..8].copy_from_slice(&0x5453595320494249u64.to_le_bytes());
        b[8..12].copy_from_slice(&0x0002_0000u32.to_le_bytes());
        b[12..16].copy_from_slice(&24u32.to_le_bytes());
        let mut crc=0xffff_ffffu32;
        for &v in &b {crc^=v as u32;for _ in 0..8 {crc=(crc>>1)^if crc&1!=0 {0xedb8_8320}else{0};}}
        b[16..20].copy_from_slice(&(!crc).to_le_bytes());
        assert_eq!(validate_table(&b,0x5453595320494249,24),Ok(()));
        b[23]=1; assert!(validate_table(&b,0x5453595320494249,24).is_err());
        b[23]=0; b[8]^=1; assert!(validate_table(&b,0x5453595320494249,24).is_err());
        assert!(validate_table(&b,0,24).is_err());
    }
}

#[cfg(test)]
#[path = "image.rs"]
mod image_tests;

pub fn validate_virtual_page(address:u64)->Result<(),Error> {
    if address==0 || address%PAGE!=0 || !canonical(address) {Err(Error::Invalid)} else {Ok(())}
}
#[cfg(test)]
mod virtual_page_tests {
    use super::*;
    #[test] fn rejects_noncanonical_alias_before_page_walk() {
        assert_eq!(validate_virtual_page(0xffff_8000_0010_0000),Ok(()));
        assert_eq!(validate_virtual_page(0x0000_8000_0010_0000),Err(Error::Invalid));
        assert_eq!(validate_virtual_page(0),Err(Error::Invalid));
        assert_eq!(validate_virtual_page(0x1001),Err(Error::Invalid));
        assert_eq!(validate_virtual_page(0x0000_7fff_ffff_f000),Ok(()));
    }
}
