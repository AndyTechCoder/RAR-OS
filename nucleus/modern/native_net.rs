//! Candidate fixed NE2000 kernel PIO leaf. No native caller/profile selects it.
#![deny(unsafe_op_in_unsafe_fn)]
use super::model::{Error,Runtime,Endpoint};
use core::marker::PhantomData;
const BASE:u16=0x300;
trait Ports{
    fn r8(&mut self,p:u16)->Result<u8,Error>;
    fn w8(&mut self,p:u16,v:u8)->Result<(),Error>;
    fn r16(&mut self,p:u16)->Result<u16,Error>;
    fn w16(&mut self,p:u16,v:u16)->Result<(),Error>;
}
struct Native{_not_send:PhantomData<*mut ()>}
#[cfg(all(target_arch="x86_64",target_os="uefi"))]
impl Ports for Native{
    fn r8(&mut self,p:u16)->Result<u8,Error>{
        let v:u8;
        // SAFETY: private dispatch supplies only fixed certified NIC/PIC ports;
        // unsafe Adapter contract requires CPL0, IF=0 and exclusive ownership.
        unsafe{core::arch::asm!("in al, dx",in("dx")p,out("al")v,options(nostack,preserves_flags));}Ok(v)
    }
    fn w8(&mut self,p:u16,v:u8)->Result<(),Error>{
        // SAFETY: fixed NIC aperture, bounded typed operation and live capability.
        unsafe{core::arch::asm!("out dx, al",in("dx")p,in("al")v,options(nostack,preserves_flags));}Ok(())
    }
    fn r16(&mut self,p:u16)->Result<u16,Error>{
        let v:u16;
        // SAFETY: only fixed NIC data port0x310, never guest-memory bus-master DMA.
        unsafe{core::arch::asm!("in ax, dx",in("dx")p,out("ax")v,options(nostack,preserves_flags));}Ok(v)
    }
    fn w16(&mut self,p:u16,v:u16)->Result<(),Error>{
        // SAFETY: only fixed NIC data port and checked16-bit value.
        unsafe{core::arch::asm!("out dx, ax",in("dx")p,in("ax")v,options(nostack,preserves_flags));}Ok(())
    }
}
#[cfg(not(all(target_arch="x86_64",target_os="uefi")))]
impl Ports for Native{
    fn r8(&mut self,_:u16)->Result<u8,Error>{Err(Error::Denied)}
    fn w8(&mut self,_:u16,_:u8)->Result<(),Error>{Err(Error::Denied)}
    fn r16(&mut self,_:u16)->Result<u16,Error>{Err(Error::Denied)}
    fn w16(&mut self,_:u16,_:u16)->Result<(),Error>{Err(Error::Denied)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
enum Op{Read(u8),Write(u8,u8),ReadWord,WriteWord(u16),Reset}
fn decode(operation:u64,value:u64,extra:u64)->Option<Op>{
    match operation{
        0 if value<16&&extra==0=>Some(Op::Read(value as u8)),
        1 if value<16&&extra<=255=>{
            // No page2/3, interrupt enable or arbitrary CR command mode. Page1 multicast byte15 is also constrained to zero.
            if value==0&&!matches!(extra,0x21|0x22|0x61|0x62|0x0a|0x12|0x26)||
                value==15&&extra!=0{return None;}
            Some(Op::Write(value as u8,extra as u8))
        },
        2 if value==0&&extra==0=>Some(Op::ReadWord),
        3 if value<=65535&&extra==0=>Some(Op::WriteWord(value as u16)),
        4 if value==0&&extra==0=>Some(Op::Reset),
        _=>None,
    }
}
struct State{owner:Option<(Endpoint,u64)>,closed:bool}
impl State{
    fn stop<P:Ports>(&mut self,io:&mut P){
        if !self.closed{self.closed=true;let _=io.w8(BASE,0x21);}
    }
    fn reconcile<P:Ports>(&mut self,io:&mut P,policy:&Runtime){
        if let Some((owner,handle))=self.owner{
            if policy.network(owner.slot as usize,handle)!=Ok(owner){self.stop(io);}
        }
    }
    fn execute<P:Ports>(&mut self,io:&mut P,policy:&Runtime,caller:usize,handle:u64,
        operation:u64,value:u64,extra:u64)->Result<u64,Error>{
        // Complete authority/framing checks precede all I/O, including STOP.
        let endpoint=policy.network(caller,handle)?;
        let op=decode(operation,value,extra).ok_or(Error::Invalid)?;
        if self.closed{return Err(Error::Denied);}
        if self.owner.is_some_and(|x|x!=(endpoint,handle)){self.stop(io);return Err(Error::Stale);}
        self.owner=Some((endpoint,handle));
        let result=match op{
            Op::Read(r)=>io.r8(BASE+u16::from(r)).map(u64::from),
            Op::Write(r,v)=>io.w8(BASE+u16::from(r),v).map(|_|0),
            Op::ReadWord=>io.r16(BASE+0x10).map(u64::from),
            Op::WriteWord(v)=>io.w16(BASE+0x10,v).map(|_|0),
            Op::Reset=>io.r8(BASE+0x1f).map(|_|0),
        };
        if result.is_err(){self.stop(io);}result
    }
}
/// Exclusive, non-Clone, !Send/!Sync kernel-owned device leaf.
pub struct Adapter{io:Native,state:State}
impl Adapter{
    /// # Safety
    /// Only in the separately certified NE2000 ISA cloud profile, after firmware
    /// exit, CPL0/IF=0/soleCPU. Exact0x300..0x31f aperture with no overlaps,
    /// IRQ5 PIC-masked, no passthrough, no other driver or open network backend.
    /// The controller independently verifies model/MAC/netdev/socket lifecycle.
    /// Failure authorizes no fallback ports, reset/retry or host use.
    pub unsafe fn initialize()->Result<Self,Error>{
        let mut io=Native{_not_send:PhantomData};
        if io.r8(0x21)?&0x20==0{return Err(Error::Denied);}
        Ok(Self{io,state:State{owner:None,closed:false}})
    }
    /// # Safety
    /// Same certified profile and exclusive CPL0/IF=0 ownership. Caller is the
    /// actual trapped physical slot; policy is current kernel state. No pointer.
    /// Native trap integration must reconcile immediately after revocations.
    pub unsafe fn execute(&mut self,p:&Runtime,caller:usize,h:u64,op:u64,v:u64,x:u64)->Result<u64,Error>{
        self.state.execute(&mut self.io,p,caller,h,op,v,x)
    }
    /// # Safety
    /// Same native ownership. Invoke after every relevant kernel revocation and
    /// before scheduling another user; logical revocation alone cannot stop NIC.
    pub unsafe fn reconcile(&mut self,p:&Runtime){self.state.reconcile(&mut self.io,p);}
}
#[cfg(test)]
mod tests{
    use super::*;
    use super::super::model;
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    enum Access{R8(u16),W8(u16,u8),R16(u16),W16(u16,u16)}
    #[derive(Default)]struct Fake{log:Vec<Access>,fail:bool}
    impl Fake{fn hit(&mut self,a:Access)->Result<(),Error>{
        self.log.push(a);if self.fail{self.fail=false;Err(Error::Invalid)}else{Ok(())}
    }}
    impl Ports for Fake{
        fn r8(&mut self,p:u16)->Result<u8,Error>{self.hit(Access::R8(p))?;Ok(0x5a)}
        fn w8(&mut self,p:u16,v:u8)->Result<(),Error>{self.hit(Access::W8(p,v))}
        fn r16(&mut self,p:u16)->Result<u16,Error>{self.hit(Access::R16(p))?;Ok(0xabcd)}
        fn w16(&mut self,p:u16,v:u16)->Result<(),Error>{self.hit(Access::W16(p,v))}
    }
    fn policy()->Runtime{
        let mut p=Runtime::expansion_bootstrap();let h=p.handle(8,model::MANAGER_CAP).unwrap();
        p.authenticated_stage(8,h,1,5,[1;32],1,10).unwrap();
        let t=p.begin_trial(8,h,1).unwrap();
        p.ready(5,p.handle(5,model::HEALTH_CAP).unwrap(),t.token()).unwrap();
        let plan=p.prepare_desktop(8,h,t.token()).unwrap();p.publish_desktop(8,h,plan).unwrap();p
    }
    fn state()->State{State{owner:None,closed:false}}
    #[test]fn typed_ops_never_name_arbitrary_ports_and_failures_stop_once(){
        let p=policy();let h=p.handle(10,model::DEVICE_CAP).unwrap();
        for (op,v,x,access,want)in [
            (0,15,0,Access::R8(0x30f),0x5a),(1,8,255,Access::W8(0x308,255),0),
            (1,15,0,Access::W8(0x30f,0),0),(2,0,0,Access::R16(0x310),0xabcd),
            (3,65535,0,Access::W16(0x310,65535),0),(4,0,0,Access::R8(0x31f),0),
        ]{
            let mut s=state();let mut io=Fake::default();
            assert_eq!(s.execute(&mut io,&p,10,h,op,v,x),Ok(want));assert_eq!(io.log,[access]);
            let mut s=state();let mut io=Fake{fail:true,..Fake::default()};
            assert_eq!(s.execute(&mut io,&p,10,h,op,v,x),Err(Error::Invalid));
            assert_eq!(io.log,[access,Access::W8(BASE,0x21)]);
            assert_eq!(s.execute(&mut io,&p,10,h,op,v,x),Err(Error::Denied));assert_eq!(io.log.len(),2);
        }
    }
    #[test]fn authority_and_full_width_shape_refusals_touch_no_ports(){
        let p=policy();let h=p.handle(10,model::DEVICE_CAP).unwrap();
        let mut s=state();let mut io=Fake::default();
        for caller in 0..=model::TASKS{if caller!=10{
            assert!(s.execute(&mut io,&p,caller,h,0,0,0).is_err());
        }}
        for cap in [0,h^1,h^(1<<32),p.handle(10,0).unwrap()]{
            assert!(s.execute(&mut io,&p,10,cap,0,0,0).is_err());
        }
        for (op,v,x)in [(0,16,0),(0,0,1),(1,16,0),(1,1,256),(1,15,1),
            (1,0,0xe2),(2,1,0),(2,0,1),(3,65536,0),(3,0,1),(4,1,0),
            (4,0,1),(5,0,0),(u64::MAX,0,0),(0,u64::MAX,0)]{
            assert!(s.execute(&mut io,&p,10,h,op,v,x).is_err());
        }
        assert!(io.log.is_empty());assert!(s.owner.is_none());assert!(!s.closed);
        for v in 0..=255{assert_eq!(decode(1,0,v).is_some(),matches!(v,0x21|0x22|0x61|0x62|0x0a|0x12|0x26));}
    }
    #[test]fn owner_death_reconciliation_stops_without_new_authority(){
        let mut p=policy();let h=p.handle(10,model::DEVICE_CAP).unwrap();
        let mut s=state();let mut io=Fake::default();
        s.execute(&mut io,&p,10,h,0,0,0).unwrap();
        p.fault(p.binding(7).unwrap().unwrap()).unwrap();
        s.reconcile(&mut io,&p);s.reconcile(&mut io,&p);
        assert_eq!(io.log,[Access::R8(BASE),Access::W8(BASE,0x21)]);assert!(s.closed);
        assert!(s.execute(&mut io,&p,10,h,0,0,0).is_err());assert_eq!(io.log.len(),2);
    }
    #[cfg(not(all(target_arch="x86_64",target_os="uefi")))]
    #[test]fn native_backend_is_inert_off_uefi(){
        let mut io=Native{_not_send:PhantomData};
        assert_eq!(io.r8(BASE),Err(Error::Denied));assert_eq!(io.w8(BASE,0),Err(Error::Denied));
        assert_eq!(io.r16(BASE+16),Err(Error::Denied));assert_eq!(io.w16(BASE+16,0),Err(Error::Denied));
    }
}
