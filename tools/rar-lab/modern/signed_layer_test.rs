//! Host-only codec conformance tests; fixture PE bytes are never executed.
#![forbid(unsafe_code)]
#[path = "../../../core/crypto/sha256.rs"] mod sha256;
#[path = "../../../core/crypto/sha512.rs"] mod sha512;
#[path = "../../../core/crypto/ed25519.rs"] mod ed25519;
#[path = "../../../core/modern/manifest.rs"] mod manifest;
#[path = "../../../core/modern/journal.rs"] mod journal;
#[path = "../../../core/modern/repair.rs"] mod repair;
#[path = "../../../core/modern/system_volume.rs"] mod system_volume;
#[path = "../../../core/modern/update_wire.rs"] mod update_wire;
#[path = "../../../core/modern/update_system.rs"] mod update_system;
#[path = "../../../core/modern/update_manager.rs"] mod update_manager;
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


    #[derive(Default)]struct CopyStage{bytes:Vec<u8>,seal:u64,finished:bool,aborted:bool,fail_append:bool,fail_begin:bool,fail_finish:bool,fail_abort:bool}
    impl update_system::Stage for CopyStage{
        fn begin(&mut self,_:usize)->Result<u64,()>{if self.fail_begin{return Err(());}self.seal=self.seal.checked_add(1).ok_or(())?;
            self.bytes.clear();self.finished=false;self.aborted=false;Ok(self.seal)}
        fn append(&mut self,seal:u64,offset:usize,bytes:&[u8])->Result<(),()>{
            if self.fail_append||seal!=self.seal||offset!=self.bytes.len(){return Err(());}
            self.bytes.extend_from_slice(bytes);Ok(())}
        fn finish(&mut self,seal:u64,length:usize)->Result<(),()>{
            if self.fail_finish||seal!=self.seal||length!=self.bytes.len(){return Err(());}self.finished=true;Ok(())}
        fn abort(&mut self,seal:u64)->Result<(),()>{if self.fail_abort||seal!=self.seal{return Err(());}
            self.aborted=true;self.bytes.clear();Ok(())}
    }
    fn lab_input(index:u64)->Option<&'static[u8]>{
        if index==0{Some(&FIXTURE[2*1408..3*1408])}else{None}
    }
    fn frame(reply:update_system::Reply)->[u8;128]{match reply{
        update_system::Reply::Frame(f)=>f,other=>panic!("expected frame: {other:?}")}}
    fn handle(server:&mut update_system::Server<Media>,stage:&mut CopyStage,bytes:&[u8])->update_system::Reply{
        server.handle(8,(1u64<<40)+1,bytes,stage,lab_input)
    }
    fn fetch(server:&mut update_system::Server<Media>,stage:&mut CopyStage,t:update_wire::Transfer)->Record{
        use update_wire::{Kind,RecordReceiver,PART};
        let mut rx=RecordReceiver::new(t,Kind::RecordPart).unwrap();
        for offset in (0..512).step_by(PART){
            let get=t.part(Kind::RecordGet,offset,&[]).unwrap();
            let f=frame(handle(server,stage,&get));rx.push(&f).unwrap();
            assert_eq!(handle(server,stage,&get),update_system::Reply::Ignore);
        }
        rx.finish().unwrap()
    }
    #[test]fn update_session_boot_authentication_replay_and_exact_readonly_ack(){
        use update_wire::{self as w,Mode,Kind,Transfer};
        use update_system::{Server,Reply};
        let (media,old,_)=seed();let mut s=Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
        let mut stage=CopyStage::default();let start=w::request(Kind::Start,Mode::Boot,1,0).unwrap();
        let before=media.0.borrow().ops.len();
        for (sender,inc) in [(9,(1u64<<40)+1),(8,1),(4,(1u64<<40)+1)]{
            assert_eq!(s.handle(sender,inc,&start,&mut stage,|_|panic!("unauthorized lookup")),Reply::Ignore);
        }
        assert_eq!(media.0.borrow().ops.len(),before);assert_eq!(stage.seal,0);
        let offer=frame(handle(&mut s,&mut stage,&start));let t=Transfer::parse(&offer,Kind::Offer).unwrap();
        assert!(stage.finished);assert_eq!(stage.bytes,package(0));
        let count=media.0.borrow().ops.len();
        assert_eq!(handle(&mut s,&mut stage,&start),Reply::Ignore);
        assert_eq!(handle(&mut s,&mut stage,&t.frame(Kind::Commit).unwrap()),Reply::Ignore);
        assert_eq!(media.0.borrow().ops.len(),count);
        let record=fetch(&mut s,&mut stage,t);assert_eq!(record,old);
        let v=update_manager::verify(t,record,&stage.bytes).unwrap();
        assert!(v.next().is_none());assert_eq!(v.layer().manifest().generation(),1);
        let expected=v.expected_ack().unwrap();
        let mut bad=t;bad.identity.transaction+=1;
        assert_eq!(handle(&mut s,&mut stage,&bad.frame(Kind::Commit).unwrap()),Reply::Ignore);
        let commit=t.frame(Kind::Commit).unwrap();
        assert_eq!(frame(handle(&mut s,&mut stage,&commit)),expected);
        let count=media.0.borrow().ops.len();
        assert_eq!(handle(&mut s,&mut stage,&commit),Reply::Ignore);
        assert_eq!(handle(&mut s,&mut stage,&start),Reply::Ignore);
        assert_eq!(media.0.borrow().ops.len(),count);
        assert!(!media.0.borrow().ops.iter().any(|o|matches!(o,Op::Write(_)|Op::Flush)));
    }
    #[test]fn update_session_install_requires_exact_verified_record_and_transaction(){
        use update_wire::{self as w,Kind,Mode,Transfer,PART};
        let (media,old,next)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
        let mut stage=CopyStage::default();
        let start=w::request(Kind::Start,Mode::Install,1,0).unwrap();
        let t=Transfer::parse(&frame(handle(&mut s,&mut stage,&start)),Kind::Offer).unwrap();
        let record=fetch(&mut s,&mut stage,t);assert_eq!(record,old);
        let v=update_manager::verify(t,record,&stage.bytes).unwrap();
        assert_eq!(v.next(),Some(next));let expected=v.expected_ack().unwrap();
        let encoded=next.encode();
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);let part=t.part(Kind::PublishPart,offset,&encoded[offset..offset+n]).unwrap();
            let ack=frame(handle(&mut s,&mut stage,&part));
            assert_eq!(ack,t.part(Kind::PartAck,offset,&[]).unwrap());
            assert_eq!(handle(&mut s,&mut stage,&part),update_system::Reply::Ignore);
        }
        assert_eq!(frame(handle(&mut s,&mut stage,&t.frame(Kind::Commit).unwrap())),expected);
        assert_eq!(mount(media).record(),next);
    }
    #[test]fn update_session_partial_copy_is_aborted_and_owner_never_retries(){
        use update_wire::{self as w,Kind,Mode};
        let (media,_,_)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
        let mut stage=CopyStage{fail_append:true,..CopyStage::default()};
        let start=w::request(Kind::Start,Mode::Boot,1,0).unwrap();
        assert_eq!(handle(&mut s,&mut stage,&start),update_system::Reply::Halt);
        assert!(stage.aborted);assert!(!stage.finished);assert!(stage.bytes.is_empty());
        let count=media.0.borrow().ops.len();
        stage.fail_append=false;
        assert_eq!(handle(&mut s,&mut stage,&start),update_system::Reply::Halt);
        assert_eq!(media.0.borrow().ops.len(),count);
    }
    #[test]fn update_manager_rejects_substituted_record_package_and_ack_identity(){
        use update_wire::{Transfer,Mode,Kind};
        let (media,old,next)=seed();let mut volume=mount(media);
        let p=volume.prepare(&package(2)).unwrap();
        let t=Transfer{mode:Mode::Install,request:1,seal:2,sequence:old.sequence(),identity:p.identity()};
        let bytes=package(2);
        assert!(update_manager::verify(t,next,&bytes).is_err());
        assert!(update_manager::verify(t,old,&package(0)).is_err());
        for change in 0..4{
            let mut bad=t;
            match change{0=>bad.identity.generation+=1,1=>bad.identity.digest[0]^=1,
                2=>bad.identity.slot=bad.identity.slot.other(),_=>bad.identity.package_hash[0]^=1}
            assert!(update_manager::verify(bad,old,&bytes).is_err());
        }
        let v=update_manager::verify(t,old,&bytes).unwrap();
        assert_ne!(v.expected_ack().unwrap(),t.frame(Kind::Committed).unwrap());
    }


    fn prepared_flow(s:&mut update_system::Server<Media>,stage:&mut CopyStage,mode:update_wire::Mode,id:u64)
        ->(update_wire::Transfer,Record,Option<Record>,[u8;128]){
        use update_wire::{self as w,Kind,Transfer};
        let start=w::request(Kind::Start,mode,id,0).unwrap();
        let t=Transfer::parse(&frame(handle(s,stage,&start)),Kind::Offer).unwrap();
        let record=fetch(s,stage,t);
        let v=update_manager::verify(t,record,&stage.bytes).unwrap();
        (t,record,v.next(),v.expected_ack().unwrap())
    }
    fn publish_parts(s:&mut update_system::Server<Media>,stage:&mut CopyStage,t:update_wire::Transfer,next:Record){
        use update_wire::{Kind,PART};let b=next.encode();
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);let f=t.part(Kind::PublishPart,offset,&b[offset..offset+n]).unwrap();
            assert_eq!(frame(handle(s,stage,&f)),t.part(Kind::PartAck,offset,&[]).unwrap());
        }
    }
    #[test]fn update_session_fallback_is_exact_prior_and_preserves_high_water(){
        use update_wire::{Mode,Kind};
        let (media,old,next)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
        let mut stage=CopyStage::default();
        let(t,_,record,ack)=prepared_flow(&mut s,&mut stage,Mode::Install,1);
        publish_parts(&mut s,&mut stage,t,record.unwrap());
        assert_eq!(frame(handle(&mut s,&mut stage,&t.frame(Kind::Commit).unwrap())),ack);
        let before=media.0.borrow().ops.len();
        let(t,current,record,ack)=prepared_flow(&mut s,&mut stage,Mode::Fallback,2);
        assert_eq!(current,next);assert_eq!(t.identity.generation,old.active().generation());
        let record=record.unwrap();assert_eq!(record.highest_committed_generation(),2);
        assert_eq!(record.active(),old.active());assert!(record.previous().is_none());
        publish_parts(&mut s,&mut stage,t,record);
        assert_eq!(frame(handle(&mut s,&mut stage,&t.frame(Kind::Commit).unwrap())),ack);
        assert!(media.0.borrow().ops[before..].iter().all(|op|!matches!(op,Op::Write(n)if *n>=2)));
        assert_eq!(mount(media).record(),record);
    }
    #[test]fn update_session_cancel_and_begin_rejection_preserve_same_session_liveness(){
        use update_wire::{self as w,Mode,Kind,Transfer};use update_system::Reply;
        let(media,_,_)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
        let mut stage=CopyStage{fail_begin:true,..CopyStage::default()};
        let start=w::request(Kind::Start,Mode::Boot,1,0).unwrap();
        assert_eq!(frame(handle(&mut s,&mut stage,&start)),w::request(Kind::Rejected,Mode::Boot,1,0).unwrap());
        assert_eq!(stage.seal,0);assert!(stage.bytes.is_empty());assert!(!stage.finished);
        stage.fail_begin=false;
        let start=w::request(Kind::Start,Mode::Boot,2,0).unwrap();
        let t=Transfer::parse(&frame(handle(&mut s,&mut stage,&start)),Kind::Offer).unwrap();
        let count=media.0.borrow().ops.len();
        let cancel=t.frame(Kind::Cancel).unwrap();
        for(sender,inc)in[(8,1),(9,(1u64<<40)+1),(4,(1u64<<40)+1)]{
            assert_eq!(s.handle(sender,inc,&cancel,&mut stage,lab_input),Reply::Ignore);
        }
        assert_eq!(media.0.borrow().ops.len(),count);
        // Simulates the trusted manager's completed native clear before Cancel.
        stage.bytes.clear();stage.finished=false;
        assert_eq!(frame(handle(&mut s,&mut stage,&cancel)),t.frame(Kind::Cancelled).unwrap());
        assert_eq!(handle(&mut s,&mut stage,&cancel),Reply::Ignore);
        let start=w::request(Kind::Start,Mode::Boot,3,0).unwrap();
        let next=Transfer::parse(&frame(handle(&mut s,&mut stage,&start)),Kind::Offer).unwrap();
        assert!(next.identity.transaction>t.identity.transaction);assert!(next.seal>t.seal);
    }
    #[test]fn update_session_finish_and_abort_failures_never_offer_or_retry(){
        use update_wire::{self as w,Mode,Kind};
        for abort in [false,true]{
            let(media,_,_)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
            let mut stage=CopyStage{fail_finish:true,fail_abort:abort,..CopyStage::default()};
            let start=w::request(Kind::Start,Mode::Boot,1,0).unwrap();
            assert_eq!(handle(&mut s,&mut stage,&start),update_system::Reply::Halt);
            assert!(!stage.finished);assert_eq!(stage.aborted,!abort);
            let count=media.0.borrow().ops.len();
            assert_eq!(handle(&mut s,&mut stage,&start),update_system::Reply::Halt);
            assert_eq!(media.0.borrow().ops.len(),count);
        }
    }
    #[test]fn update_session_substituted_valid_proposal_and_bad_record_halt_without_ack(){
        use update_wire::{Mode,Kind,PART};
        for malformed in [false,true]{
            let(media,old,_)=seed();let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
            let mut stage=CopyStage::default();let(t,_,_,_)=prepared_flow(&mut s,&mut stage,Mode::Install,1);
            let count=media.0.borrow().ops.len();
            let b=if malformed{[0u8;512]}else{old.encode()};
            for offset in (0..512).step_by(PART){
                let n=(512-offset).min(PART);let f=t.part(Kind::PublishPart,offset,&b[offset..offset+n]).unwrap();
                let reply=handle(&mut s,&mut stage,&f);
                if malformed&&offset==440{assert_eq!(reply,update_system::Reply::Halt);}
                else{assert_eq!(frame(reply),t.part(Kind::PartAck,offset,&[]).unwrap());}
            }
            assert_eq!(handle(&mut s,&mut stage,&t.frame(Kind::Commit).unwrap()),update_system::Reply::Halt);
            assert_eq!(media.0.borrow().ops.len(),count);
        }
    }
    #[test]fn update_session_each_media_error_is_terminal_and_never_success_ack(){
        use update_wire::{self as w,Mode,Kind,Transfer,RecordReceiver,PART};
        use update_system::Reply;
        fn run(mode:Mode,fault:Option<usize>)->usize{
            let(media,_,next)=seed();
            if mode==Mode::Fallback{
                let mut v=mount(media.clone());let p=v.prepare(&package(2)).unwrap();v.publish(&p,next).unwrap();
            }
            let mut s=update_system::Server::new(mount(media.clone()),(1u64<<40)+1).unwrap();
            media.0.borrow_mut().ops.clear();media.0.borrow_mut().fail=fault;
            let mut stage=CopyStage::default();
            let start=w::request(Kind::Start,mode,1,0).unwrap();
            let mut reply=handle(&mut s,&mut stage,&start);
            if let Reply::Frame(offer)=reply{
                let t=Transfer::parse(&offer,Kind::Offer).unwrap();
                let mut rx=RecordReceiver::new(t,Kind::RecordPart).unwrap();
                for offset in (0..512).step_by(PART){
                    rx.push(&frame(handle(&mut s,&mut stage,&t.part(Kind::RecordGet,offset,&[]).unwrap()))).unwrap();
                }
                let current=rx.finish().unwrap();
                let v=update_manager::verify(t,current,&stage.bytes).unwrap();
                let next=v.next();let expected=v.expected_ack().unwrap();
                if let Some(next)=next{publish_parts(&mut s,&mut stage,t,next);}
                let commit=t.frame(Kind::Commit).unwrap();
                reply=handle(&mut s,&mut stage,&commit);
                if fault.is_none(){
                    assert_eq!(frame(reply),expected);
                    // Simulate lost ACK: retransmission cannot replay any I/O.
                    let count=media.0.borrow().ops.len();
                    assert_eq!(handle(&mut s,&mut stage,&commit),Reply::Ignore);
                    assert_eq!(media.0.borrow().ops.len(),count);
                    return count;
                }
            }
            assert!(fault.is_some());assert_eq!(reply,Reply::Halt);
            let count=media.0.borrow().ops.len();
            assert_eq!(handle(&mut s,&mut stage,&start),Reply::Halt);
            assert_eq!(media.0.borrow().ops.len(),count);count
        }
        for mode in [Mode::Boot,Mode::Install,Mode::Fallback]{
            let calls=run(mode,None);assert!(calls>5);
            for at in 1..=calls{run(mode,Some(at));}
        }
    }

    #[test] fn system_selected_boot_is_read_only_exact_and_not_publication_authority(){
        let (media,old,next)=seed();let mut volume=mount(media.clone());
        let p=volume.prepare_boot().unwrap();let id=p.identity();
        assert_eq!((id.slot,id.generation,id.digest),(old.active().slot(),old.active().generation(),old.active().digest()));
        assert_eq!(id.package_hash,sha256::sha256(&package(0)).unwrap());
        let count=media.0.borrow().ops.len();
        assert!(matches!(volume.prepare_boot(),Err(Reject::Policy)));
        assert!(matches!(volume.prepare_fallback(),Err(Reject::Policy)));
        assert!(matches!(volume.prepare(&package(2)),Err(Reject::Policy)));
        assert_eq!(media.0.borrow().ops.len(),count);
        let mut copied=Vec::new();
        volume.copy_prepared(&p,|total,offset,part|{
            assert_eq!(total,package(0).len());assert_eq!(offset,copied.len());
            copied.extend_from_slice(part);Ok(())
        }).unwrap();assert_eq!(copied,package(0));
        assert_eq!(volume.publish(&p,old),Err(Reject::Policy));
        // Policy rejection preserves the same token/session for completion.
        assert!(!volume.is_readonly());
        volume.complete_boot(&p).unwrap();
        assert_eq!(volume.complete_boot(&p),Err(Reject::Policy));
        assert_eq!(volume.record(),old);assert!(!volume.is_readonly());
        assert!(!media.0.borrow().ops.iter().any(|op|matches!(op,Op::Write(_)|Op::Flush)));
        let p=volume.prepare(&package(2)).unwrap();volume.publish(&p,next).unwrap();
    }
    #[test] fn system_wrong_completion_method_or_record_preserves_cancellable_token(){
        let (media,old,next)=seed();let mut volume=mount(media.clone());
        let p=volume.prepare(&package(2)).unwrap();let count=media.0.borrow().ops.len();
        assert_eq!(volume.complete_boot(&p),Err(Reject::Policy));
        assert_eq!(volume.publish(&p,old),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);assert!(!volume.is_readonly());
        volume.cancel(&p).unwrap();
        let next_token=volume.prepare(&package(2)).unwrap();
        assert_eq!(volume.publish(&p,next),Err(Reject::Policy));
        volume.publish(&next_token,next).unwrap();
        let count=media.0.borrow().ops.len();
        assert_eq!(volume.publish(&next_token,next),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);
    }
    #[test] fn system_cancel_is_explicit_and_stale_tokens_never_unlock_current(){
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        let old=volume.prepare_boot().unwrap();let count=media.0.borrow().ops.len();
        volume.cancel(&old).unwrap();assert_eq!(media.0.borrow().ops.len(),count);
        let new=volume.prepare_boot().unwrap();assert!(new.identity().transaction>old.identity().transaction);
        assert_eq!(volume.cancel(&old),Err(Reject::Policy));
        assert_eq!(volume.copy_prepared(&old,|_,_,_|panic!("stale boot")),Err(Reject::Policy));
        volume.complete_boot(&new).unwrap();
    }
    #[test] fn system_boot_content_failure_allows_exact_prior_but_io_is_terminal(){
        let (media,mut volume,installed)=installed_volume();
        media.0.borrow_mut().blocks.get_mut(&4099).unwrap()[0]^=1;
        assert!(matches!(volume.prepare_boot(),Err(Reject::Framing)));
        assert!(!volume.is_readonly());
        let prior=volume.prepare_fallback().unwrap();
        assert_eq!(prior.identity().digest,installed.previous().unwrap().digest());
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        let count=media.0.borrow().ops.len();media.0.borrow_mut().fail=Some(count+1);
        assert!(matches!(volume.prepare_boot(),Err(Reject::Io)));
        assert!(volume.is_readonly());
        assert!(matches!(volume.prepare_fallback(),Err(Reject::ReadOnly)));
    }
    #[test] fn system_boot_copy_and_completion_detect_intervening_selector_change(){
        for at_completion in [false,true]{
            let (media,_,_)=seed();let mut volume=mount(media.clone());
            let p=volume.prepare_boot().unwrap();
            media.0.borrow_mut().blocks.insert(0,[0;512]);
            if at_completion{assert!(volume.complete_boot(&p).is_err());}
            else{assert!(volume.copy_prepared(&p,|_,_,_|panic!("changed selector")).is_err());}
            assert!(volume.is_readonly());
        }
    }
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
        assert_eq!(volume.copy_prepared(&prepared,|length,offset,chunk|{
            assert_eq!(length,data.len());assert_eq!(offset,copied.len());
            assert!(chunk.len()<=512);copied.extend_from_slice(chunk);Ok(())
        }).unwrap(),());
        assert_eq!(copied,data);
        assert_eq!(volume.record(),old);
        volume.publish(&prepared,next).unwrap();
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
        volume.publish(&prepared,next).unwrap();let operations=media.0.borrow().ops.clone();
        let write=operations.iter().position(|op|matches!(op,Op::Write(_))).unwrap()+1;
        for cut in 1..=operations.len(){
            let (media,old,next)=seed();let mut volume=mount(media.clone());
            let prepared=volume.prepare(&package(2)).unwrap();
            {let mut d=media.0.borrow_mut();d.ops.clear();d.fail=Some(cut);}
            let result=volume.publish(&prepared,next);
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
        assert!(matches!(volume.prepare(&package(2)),Err(Reject::Policy)));
        volume.cancel(&stale).unwrap();
        let current=volume.prepare(&package(2)).unwrap();
        let count=media.0.borrow().ops.len();
        assert_eq!(volume.publish(&stale,next),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);
        assert_eq!(volume.publish(&current,old),Err(Reject::Policy));
        assert_eq!(media.0.borrow().ops.len(),count);
        assert_eq!(volume.record(),old);
    }
    #[test] fn system_changed_package_false_write_ack_and_selector_changes_fail_closed(){
        let (media,_,next)=seed();let mut volume=mount(media.clone());
        let prepared=volume.prepare(&package(2)).unwrap();
        media.0.borrow_mut().blocks.get_mut(&4100).unwrap()[7]^=1;
        assert_eq!(volume.publish(&prepared,next),Err(Reject::Changed));assert!(volume.is_readonly());
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
        assert_eq!(volume.read_stream(Slot::A,|_,_,_|Ok(())),Err(Reject::Io));
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
        assert_eq!(volume.read_stream(Slot::B,|_,_,_|Ok(())),Err(Reject::Framing));
        assert!(!volume.is_readonly());
    }
    #[test] fn system_prepared_copy_checks_selector_races_and_sink_prefix_failure(){
        for position in [0usize,1,3]{
            let (media,_,_)=seed();let mut volume=mount(media.clone());
            let prepared=volume.prepare(&package(2)).unwrap();let mut calls=0;
            if position==0{media.0.borrow_mut().blocks.insert(0,[0;512]);}
            let result=volume.copy_prepared(&prepared,|_,_,_|{
                calls+=1;
                if calls==position{media.0.borrow_mut().blocks.insert(0,[0;512]);}
                Ok(())
            });
            assert!(result.is_err());assert!(volume.is_readonly());
            if position==0{assert_eq!(calls,0);}
        }
        let (media,_,next)=seed();let mut volume=mount(media.clone());
        let stale=volume.prepare(&package(2)).unwrap();
        assert!(matches!(volume.prepare(&package(2)),Err(Reject::Policy)));
        volume.cancel(&stale).unwrap();
        let current=volume.prepare(&package(2)).unwrap();let mut called=false;
        assert_eq!(volume.copy_prepared(&stale,|_,_,_|{called=true;Ok(())}),Err(Reject::Policy));
        assert!(!called);assert!(!volume.is_readonly());
        let mut prefix=0;
        assert_eq!(volume.copy_prepared(&current,|_,_,bytes|{
            prefix+=bytes.len();Err(())
        }),Err(Reject::Sink));
        assert_eq!(prefix,512);assert!(volume.is_readonly());
        assert_eq!(volume.publish(&current,next),Err(Reject::ReadOnly));
        // The native caller must clear its partial reservation; no source test
        // claims that a kernel abort or target execution occurred here.
    }
    fn installed_volume()->(Media,Volume<Media>,Record){
        let (media,_,next)=seed();let mut volume=mount(media.clone());
        let p=volume.prepare(&package(2)).unwrap();volume.publish(&p,next).unwrap();
        (media,volume,next)
    }
    #[test] fn system_authorized_fallback_preserves_payloads_and_high_water(){
        let (media,mut volume,installed)=installed_volume();
        // Content failure is not a transport fault; the committed exact prior
        // remains available. Corrupt only the inactive-after-fallback B payload.
        media.0.borrow_mut().blocks.get_mut(&4099).unwrap()[0]^=1;
        assert_eq!(volume.read_stream(Slot::B,|_,_,_|Ok(())),Err(Reject::Framing));
        assert!(!volume.is_readonly());let original=media.0.borrow().blocks.clone();
        let p=volume.prepare_fallback().unwrap();
        assert_eq!(p.identity().slot,Slot::A);assert_eq!(p.identity().generation,1);
        let mut bytes=Vec::new();
        volume.copy_prepared(&p,|_,_,part|{bytes.extend_from_slice(part);Ok(())}).unwrap();
        let verified=manifest::verify(&bytes[..384],&bytes[384..],1).unwrap();
        assert_eq!(verified.manifest().digest(),installed.previous().unwrap().digest());
        let next=installed.fallback().unwrap();volume.publish(&p,next).unwrap();
        assert_eq!(volume.record(),next);assert_eq!(next.highest_committed_generation(),2);
        assert_eq!(mount(media.clone()).record(),next);
        for (sector,bytes) in original{if sector>=2{assert_eq!(media.0.borrow().blocks.get(&sector),Some(&bytes));}}
        assert!(matches!(volume.prepare_fallback(),Err(Reject::Policy)));
        assert!(matches!(volume.prepare(&package(2)),Err(Reject::Policy)));
    }
    #[test] fn system_fallback_prepare_and_publication_failures_never_rewrite_packages(){
        let (media,mut volume,installed)=installed_volume();
        media.0.borrow_mut().ops.clear();let p=volume.prepare_fallback().unwrap();
        let preparation=media.0.borrow().ops.len();
        media.0.borrow_mut().ops.clear();volume.publish(&p,installed.fallback().unwrap()).unwrap();
        let publication=media.0.borrow().ops.len();
        for phase in 0..2{
            for cut in 1..=if phase==0{preparation}else{publication}{
                let (media,mut volume,installed)=installed_volume();
                let p=if phase==1{Some(volume.prepare_fallback().unwrap())}else{None};
                {let mut d=media.0.borrow_mut();d.ops.clear();d.fail=Some(cut);}
                let failed=if let Some(p)=p{volume.publish(&p,installed.fallback().unwrap()).is_err()}
                    else{volume.prepare_fallback().is_err()};
                assert!(failed);assert!(volume.is_readonly());
                assert!(!media.0.borrow().ops.iter().any(|op|matches!(op,Op::Write(s) if *s>=2)));
            }
        }
    }
    #[test] fn system_maximum_package_io_stays_inside_its_slot(){
        let (media,_,_)=seed();let mut volume=mount(media.clone());
        let mut data=vec![0x5a;MAX_PACKAGE];data[..384].copy_from_slice(case(2).0);
        data[56..60].copy_from_slice(&2_097_152u32.to_le_bytes());
        media.0.borrow_mut().blocks.insert(system_volume::FIRST_RESERVED,[0xa5;512]);
        let p=volume.prepare(&data).unwrap();assert_eq!(p.identity().length,MAX_PACKAGE);
        let mut copied=0;
        volume.copy_prepared(&p,|total,offset,chunk|{
            assert_eq!(total,MAX_PACKAGE);assert_eq!(offset,copied);
            assert_eq!(chunk,&data[offset..offset+chunk.len()]);copied+=chunk.len();Ok(())
        }).unwrap();
        assert_eq!(copied,MAX_PACKAGE);
        assert_eq!(media.0.borrow().blocks.get(&system_volume::FIRST_RESERVED),Some(&[0xa5;512]));
        assert!(media.0.borrow().ops.iter().all(|op|match op{
            Op::Read(s)|Op::Write(s)=>*s<system_volume::FIRST_RESERVED,Op::Flush=>true
        }));
    }

}

mod repair_decisions {
    use super::*;
    use repair::{Factory,Plan,Role,Reject,inspect};
    use journal::{Record,Slot};
    fn package(index:usize)->Vec<u8>{let(r,p)=case(index);[r,p].concat()}
    fn record()->Record{let(r,p)=case(0);Record::factory(&manifest::verify(r,p,1).unwrap())}
    fn root<'a>(p:&'a[u8])->Factory<'a>{Factory::verify(p,sha256::sha256(p).unwrap()).unwrap()}
    #[test]fn repair_root_is_exact_authenticated_immutable_generation_one(){
        let p=package(0);let hash=sha256::sha256(&p).unwrap();
        let f=Factory::verify(&p,hash).unwrap();
        assert_eq!(f.package_hash(),hash);assert_eq!(f.manifest_digest(),record().active().digest());
        assert!(Factory::verify(&p,[0;32]).is_err());
        let mut wrong=hash;wrong[0]^=1;assert!(Factory::verify(&p,wrong).is_err());
        let mut damaged=p.clone();damaged[512]^=1;
        assert!(Factory::verify(&damaged,hash).is_err());
        // Recomputing the expected hash cannot bypass signature/payload checks.
        assert!(Factory::verify(&damaged,sha256::sha256(&damaged).unwrap()).is_err());
        for index in [1,2,3]{
            let p=package(index);assert!(Factory::verify(&p,sha256::sha256(&p).unwrap()).is_err());
        }
        for length in [0usize,383,384,895,p.len()-1]{
            let bytes=&p[..length];assert!(Factory::verify(bytes,sha256::sha256(bytes).unwrap()).is_err());
        }
    }
    #[test]fn repair_requires_damaged_active_and_no_verified_prior(){
        let p=package(0);let f=root(&p);let old=record();
        let good=inspect(old,Role::Active,&p).unwrap();
        assert!(matches!(Plan::new(old,&f,good,None),Err(Reject::ActiveUsable)));
        assert_eq!(inspect(old,Role::Prior,&p),Err(Reject::MissingPrior));
        let mut bad=p.clone();bad[512]^=1;
        let damage=inspect(old,Role::Active,&bad).unwrap();
        let plan=Plan::new(old,&f,damage,None).unwrap();let next=plan.next();
        assert_eq!(next.sequence(),2);assert_eq!(next.active().slot(),Slot::B);
        assert_eq!(next.active().generation(),1);assert_eq!(next.active().digest(),old.active().digest());
        assert_eq!(next.highest_committed_generation(),1);assert!(next.previous().is_none());
        assert_eq!(Record::decode(&next.encode()),Ok(next));
        assert_eq!(journal::select([&old.encode(),&next.encode()]).unwrap().record(),next);
        assert_eq!(plan.recheck(old,&f,damage,None),Ok(()));
        assert_eq!(plan.recheck(old,&f,good,None),Err(Reject::Changed));
        let updated=old.install(&manifest::verify(case(2).0,case(2).1,2).unwrap()).unwrap();
        let active=inspect(updated,Role::Active,&bad).unwrap();
        let prior=inspect(updated,Role::Prior,&p).unwrap();
        assert!(matches!(Plan::new(updated,&f,active,None),Err(Reject::MissingPrior)));
        assert!(matches!(Plan::new(updated,&f,active,Some(prior)),Err(Reject::PriorUsable)));
        let prior_bad=inspect(updated,Role::Prior,&bad).unwrap();
        let plan=Plan::new(updated,&f,active,Some(prior_bad)).unwrap();let repaired=plan.next();
        assert_eq!(repaired.active().slot(),Slot::A);
        assert_eq!(repaired.highest_committed_generation(),2);
        assert_eq!(repaired.minimum_install_generation(),Ok(3));
        assert!(repaired.install(&manifest::verify(case(2).0,case(2).1,2).unwrap()).is_err());
        assert!(repaired.fallback().is_err());
        assert_eq!(journal::select([&repaired.encode(),&updated.encode()]).unwrap().record(),repaired);
    }
    #[test]fn repair_plan_refuses_mixed_records_roles_and_changed_observations(){
        let p=package(0);let f=root(&p);let old=record();
        let updated=old.install(&manifest::verify(case(2).0,case(2).1,2).unwrap()).unwrap();
        let mut bad=p.clone();bad[512]^=1;
        let active=inspect(updated,Role::Active,&bad).unwrap();
        let prior=inspect(updated,Role::Prior,&bad).unwrap();
        let stale=inspect(old,Role::Active,&bad).unwrap();
        assert!(matches!(Plan::new(updated,&f,stale,Some(prior)),Err(Reject::Identity)));
        assert!(matches!(Plan::new(updated,&f,prior,Some(active)),Err(Reject::Identity)));
        assert!(matches!(Plan::new(old,&f,stale,Some(prior)),Err(Reject::MissingPrior)));
        let plan=Plan::new(updated,&f,active,Some(prior)).unwrap();
        assert_eq!(plan.factory_hash(),sha256::sha256(&p).unwrap());
        bad[513]^=1;let changed=inspect(updated,Role::Active,&bad).unwrap();
        assert_eq!(plan.recheck(updated,&f,changed,Some(prior)),Err(Reject::Changed));
        assert_eq!(plan.recheck(updated,&f,active,None),Err(Reject::Changed));
        assert_eq!(plan.recheck(old,&f,active,Some(prior)),Err(Reject::Changed));
        // A signed package under the wrong journal identity is not usable.
        let mismatched=inspect(updated,Role::Active,&p).unwrap();
        assert!(Plan::new(updated,&f,mismatched,Some(prior)).is_ok());
        assert!(inspect(old,Role::Active,&vec![0;system_volume::MAX_PACKAGE+1]).is_err());
    }
}
