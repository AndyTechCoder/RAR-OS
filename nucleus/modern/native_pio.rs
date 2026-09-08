//! Fixed Modern PIO boundary; native instructions only on x86-64 UEFI.
//! Tests use fake I/O. This source does not activate a VM profile.
#![deny(unsafe_op_in_unsafe_fn)]
use crate::{abi::{self,DeviceOp},model::{Device,Error,Runtime}};
use core::marker::PhantomData;
trait PortIo {
    fn read8(&mut self,p:u16)->Result<u8,Error>;
    fn write8(&mut self,p:u16,v:u8)->Result<(),Error>;
    fn read16(&mut self,p:u16)->Result<u16,Error>;
    fn write16(&mut self,p:u16,v:u16)->Result<(),Error>;
}
struct Native {_not_send:PhantomData<*mut ()>}
#[cfg(all(target_arch="x86_64",target_os="uefi"))]
impl PortIo for Native {
    fn read8(&mut self,p:u16)->Result<u8,Error> {
        let v:u8;
        // SAFETY: private dispatch chooses fixed status/PIC registers; unsafe
        // Adapter entry requires certified VM, CPL0, IF=0, exclusive ownership.
        unsafe {core::arch::asm!("in al, dx",in("dx")p,out("al")v,options(nostack,preserves_flags));} Ok(v)
    }
    fn write8(&mut self,p:u16,v:u8)->Result<(),Error> {
        // SAFETY: fixed control/register/allowlisted command, no arbitrary port.
        unsafe {core::arch::asm!("out dx, al",in("dx")p,in("al")v,options(nostack,preserves_flags));} Ok(())
    }
    fn read16(&mut self,p:u16)->Result<u16,Error> {
        let v:u16;
        // SAFETY: fixed Data or System data register; no device selector.
        unsafe {core::arch::asm!("in ax, dx",in("dx")p,out("ax")v,options(nostack,preserves_flags));} Ok(v)
    }
    fn write16(&mut self,p:u16,v:u16)->Result<(),Error> {
        // SAFETY: fixed data register and already checked 16-bit value.
        unsafe {core::arch::asm!("out dx, ax",in("dx")p,in("ax")v,options(nostack,preserves_flags));} Ok(())
    }
}
#[cfg(not(all(target_arch="x86_64",target_os="uefi")))]
impl PortIo for Native {
    fn read8(&mut self,_:u16)->Result<u8,Error>{Err(Error::Denied)}
    fn write8(&mut self,_:u16,_:u8)->Result<(),Error>{Err(Error::Denied)}
    fn read16(&mut self,_:u16)->Result<u16,Error>{Err(Error::Denied)}
    fn write16(&mut self,_:u16,_:u16)->Result<(),Error>{Err(Error::Denied)}
}
fn ports(device:Device)->(u16,u16) {
    match device {Device::Data=>(0x1f0,0x3f6),Device::System=>(0x170,0x376)}
}
fn initialize_with<I:PortIo>(io:&mut I)->Result<(),Error> {
    // Trusted PIC setup must already mask IRQ14/15. Never change the PIC here.
    if io.read8(0xa1)?&0xc0!=0xc0 {return Err(Error::Denied);}
    for d in [Device::Data,Device::System] {io.write8(ports(d).1,2)?;}
    // nIEN=1, SRST=0. No disk command/write/reset is issued by initialization.
    for d in [Device::Data,Device::System] {
        if matches!(io.read8(ports(d).1)?,0|255) {return Err(Error::Invalid);}
    }
    Ok(())
}
fn execute_with<I:PortIo>(io:&mut I,policy:&Runtime,caller:usize,handle:u64,
    operation:u64,value:u64,extra:u64)->Result<u64,Error> {
    let device=policy.device(caller,handle)?;
    let op=abi::device_op(operation,value,extra).ok_or(Error::Invalid)?;
    let (base,control)=ports(device);
    match op {
        DeviceOp::Status=>Ok(io.read8(control)? as u64),
        DeviceOp::Count(v)=>{io.write8(base+2,v)?;Ok(0)},
        DeviceOp::LbaLow(v)=>{io.write8(base+3,v)?;Ok(0)},
        DeviceOp::LbaMid(v)=>{io.write8(base+4,v)?;Ok(0)},
        DeviceOp::LbaHigh(v)=>{io.write8(base+5,v)?;Ok(0)},
        DeviceOp::Head(v)=>{io.write8(base+6,v)?;Ok(0)},
        DeviceOp::Identify=>{io.write8(base+7,0xec)?;Ok(0)},
        DeviceOp::Read=>{io.write8(base+7,0x20)?;Ok(0)},
        DeviceOp::Write=>{io.write8(base+7,0x30)?;Ok(0)},
        DeviceOp::Flush=>{io.write8(base+7,0xe7)?;Ok(0)},
        DeviceOp::ReadWord=>Ok(io.read16(base)? as u64),
        DeviceOp::WriteWord(v)=>{io.write16(base,v)?;Ok(0)},
    }
}
/// Kernel-exclusive adapter, neither Send nor Sync. No public port/device field,
/// ordinary constructor, Clone, or reset/fallback operation.
pub struct Adapter {io:Native}
impl Adapter {
    /// # Safety
    /// Certified Modern x86-64 UEFI cloud VM only, after firmware exit and PIC
    /// setup, sole CPU, CPL0, IF=0. Controller must independently verify separate
    /// ISA IDE Data(1f0/3f6,IRQ14)/System(170/376,IRQ15) masters, no register
    /// collisions, bounded synthetic disks, no passthrough/DMA, exact identity
    /// and geometry. This function does NOT certify those facts.
    /// Keep exclusive ownership and initialize before scheduling storage users.
    /// Failure does not authorize retry, reset, alternative ports or host use.
    pub unsafe fn initialize()->Result<Self,Error> {
        let mut io=Native{_not_send:PhantomData};
        initialize_with(&mut io)?;Ok(Self{io})
    }
    /// # Safety
    /// Same VM/CPL0/IF=0/sole-CPU/exclusive-ownership contract for EVERY call.
    /// caller comes from saved kernel CPU context, never request bytes; policy
    /// is the live kernel-owned state, serialized against revocation/process
    /// changes. Userspace PIO separately enforces IDENTIFY, geometry, bounded
    /// sequencing/polling and sticky poisoning. This performs one I/O only.
    pub unsafe fn execute(&mut self,policy:&Runtime,caller:usize,handle:u64,
        operation:u64,value:u64,extra:u64)->Result<u64,Error> {
        execute_with(&mut self.io,policy,caller,handle,operation,value,extra)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Endpoint,DEVICE_CAP};
    #[derive(Clone,Copy,Debug,PartialEq,Eq)]
    enum Access {R8(u16),W8(u16,u8),R16(u16),W16(u16,u16)}
    struct Fake {log:Vec<Access>,fail:Option<usize>,mask:u8,status:u8}
    impl Fake {
        fn new()->Self {Self{log:Vec::new(),fail:None,mask:0xff,status:0x40}}
        fn hit(&mut self,a:Access)->Result<(),Error> {
            self.log.push(a);if self.fail==Some(self.log.len()) {Err(Error::Invalid)}else{Ok(())}
        }
    }
    impl PortIo for Fake {
        fn read8(&mut self,p:u16)->Result<u8,Error> {
            self.hit(Access::R8(p))?;Ok(if p==0xa1{self.mask}else{self.status})
        }
        fn write8(&mut self,p:u16,v:u8)->Result<(),Error>{self.hit(Access::W8(p,v))}
        fn read16(&mut self,p:u16)->Result<u16,Error>{self.hit(Access::R16(p))?;Ok(0xabcd)}
        fn write16(&mut self,p:u16,v:u16)->Result<(),Error>{self.hit(Access::W16(p,v))}
    }
    #[test] fn initialization_has_no_disk_commands_or_reset() {
        let mut io=Fake::new();initialize_with(&mut io).unwrap();
        assert_eq!(io.log,[Access::R8(0xa1),Access::W8(0x3f6,2),
            Access::W8(0x376,2),Access::R8(0x3f6),Access::R8(0x376)]);
        for mask in [0,0x3f,0x7f,0xbf] {
            let mut io=Fake::new();io.mask=mask;
            assert_eq!(initialize_with(&mut io),Err(Error::Denied));assert_eq!(io.log.len(),1);
        }
        for fail in 1..=5 {
            let mut io=Fake::new();io.fail=Some(fail);
            assert!(initialize_with(&mut io).is_err());assert_eq!(io.log.len(),fail);
        }
        for status in [0,255] {
            let mut io=Fake::new();io.status=status;
            assert_eq!(initialize_with(&mut io),Err(Error::Invalid));assert_eq!(io.log.len(),4);
        }
    }
    #[test] fn operations_use_only_derived_fixed_registers_with_no_retry() {
        let policy=Runtime::new();
        for (caller,base,control) in [(1,0x1f0,0x3f6),(9,0x170,0x376)] {
            let h=policy.handle(caller,DEVICE_CAP).unwrap();
            for (op,v,access,want) in [
                (0,0,Access::R8(control),0x40),(1,1,Access::W8(base+2,1),0),
                (2,255,Access::W8(base+3,255),0),(3,255,Access::W8(base+4,255),0),
                (4,255,Access::W8(base+5,255),0),(5,0xef,Access::W8(base+6,0xef),0),
                (6,0,Access::W8(base+7,0xec),0),(7,0,Access::W8(base+7,0x20),0),
                (8,0,Access::W8(base+7,0x30),0),(9,0,Access::W8(base+7,0xe7),0),
                (10,0,Access::R16(base),0xabcd),(11,65535,Access::W16(base,65535),0),
            ] {
                let mut io=Fake::new();
                assert_eq!(execute_with(&mut io,&policy,caller,h,op,v,0),Ok(want));
                assert_eq!(io.log,[access]);
                let mut io=Fake::new();io.fail=Some(1);
                assert_eq!(execute_with(&mut io,&policy,caller,h,op,v,0),Err(Error::Invalid));
                assert_eq!(io.log,[access]);
            }
        }
    }
    #[test] fn denied_stale_malformed_and_slave_requests_touch_no_ports() {
        let mut p=Runtime::new();let h=p.handle(1,DEVICE_CAP).unwrap();let mut io=Fake::new();
        for caller in 0..=16 {if ![1,9].contains(&caller) {
            assert!(execute_with(&mut io,&p,caller,h,0,0,0).is_err());
        }}
        for caller in [1,9] {
            for handle in [0,1,u64::MAX,h^(1<<32)] {
                assert!(execute_with(&mut io,&p,caller,handle,0,0,0).is_err());
            }
            for (op,v,x) in [(0,1,0),(1,2,0),(2,256,0),(3,256,0),(4,256,0),
                (5,0xb0,0),(5,0xf0,0),(6,1,0),(7,1,0),(8,1,0),(9,1,0),(10,1,0),
                (11,65536,0),(12,0,0),(u64::MAX,0,0),(0,0,1)] {
                assert!(execute_with(&mut io,&p,caller,h,op,v,x).is_err());
            }
        }
        p.fault(Endpoint{slot:1,incarnation:1}).unwrap();
        assert!(execute_with(&mut io,&p,1,h,0,0,0).is_err());
        p.fault(Endpoint{slot:9,incarnation:1}).unwrap();
        assert!(execute_with(&mut io,&p,9,h,0,0,0).is_err());assert!(io.log.is_empty());
    }
    #[cfg(not(all(target_arch="x86_64",target_os="uefi")))]
    #[test] fn non_uefi_native_backend_is_inert() {
        let mut io=Native{_not_send:PhantomData};
        assert_eq!(io.read8(0xa1),Err(Error::Denied));
        assert_eq!(io.write8(0x3f6,2),Err(Error::Denied));
        assert_eq!(io.read16(0x1f0),Err(Error::Denied));
        assert_eq!(io.write16(0x1f0,0),Err(Error::Denied));
    }
}
