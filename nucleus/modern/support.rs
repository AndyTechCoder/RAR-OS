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
    if length==0||length>abi::ENVELOPE_BYTES as usize||pointer<4096{return Err(Error::Invalid);}
    let end=pointer.checked_add(length as u64).ok_or(Error::Invalid)?;
    if end>0x0000_8000_0000_0000{return Err(Error::Denied);}
    if ranges.iter().any(|r|r.start<r.end&&r.start<=pointer&&end<=r.end&&
        !(r.writable&&r.executable)&&(!write||r.writable)){Ok(())}else{Err(Error::Denied)}
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
#[cfg(test)]
mod tests{
    use super::*;
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
