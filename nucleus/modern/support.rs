//! Shared Modern kernel bootstrap and checked CPU/memory bookkeeping.
//! Pure checks used by the actual kernel entry; not VM certification.
#![forbid(unsafe_code)]
use super::{abi,model::{self,Error}};
pub const TASKS:usize=16;
pub const INITIAL:[usize;10]=[0,1,2,3,4,5,6,8,9,15];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum CpuState {Runnable,Blocked,Dead}
#[derive(Clone,Copy,Debug)]
pub struct UserRange {pub start:u64,pub end:u64,pub writable:bool,pub executable:bool}
pub const EMPTY_RANGE:UserRange=UserRange{start:0,end:0,writable:false,executable:false};
pub fn user_buffer(ranges:&[UserRange],pointer:u64,length:usize,write:bool)->Result<(),Error>{
    bounded_buffer(ranges,pointer,length,write,abi::ENVELOPE_BYTES as usize)
}
/// Separate fixed limit; the existing IPC envelope boundary remains152 bytes.
pub fn staging_buffer(ranges:&[UserRange],pointer:u64,length:usize)->Result<(),Error>{
    bounded_buffer(ranges,pointer,length,false,512)
}
fn bounded_buffer(ranges:&[UserRange],pointer:u64,length:usize,write:bool,limit:usize)->Result<(),Error>{
    if length==0||length>limit||pointer<4096{return Err(Error::Invalid);}
    let end=pointer.checked_add(length as u64).ok_or(Error::Invalid)?;
    if end>0x0000_8000_0000_0000{return Err(Error::Denied);}
    if ranges.iter().any(|r|r.start<r.end&&r.start<=pointer&&end<=r.end&&
        !(r.writable&&r.executable)&&(!write||r.writable)){Ok(())}else{Err(Error::Denied)}
}
/// Saved user return bounds shared with the real trap adapter. Only the native
/// owner chooses stack_end; trials use four pages, initial services sixteen.
pub fn user_return(ranges:&[UserRange],stack_end:u64,rsp:u64,rip:u64,cs:u64,ss:u64,segments:[u64;4])->bool{
    matches!(stack_end,0x604000|0x610000)&&cs==0x1b&&ss==0x23&&
        (0x600000..=stack_end).contains(&rsp)&&
        ranges.iter().any(|r|r.executable&&!r.writable&&r.start<=rip&&rip<r.end)&&
        segments.iter().all(|s|[0,0x1b,0x23].contains(s))
}
/// Validate the complete destination before mutating the caller's queue.
pub fn receive(policy:&mut model::Runtime,caller:usize,handle:u64,ranges:&[UserRange],
    pointer:u64,length:u64,mode:u64)->Result<model::Message,Error>{
    if length!=abi::ENVELOPE_BYTES||mode>1{return Err(Error::Invalid);}
    user_buffer(ranges,pointer,abi::ENVELOPE_BYTES as usize,true)?;
    policy.receive(caller,handle)
}
pub fn next(states:&[CpuState;TASKS],current:usize)->Result<usize,Error>{
    if current>=TASKS{return Err(Error::Invalid);}
    (1..=TASKS).map(|n|(current+n)%TASKS)
        .find(|&i|states[i]==CpuState::Runnable).ok_or(Error::Empty)
}
/// Counts delivered IRQ0 events, not wall-clock time. Exhaustion is sticky.
pub fn tick(value:Option<u64>)->Option<u64>{
    value.and_then(|v|v.checked_add(1)).filter(|v|*v<=i64::MAX as u64)
}
pub fn framebuffer_span(width:u32,height:u32,pitch:u32,format:u32,base:u64,bytes:u64)->Result<u64,Error>{
    if width!=640||height!=480||!(640..=4096).contains(&pitch)||format>1||
        base==0||base%4096!=0{return Err(Error::Invalid);}
    let span=(pitch as u64).checked_mul(height as u64).and_then(|v|v.checked_mul(4)).ok_or(Error::Invalid)?;
    let rounded=span.div_ceil(4096)*4096;
    if rounded>8*1024*1024||rounded>bytes||base.checked_add(rounded).is_none_or(|e|e>0x1_0000_0000){
        return Err(Error::Denied);
    }
    Ok(rounded)
}
#[derive(Clone,Copy)]
pub struct DeviceProfile {pub sectors:u64,pub serial:[u8;20],pub model:[u8;40]}
const fn padded<const N:usize>(text:&[u8])->[u8;N]{
    assert!(!text.is_empty()&&text.len()<=N);
    let mut out=[b' ';N];let mut i=0;
    while i<text.len(){out[i]=text[i];i+=1;}out
}
// Candidate fixed synthetic geometry only. A trusted cloud controller must
// independently certify the exact attachments before this entry can execute.
// Identity strings are not keys. Data image keys must still be unique per image.
pub const DATA:DeviceProfile=DeviceProfile{sectors:194,
    serial:padded(b"RAR-M4-DATA-00000001"),model:padded(b"RAR M4 DATA PIO")};
pub const SYSTEM:DeviceProfile=DeviceProfile{sectors:16384,
    serial:padded(b"RAR-M4-SYS-000000001"),model:padded(b"RAR M4 SYSTEM PIO")};
pub fn bootstrap(policy:&model::Runtime,role:usize,entry:u64,pitch:u64,format:u64)->Result<abi::Boot,Error>{
    if !INITIAL.contains(&role){return Err(Error::Invalid);}
    let generation=if role==15 {1}else{
        let endpoint=policy.binding(role)?.ok_or(Error::Stale)?;
        if endpoint.slot as usize!=role||policy.state(role)?!=model::State::Active{return Err(Error::Denied);}
        endpoint.incarnation
    };
    let mut b=abi::Boot{magic:abi::MAGIC,version:abi::VERSION,bytes:abi::BOOT_BYTES,
        role:role as u64,phase:abi::ACTIVE,generation,entry,..abi::Boot::EMPTY};
    for i in 0..model::PRINCIPALS {b.peers[i]=policy.binding(i)?.map_or(0,|e|e.incarnation);}
    if role!=15 {
        for i in 0..model::CAP_SLOTS {b.caps[i]=policy.handle(role,i).unwrap_or(0);}
    }
    if role==3 {b.framebuffer=0x800000;b.width=640;b.height=480;b.pitch=pitch;b.format=format;}
    if matches!(role,1|9){
        let d=if role==1{DATA}else{SYSTEM};
        b.device_sectors=d.sectors;b.device_serial=d.serial;b.device_model=d.model;
    }
    if !abi::valid_boot(&b){return Err(Error::Invalid);}Ok(b)
}
/// Geometry only, NOT signature or generation-policy verification. The native
/// caller reads this from its sealed buffer after manager authentication.
pub struct StageMetadata{pub image_bytes:usize,pub generation:u64,pub digest:[u8;32],pub budget:u32}
pub fn stage_metadata(bytes:&[u8])->Result<StageMetadata,Error>{
    if !(896..=2_097_536).contains(&bytes.len()){return Err(Error::Invalid);}
    let word=|p:usize|u32::from_le_bytes(bytes[p..p+4].try_into().unwrap());
    let wide=|p:usize|u64::from_le_bytes(bytes[p..p+8].try_into().unwrap());
    let image_bytes=word(60) as usize;let generation=wide(72);
    let digest: [u8;32]=bytes[288..320].try_into().unwrap();let budget=word(232);
    if word(56) as usize!=bytes.len()-384||image_bytes==0||image_bytes>128*1024||
        image_bytes%4096!=0||wide(64)!=7||generation==0||digest==[0;32]||
        !(1..=100).contains(&budget)||word(236)!=0||word(240)!=16384{
        return Err(Error::Invalid);
    }
    Ok(StageMetadata{image_bytes,generation,digest,budget})
}
/// All initial desktop descriptors derive from the unpublished complete plan.
/// No native process may run until every descriptor/root is ready and published.
pub fn desktop_bootstrap(plan:&model::DesktopHandover,role:usize,entry:u64,pitch:u64,format:u64)
    ->Result<abi::Boot,Error>{
    if role>6{return Err(Error::Invalid);}
    let e=plan.binding(role).ok_or(Error::Stale)?;
    let mut b=abi::Boot{magic:abi::MAGIC,version:abi::VERSION,bytes:abi::BOOT_BYTES,
        role:role as u64,phase:abi::ACTIVE,generation:e.incarnation,entry,..abi::Boot::EMPTY};
    for i in 0..model::PRINCIPALS{b.peers[i]=plan.binding(i).map_or(0,|e|e.incarnation);}
    for i in 0..model::CAP_SLOTS{b.caps[i]=plan.handle(e.slot as usize,i).unwrap_or(0);}
    if role==3{b.framebuffer=0x800000;b.width=640;b.height=480;b.pitch=pitch;b.format=format;}
    if role==1{b.device_sectors=DATA.sectors;b.device_serial=DATA.serial;b.device_model=DATA.model;}
    if !abi::valid_boot(&b){return Err(Error::Invalid);}Ok(b)
}
/// Prepare the future read-only ACTIVE descriptor without publishing authority.
pub fn handover_bootstrap(policy:&model::Runtime,h:&model::Handover,entry:u64)->Result<abi::Boot,Error>{
    let t=policy.trial().ok_or(Error::Stale)?;
    if t.endpoint()!=h.endpoint()||t.token()!=h.token()||
        policy.state(h.endpoint().slot as usize)?!=model::State::Healthy{return Err(Error::Stale);}
    let mut b=abi::Boot{magic:abi::MAGIC,version:abi::VERSION,bytes:abi::BOOT_BYTES,
        role:5,phase:abi::ACTIVE,generation:h.endpoint().incarnation,entry,..abi::Boot::EMPTY};
    for i in 0..model::PRINCIPALS{b.peers[i]=policy.binding(i)?.map_or(0,|e|e.incarnation);}
    b.peers[5]=h.endpoint().incarnation;
    for i in 0..model::CAP_SLOTS{b.caps[i]=h.handle(i).unwrap_or(0);}
    if !abi::valid_boot(&b){return Err(Error::Invalid);}Ok(b)
}
/// A trial has only its one-shot health capability, never production grants.
pub fn trial_bootstrap(policy:&model::Runtime,trial:model::Trial,entry:u64)->Result<abi::Boot,Error>{
    let endpoint=trial.endpoint();
    if policy.trial()!=Some(trial)||policy.state(endpoint.slot as usize)?!=model::State::Trial{
        return Err(Error::Stale);
    }
    let mut b=abi::Boot{magic:abi::MAGIC,version:abi::VERSION,bytes:abi::BOOT_BYTES,
        role:5,phase:abi::TRIAL,generation:endpoint.incarnation,entry,
        health_token:trial.token(),..abi::Boot::EMPTY};
    b.caps[model::HEALTH_CAP]=policy.handle(endpoint.slot as usize,model::HEALTH_CAP)?;
    for i in 0..model::PRINCIPALS{b.peers[i]=policy.binding(i)?.map_or(0,|e|e.incarnation);}
    if !abi::valid_boot(&b){return Err(Error::Invalid);}Ok(b)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test] fn actual_user_return_check_respects_initial_and_trial_stack_bounds(){
        let rx=UserRange{start:0x401000,end:0x402000,writable:false,executable:true};
        for end in [0x604000,0x610000]{
            for rsp in [0x600000,end]{assert!(user_return(&[rx],end,rsp,rx.start,0x1b,0x23,[0;4]));}
            for rsp in [0x5fffff,end+1,u64::MAX]{assert!(!user_return(&[rx],end,rsp,rx.start,0x1b,0x23,[0;4]));}
        }
        assert!(!user_return(&[rx],0x604000,0x608000,rx.start,0x1b,0x23,[0;4]));
        assert!(user_return(&[rx],0x610000,0x608000,rx.start,0x1b,0x23,[0;4]));
        for (end,rip,cs,ss,segments) in [(0x604001,rx.start,0x1b,0x23,[0;4]),
            (0x604000,rx.end,0x1b,0x23,[0;4]),(0x604000,rx.start,8,0x23,[0;4]),
            (0x604000,rx.start,0x1b,16,[0;4]),(0x604000,rx.start,0x1b,0x23,[16;4])]{
            assert!(!user_return(&[rx],end,0x600000,rip,cs,ss,segments));
        }
        assert!(!user_return(&[UserRange{writable:true,..rx}],0x604000,0x600000,rx.start,0x1b,0x23,[0;4]));
    }
    #[test] fn sealed_metadata_rejects_unbounded_or_mismatched_resources(){
        let mut b=[0u8;896];b[56..60].copy_from_slice(&512u32.to_le_bytes());
        b[60..64].copy_from_slice(&8192u32.to_le_bytes());b[64]=7;b[72]=2;
        b[288..320].fill(1);b[232]=100;b[240..244].copy_from_slice(&16384u32.to_le_bytes());
        assert_eq!(stage_metadata(&b).unwrap().budget,100);
        for n in [0,383,895]{assert!(stage_metadata(&b[..n]).is_err());}
        for (p,v) in [(56,513u32),(60,0),(60,4097),(60,131073),(64,8),
            (232,0),(232,101),(236,1),(240,65536),(72,0)]{
            let mut bad=b;bad[p..p+4].copy_from_slice(&v.to_le_bytes());
            assert!(stage_metadata(&bad).is_err());
        }
        b[288..320].fill(0);assert!(stage_metadata(&b).is_err());
    }
    #[test] fn unpublished_desktop_boots_agree_without_any_live_desktop_binding(){
        for slot in [5usize,7]{
            let mut r=model::Runtime::bootstrap();let h=r.handle(8,model::MANAGER_CAP).unwrap();
            r.authenticated_stage(8,h,31,slot,[1;32],1,50).unwrap();let t=r.begin_trial(8,h,31).unwrap();
            r.ready(slot,r.handle(slot,model::HEALTH_CAP).unwrap(),t.token()).unwrap();
            let plan=r.prepare_desktop(8,h,t.token()).unwrap();
            for role in 0..7{
                let b=desktop_bootstrap(&plan,role,0x401000,640,0).unwrap();
                assert!(abi::valid_boot(&b));assert_eq!(b.peers[5],t.endpoint().incarnation);
                assert_eq!(b.peers[role],b.generation);assert_eq!(r.binding(role),Ok(None));
                assert_eq!(b.device_sectors,if role==1{DATA.sectors}else{0});
            }
            assert!(desktop_bootstrap(&plan,3,0x401000,639,0).is_err());
            assert!(desktop_bootstrap(&plan,8,0x401000,640,0).is_err());
        }
    }
    #[test] fn handover_refreshes_intervening_peer_fault_without_losing_peer_queue(){
        let mut r=model::Runtime::new();let h=r.handle(8,model::MANAGER_CAP).unwrap();
        r.authenticated_stage(8,h,19,7,[1;32],2,50).unwrap();
        let t=r.begin_trial(8,h,19).unwrap();
        r.ready(7,r.handle(7,model::HEALTH_CAP).unwrap(),t.token()).unwrap();
        let handover=r.prepare_cutover(8,h,t.token()).unwrap();
        let mut b=handover_bootstrap(&r,&handover,0x401000).unwrap();
        assert_eq!(b.peers[6],1);
        r.send(1,r.handle(1,4).unwrap(),b"peer queued").unwrap();
        r.fault(model::Endpoint{slot:6,incarnation:1}).unwrap();
        r.cutover_prepared(8,h,handover).unwrap();
        b.peers=r.binding_generations();assert!(abi::valid_boot(&b));
        assert_eq!(b.peers[6],0);assert_eq!(b.peers[5],t.endpoint().incarnation);
        assert_eq!(b.peers[1],1);
        assert_eq!(r.state(6),Ok(model::State::Vacant));
        let m=r.receive(4,r.handle(4,0).unwrap()).unwrap();
        assert_eq!(&m.bytes[..m.length as usize],b"peer queued");
        assert_eq!(r.delivered_preemption(model::Endpoint{slot:6,incarnation:1}),Err(Error::Stale));
    }
    #[test] fn prepared_active_boot_is_exact_but_does_not_publish_authority(){
        let mut r=model::Runtime::new();let h=r.handle(8,model::MANAGER_CAP).unwrap();
        r.authenticated_stage(8,h,18,7,[1;32],2,50).unwrap();
        let t=r.begin_trial(8,h,18).unwrap();let trial=trial_bootstrap(&r,t,0x401000).unwrap();
        r.ready(t.endpoint().slot as usize,trial.caps[model::HEALTH_CAP],t.token()).unwrap();
        let handover=r.prepare_cutover(8,h,t.token()).unwrap();
        let b=handover_bootstrap(&r,&handover,0x401000).unwrap();
        assert!(abi::valid_trial_activation(&trial,&b));assert_eq!(b.health_token,0);
        assert_eq!(r.state(7),Ok(model::State::Healthy));
        assert_eq!(r.binding(5).unwrap().unwrap().slot,5);
        for i in 0..model::CAP_SLOTS{assert_eq!(b.caps[i]!=0,i<3);}
        r.abort(8,h,t.token()).unwrap();
        assert!(handover_bootstrap(&r,&handover,0x401000).is_err());
    }
    #[test] fn sealed_trial_bootstrap_has_only_health_authority(){
        let mut r=model::Runtime::new();let h=r.handle(8,model::MANAGER_CAP).unwrap();
        r.authenticated_stage(8,h,17,7,[1;32],2,50).unwrap();
        let trial=r.begin_trial(8,h,17).unwrap();
        let b=trial_bootstrap(&r,trial,0x401000).unwrap();
        assert!(abi::valid_boot(&b));assert_eq!(b.phase,abi::TRIAL);
        assert_eq!(b.health_token,trial.token());assert_eq!(b.generation,trial.endpoint().incarnation);
        for i in 0..model::CAP_SLOTS{assert_eq!(b.caps[i]!=0,i==model::HEALTH_CAP);}
        assert_eq!((b.framebuffer,b.device_sectors),(0,0));
        assert!(trial_bootstrap(&r,trial,0x500000).is_err());
        r.ready(trial.endpoint().slot as usize,b.caps[model::HEALTH_CAP],trial.token()).unwrap();
        assert!(trial_bootstrap(&r,trial,0x401000).is_err());
    }
    #[test]fn real_initial_bootstrap_graph_and_isolated_idle(){
        let r=model::Runtime::new();
        for role in INITIAL{
            let b=bootstrap(&r,role,0x401000,640,0).unwrap();
            assert!(abi::valid_boot(&b));
            assert_eq!((b.kernel_probe,b.peer_probe),(0,0));
            assert_eq!(b.device_sectors,match role{1=>194,9=>16384,_=>0});
            assert_eq!(b.framebuffer,if role==3{0x800000}else{0});
            if role==15{assert_eq!(b.caps,[0;12]);assert_eq!(r.state(15),Ok(model::State::Vacant));}
        }
        for role in [7,10,14,16,usize::MAX]{assert!(bootstrap(&r,role,0x401000,640,0).is_err());}
        assert!(bootstrap(&r,3,0x401000,639,0).is_err());
        assert!(bootstrap(&r,3,0x401000,640,2).is_err());
        assert!(bootstrap(&r,0,0x500000,640,0).is_err());
        let mut r=r;r.fault(model::Endpoint{slot:1,incarnation:1}).unwrap();
        assert!(bootstrap(&r,1,0x401000,640,0).is_err());
        assert!(bootstrap(&r,9,0x401000,640,0).is_ok());
    }
    #[test]fn full_envelope_checked_without_desktop_truncation(){
        let rw=UserRange{start:0x600000,end:0x601000,writable:true,executable:false};
        let ro=UserRange{writable:false,..rw};
        assert!(user_buffer(&[rw],rw.end-152,152,true).is_ok());
        assert!(user_buffer(&[ro],ro.end-152,152,false).is_ok());
        assert!(user_buffer(&[ro],ro.end-152,152,true).is_err());
        for (p,n) in [(rw.end-151,152),(rw.start,153),(rw.start,0),(0,1),(u64::MAX,152)]{
            assert!(user_buffer(&[rw],p,n,true).is_err());
        }
        assert!(user_buffer(&[UserRange{executable:true,..rw}],rw.start,152,true).is_err());
        let high=UserRange{start:0x0000_7fff_ffff_f000,end:0x0000_8000_0000_1000,..rw};
        assert!(user_buffer(&[high],0x0000_8000_0000_0000,1,true).is_err());
        // Adjacent distinct ranges cannot authorize a cross-boundary copy.
        assert!(user_buffer(&[UserRange{end:rw.start+144,..rw},
            UserRange{start:rw.start+144,..rw}],rw.start,152,true).is_err());
    }
    #[test]fn staging_read_bound_does_not_expand_ipc_or_cross_ranges(){
        let r=UserRange{start:0x600000,end:0x601000,writable:false,executable:false};
        assert!(staging_buffer(&[r],r.end-512,512).is_ok());
        assert!(user_buffer(&[r],r.end-512,512,false).is_err());
        for (p,n) in [(r.end-511,512),(r.start,513),(r.start,0),(u64::MAX,512),(0,1)]{
            assert!(staging_buffer(&[r],p,n).is_err());
        }
        assert!(staging_buffer(&[UserRange{end:r.start+256,..r},
            UserRange{start:r.start+256,..r}],r.start,512).is_err());
        assert!(staging_buffer(&[UserRange{writable:true,executable:true,..r}],r.start,512).is_err());
    }
    #[test]fn bad_receive_cannot_consume_a_real_policy_message(){
        let mut p=model::Runtime::new();let tx=p.handle(1,4).unwrap();
        let rx=p.handle(4,0).unwrap();
        p.send(1,tx,b"committed").unwrap();
        let range=UserRange{start:0x600000,end:0x601000,writable:true,executable:false};
        for (ptr,len,mode) in [(range.end-151,152,0),(range.start,144,0),
            (range.start,152,2),(u64::MAX,152,0)]{
            assert!(receive(&mut p,4,rx,&[range],ptr,len,mode).is_err());
        }
        assert!(receive(&mut p,4,rx,&[UserRange{writable:false,..range}],
            range.start,152,0).is_err());
        assert!(receive(&mut p,4,rx^(1<<32),&[range],range.start,152,0).is_err());
        let m=receive(&mut p,4,rx,&[range],range.start,152,0).unwrap();
        assert_eq!(m.principal,1);assert_eq!(m.incarnation,1);
        assert_eq!(&m.bytes[..m.length as usize],b"committed");
        assert_eq!(receive(&mut p,4,rx,&[range],range.start,152,1),Err(Error::Empty));
    }
    #[test]fn round_robin_and_tick_exhaustion_are_bounded(){
        let mut s=[CpuState::Dead;TASKS];s[0]=CpuState::Blocked;s[15]=CpuState::Runnable;
        assert_eq!(next(&s,0),Ok(15));assert_eq!(next(&s,15),Ok(15));
        s[4]=CpuState::Runnable;assert_eq!(next(&s,15),Ok(4));
        assert_eq!(next(&s,4),Ok(15));assert_eq!(next(&s,16),Err(Error::Invalid));
        s[4]=CpuState::Dead;s[15]=CpuState::Dead;assert_eq!(next(&s,0),Err(Error::Empty));
        assert_eq!(tick(Some(0)),Some(1));
        assert_eq!(tick(Some(i64::MAX as u64-1)),Some(i64::MAX as u64));
        assert_eq!(tick(Some(i64::MAX as u64)),None);assert_eq!(tick(None),None);
    }
    #[test]fn fixed_geometry_and_framebuffer_bounds(){
        assert_eq!(DATA.sectors,2+3*64);assert_ne!(DATA.serial,SYSTEM.serial);
        assert_eq!(framebuffer_span(640,480,640,0,0x80000000,1228800),Ok(1228800));
        for (w,h,p,f,b,n) in [(800,480,800,0,0x80000000,2000000),
            (640,600,640,0,0x80000000,2000000),(640,480,639,0,0x80000000,2000000),
            (640,480,u32::MAX,0,0x80000000,u64::MAX),(640,480,640,2,0x80000000,2000000),
            (640,480,640,0,0,2000000),(640,480,640,0,0x80000001,2000000),
            (640,480,640,0,0xfffff000,2000000),(640,480,640,0,0x80000000,100)]{
            assert!(framebuffer_span(w,h,p,f,b,n).is_err());
        }
    }
}
