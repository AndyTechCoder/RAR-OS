//! Host-only codec conformance tests; fixture PE bytes are never executed.
#![forbid(unsafe_code)]
#[path = "../../../core/crypto/sha256.rs"] mod sha256;
#[path = "../../../core/crypto/sha512.rs"] mod sha512;
#[path = "../../../core/crypto/ed25519.rs"] mod ed25519;
#[path = "../../../core/modern/manifest.rs"] mod manifest;
#[path = "../../../core/modern/journal.rs"] mod journal;
#[path = "../../../core/modern/system_volume.rs"] mod system_volume;
#[derive(Clone, Copy, Debug, PartialEq, Eq)] pub enum Error { Invalid, Denied }
#[path = "../../../nucleus/platform/pe.rs"] mod pe;

const FIXTURE: &[u8; 4 * (384 + 1024)] =
    include_bytes!(env!("RAR_LAB_SIGNED_CODEC_FIXTURE"));
fn case(index: usize) -> (&'static [u8], &'static [u8]) {
    let start = index * (384 + 1024);
    (&FIXTURE[start..start+384], &FIXTURE[start+384..start+384+1024])
}
#[test]
fn separately_generated_signed_package_reaches_verified_layer() {
    let (raw, payload) = case(0);
    let verified = manifest::verify(raw, payload, 1).unwrap();
    assert_eq!(verified.manifest().generation(), 1);
    assert_eq!(verified.payload(), payload);
    assert_eq!(verified.layout().entry, 0x401000);
    assert_eq!(verified.layout().image_size, 8192);
    assert_eq!(verified.manifest().payload_digest(), sha256::sha256(payload).unwrap());
}
#[test]
fn authenticated_compatibility_budget_and_executable_rules_still_apply() {
    let (raw, payload) = case(0);
    assert!(matches!(manifest::verify(raw, payload, 2), Err(manifest::Reject::Rollback)));
    let (raw, payload) = case(1);
    assert!(matches!(manifest::verify(raw, payload, 1), Err(manifest::Reject::Compatibility)));
    let (raw, payload) = case(2);
    assert_eq!(manifest::verify(raw, payload, 2).unwrap().manifest().generation(), 2);
    let (raw, payload) = case(3);
    assert!(matches!(manifest::verify(raw, payload, 1), Err(manifest::Reject::Executable)));
}
#[test]
fn manifest_bytes_and_representative_payload_tampering_do_not_authenticate() {
    let (raw, payload) = case(0);
    for offset in 0..384 {
        let mut changed = raw.to_vec();
        changed[offset] ^= 1;
        assert!(manifest::verify(&changed, payload, 1).is_err(), "manifest byte {offset}");
    }
    for offset in [0,1,60,64,112,328,364,511,512,513,1023] {
        let mut changed = payload.to_vec();
        changed[offset] ^= 1;
        assert!(matches!(manifest::verify(raw, &changed, 1), Err(manifest::Reject::PayloadDigest)),
            "payload byte {offset}");
    }
}

mod system_media {
    use super::*;
    use std::{cell::RefCell,collections::BTreeMap,rc::Rc};
    use system_volume::{Io,Volume,Reject,SECTORS,MAX_PACKAGE,slot_start};
    use journal::{Record,Slot};
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    enum Op{Read(u32),Write(u32),Flush}
    #[derive(Default)]
    struct Disk{blocks:BTreeMap<u32,[u8;512]>,ops:Vec<Op>,fail:Option<usize>,lie:bool}
    #[derive(Clone)]
    struct Media(Rc<RefCell<Disk>>);
    impl Io for Media {
        fn read(&mut self,s:u32)->Result<[u8;512],()>{
            let mut d=self.0.borrow_mut();d.ops.push(Op::Read(s));
            if d.fail==Some(d.ops.len()){return Err(());}
            Ok(*d.blocks.get(&s).unwrap_or(&[0;512]))
        }
        fn write(&mut self,s:u32,b:&[u8;512])->Result<(),()>{
            let mut d=self.0.borrow_mut();d.ops.push(Op::Write(s));
            if d.fail==Some(d.ops.len()){return Err(());}
            if !d.lie{d.blocks.insert(s,*b);}Ok(())
        }
        fn flush(&mut self)->Result<(),()>{
            let mut d=self.0.borrow_mut();d.ops.push(Op::Flush);
            if d.fail==Some(d.ops.len()){return Err(());}Ok(())
        }
    }
    fn package(index:usize)->Vec<u8>{
        let (raw,payload)=case(index);[raw,payload].concat()
    }
    fn seed()->(Media,Record,Record){
        let (raw,payload)=case(0);
        let old=Record::factory(&manifest::verify(raw,payload,1).unwrap());
        let (raw,payload)=case(2);
        let next=old.install(&manifest::verify(raw,payload,2).unwrap()).unwrap();
        let mut d=Disk::default();d.blocks.insert(0,old.encode());
        for (i,part) in package(0).chunks(512).enumerate(){
            let mut block=[0;512];block[..part.len()].copy_from_slice(part);
            d.blocks.insert(slot_start(Slot::A)+i as u32,block);
        }
        (Media(Rc::new(RefCell::new(d))),old,next)
    }
    fn mount(media:Media)->Volume<Media>{Volume::mount(media,SECTORS).unwrap()}
    #[test] fn system_preparation_is_inactive_durable_and_publication_is_bound(){
        let (media,old,next)=seed();let original=media.0.borrow().blocks.clone();
        let mut volume=mount(media.clone());let data=package(2);
        let prepared=volume.prepare(&data).unwrap();
        assert_eq!(prepared.identity().slot,Slot::B);
        assert_eq!(prepared.identity().length,data.len());
        assert_eq!(prepared.identity().package_hash,sha256::sha256(&data).unwrap());
        let before=media.0.borrow().ops.clone();
        assert!(before.contains(&Op::Flush));
        assert_eq!(before.iter().filter(|op|matches!(op,Op::Write(_))).count(),3);
        for op in &before{if let Op::Write(s)=op{assert!((4099..4102).contains(s));}}
        for (sector,bytes) in &original{assert_eq!(media.0.borrow().blocks.get(sector),Some(bytes));}
        let mut copied=Vec::new();
        assert_eq!(volume.copy_package(Slot::B,|length,offset,chunk|{
            assert_eq!(length,data.len());assert_eq!(offset,copied.len());
            assert!(chunk.len()<=512);copied.extend_from_slice(chunk);Ok(())
        }).unwrap(),(data.len(),sha256::sha256(&data).unwrap()));
        assert_eq!(copied,data);
        assert_eq!(volume.record(),old);
        volume.publish(prepared,next).unwrap();
        assert_eq!(volume.record(),next);
        assert_eq!(mount(media.clone()).record(),next);
        assert_eq!(media.0.borrow().blocks.get(&0),Some(&old.encode()));
        assert!(!volume.is_readonly());
    }
    #[test] fn system_every_prepare_io_failure_locks_without_publication_or_retry(){
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        media.0.borrow_mut().ops.clear();volume.prepare(&package(2)).unwrap();
        let operations=media.0.borrow().ops.len();
        for cut in 1..=operations{
            let (media,old,_)=seed();let mut volume=mount(media.clone());
            {let mut d=media.0.borrow_mut();d.ops.clear();d.fail=Some(cut);}
            assert!(volume.prepare(&package(2)).is_err(),"{cut}");
            assert!(volume.is_readonly());
            let count=media.0.borrow().ops.len();
            assert!(matches!(volume.prepare(&package(2)),Err(Reject::ReadOnly)));
            assert_eq!(media.0.borrow().ops.len(),count);
            assert_eq!(media.0.borrow().blocks.get(&0),Some(&old.encode()));
            assert!(!media.0.borrow().ops.iter().any(|op|matches!(op,Op::Write(0|1))));
        }
    }
    #[test] fn system_every_publication_io_failure_is_sticky_and_preserves_old_selector(){
        let (media,_,next)=seed();let mut volume=mount(media.clone());
        let prepared=volume.prepare(&package(2)).unwrap();media.0.borrow_mut().ops.clear();
        volume.publish(prepared,next).unwrap();let operations=media.0.borrow().ops.clone();
        let write=operations.iter().position(|op|matches!(op,Op::Write(_))).unwrap()+1;
        for cut in 1..=operations.len(){
            let (media,old,next)=seed();let mut volume=mount(media.clone());
            let prepared=volume.prepare(&package(2)).unwrap();
            {let mut d=media.0.borrow_mut();d.ops.clear();d.fail=Some(cut);}
            let result=volume.publish(prepared,next);
            assert!(result.is_err(),"{cut}");
            if cut>=write{assert_eq!(result,Err(Reject::Indeterminate),"{cut}");}
            assert!(volume.is_readonly());
            assert_eq!(media.0.borrow().blocks.get(&0),Some(&old.encode()));
            media.0.borrow_mut().fail=None;
            let selected=mount(media.clone()).record();
            assert!(selected==old||selected==next);
        }
    }
    #[test] fn system_stale_token_and_record_mismatch_cannot_publish(){
        let (media,old,next)=seed();let mut volume=mount(media.clone());
        let stale=volume.prepare(&package(2)).unwrap();
        let current=volume.prepare(&package(2)).unwrap();
        let count=media.0.borrow().ops.len();
        assert_eq!(volume.publish(stale,next),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);
        assert_eq!(volume.publish(current,old),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);
        assert_eq!(volume.record(),old);
    }
    #[test] fn system_changed_package_false_write_ack_and_selector_changes_fail_closed(){
        let (media,_,next)=seed();let mut volume=mount(media.clone());
        let prepared=volume.prepare(&package(2)).unwrap();
        media.0.borrow_mut().blocks.get_mut(&4100).unwrap()[7]^=1;
        assert_eq!(volume.publish(prepared,next),Err(Reject::Changed));assert!(volume.is_readonly());
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        media.0.borrow_mut().lie=true;
        assert!(matches!(volume.prepare(&package(2)),Err(Reject::Changed)));assert!(volume.is_readonly());
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        media.0.borrow_mut().blocks.insert(0,[0;512]);
        assert!(volume.prepare(&package(2)).is_err());assert!(volume.is_readonly());
        assert!(!media.0.borrow().ops.iter().any(|op|matches!(op,Op::Write(_))));
    }
    #[test] fn system_invalid_content_allows_bounded_fallback_read_but_io_does_not(){
        let (media,_,_)=seed();let mut volume=mount(media.clone());let mut out=vec![0;MAX_PACKAGE];
        assert_eq!(volume.read_package(Slot::B,&mut out),Err(Reject::Framing));
        assert!(!volume.is_readonly());
        assert_eq!(volume.read_package(Slot::A,&mut out),Ok(package(0).len()));
        assert_eq!(&out[..package(0).len()],package(0));
        let count=media.0.borrow().ops.len();media.0.borrow_mut().fail=Some(count+1);
        assert_eq!(volume.copy_package(Slot::A,|_,_,_|Ok(())),Err(Reject::Io));
        assert!(volume.is_readonly());
    }
    #[test] fn system_length_padding_and_capacity_are_bounded_before_effects(){
        let (media,_,_)=seed();
        assert!(matches!(Volume::mount(media.clone(),SECTORS-1),Err(Reject::Capacity)));
        assert!(media.0.borrow().ops.is_empty());
        let mut volume=mount(media.clone());let good=package(2);
        for length in [0,383,511,895,good.len()-1]{
            let count=media.0.borrow().ops.len();
            assert!(volume.prepare(&good[..length]).is_err());
            assert_eq!(media.0.borrow().ops.len(),count);assert!(!volume.is_readonly());
        }
        for length in [0u32,511,2_097_153,u32::MAX]{
            let mut bad=good.clone();bad[56..60].copy_from_slice(&length.to_le_bytes());
            let count=media.0.borrow().ops.len();assert!(volume.prepare(&bad).is_err());
            assert_eq!(media.0.borrow().ops.len(),count);
        }
        let prepared=volume.prepare(&good).unwrap();
        assert_eq!(prepared.identity().length,1408);
        media.0.borrow_mut().blocks.get_mut(&4101).unwrap()[384]=1;
        assert_eq!(volume.copy_package(Slot::B,|_,_,_|Ok(())),Err(Reject::Framing));
        assert!(!volume.is_readonly());
    }
}
