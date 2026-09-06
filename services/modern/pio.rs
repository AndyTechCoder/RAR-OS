//! Bounded userspace ATA PIO transport. No native I/O, disks, syscalls or DMA.
//! Io must be backed by one fixed, nondelegable kernel adapter capability.
#![forbid(unsafe_code)]
pub const SECTOR_BYTES:usize=512;
pub const POLLS:usize=65_536;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Register {Count,LbaLow,LbaMid,LbaHigh,Head}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Command {Identify,Read,Write,Flush}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Bounds,Unavailable,Device,Timeout,Transport,Poisoned,Identity}
/// This trait is not a capability boundary. The eventual kernel implementation
/// must independently enforce fixed adapter ownership, registers and commands.
pub trait Io {
    fn status(&mut self)->Result<u8,()>;
    fn register(&mut self,reg:Register,value:u8)->Result<(),()>;
    fn command(&mut self,command:Command)->Result<(),()>;
    fn read_word(&mut self)->Result<u16,()>;
    fn write_word(&mut self,word:u16)->Result<(),()>;
    fn yield_cpu(&mut self)->Result<(),()>;
}

/// Fixed trusted-profile expectations, never an IPC-provided device selector.
/// ATA text is exact space-padded ASCII, in display order rather than word order.
#[derive(Clone,Copy)]
pub struct Identity {pub sectors:u32,pub serial:[u8;20],pub model:[u8;40]}
impl Identity {
    fn valid(&self)->bool {
        self.sectors>0 && self.sectors<1<<28 &&
        [&self.serial[..],&self.model[..]].iter().all(|text|
            text.iter().all(|b|(0x20..=0x7e).contains(b)) && text.iter().any(|b|*b!=b' '))
    }
}
fn identify_matches(words:&[u16;256],expected:&Identity)->Result<(),Error> {
    if !expected.valid(){return Err(Error::Bounds);}
    // This is the pinned QEMU ATA disk profile, not arbitrary ATA/ATAPI support.
    // Advertised DMA capability does not grant DMA; the transport remains PIO.
    if words[0]!=0x0040 || words[49]&(1<<9)==0 ||
        words[83]&0xc000!=0x4000 || words[83]&(1<<12)==0 ||
        words[87]&0xc000!=0x4000 || words[86]&(1<<12)==0 {
        return Err(Error::Identity);
    }
    let capacity=words[60]as u32|((words[61]as u32)<<16);
    if capacity!=expected.sectors{return Err(Error::Identity);}
    let extended=words[100]as u64|((words[101]as u64)<<16)|
        ((words[102]as u64)<<32)|((words[103]as u64)<<48);
    if (words[83]&(1<<10)!=0 && extended!=expected.sectors as u64) ||
        (words[83]&(1<<10)==0 && extended!=0) {return Err(Error::Identity);}
    // Legacy default or valid 512-byte logical/physical sectors only.
    // No long logical sectors or nonzero physical-sector exponent are admitted.
    if !matches!(words[106],0|0x4000|0x6000) || words[117]!=0 || words[118]!=0 {
        return Err(Error::Identity);
    }
    for (start,text) in [(10,&expected.serial[..]),(27,&expected.model[..])] {
        for (i,byte) in text.iter().enumerate() {
            let word=words[start+i/2];
            let actual=if i%2==0 {(word>>8)as u8}else{word as u8};
            if actual!=*byte{return Err(Error::Identity);}
        }
    }
    Ok(())
}

pub struct Device<I:Io> {io:I,sectors:u32,poisoned:bool}
impl<I:Io> Device<I> {
    /// Only this verified constructor is available outside source tests.
    /// The kernel must already constrain Io to the fixed master-only PIO adapter,
    /// mask its IRQ and set nIEN. IDENTIFY does not itself confer device authority.
    pub fn identify(io:I,expected:Identity)->Result<Self,Error> {
        if !expected.valid(){return Err(Error::Bounds);}
        let mut device=Self {io,sectors:expected.sectors,poisoned:false};
        device.wait(false)?;
        device.io.register(Register::Head,0xa0).map_err(|_|Error::Transport)?;
        device.delay()?;device.wait(false)?;
        for register in [Register::Count,Register::LbaLow,Register::LbaMid,Register::LbaHigh] {
            device.io.register(register,0).map_err(|_|Error::Transport)?;
        }
        device.io.command(Command::Identify).map_err(|_|Error::Transport)?;
        device.delay()?;device.wait(true)?;
        let mut words=[0u16;256];
        for word in &mut words {*word=device.io.read_word().map_err(|_|Error::Transport)?;}
        device.delay()?;device.wait(false)?;
        identify_matches(&words,&expected)?;
        Ok(device)
    }
    #[cfg(test)]
    fn test_device(io:I,sectors:u32)->Result<Self,Error> {
        if sectors==0||sectors>=1<<28{return Err(Error::Bounds);}
        Ok(Self {io,sectors,poisoned:false})
    }
    pub fn poisoned(&self)->bool {self.poisoned}
    fn wait(&mut self,drq:bool)->Result<(),Error> {
        for poll in 0..POLLS {
            let s=self.io.status().map_err(|_|Error::Transport)?;
            if s==0||s==255{return Err(Error::Unavailable);}
            if s&0x80==0 {
                if s&0x21!=0{return Err(Error::Device);}
                if s&0x40!=0 && (s&8!=0)==drq{return Ok(());}
            }
            if poll%16==15 {self.io.yield_cpu().map_err(|_|Error::Transport)?;}
        }
        Err(Error::Timeout)
    }
    fn delay(&mut self)->Result<(),Error> {
        // Four alternate-status reads supply ATA device-select/command settling.
        for _ in 0..4 {
            let s=self.io.status().map_err(|_|Error::Transport)?;
            if s==0||s==255{return Err(Error::Unavailable);}
        }
        Ok(())
    }
    fn address(&mut self,lba:u32)->Result<(),Error> {
        self.wait(false)?;
        self.io.register(Register::Head,0xe0|((lba>>24)as u8&15)).map_err(|_|Error::Transport)?;
        self.delay()?;self.wait(false)?;
        for (r,v) in [(Register::Count,1),(Register::LbaLow,lba as u8),
            (Register::LbaMid,(lba>>8)as u8),(Register::LbaHigh,(lba>>16)as u8)] {
            self.io.register(r,v).map_err(|_|Error::Transport)?;
        }
        Ok(())
    }
    fn check(&self,lba:u32)->Result<(),Error> {
        if self.poisoned {Err(Error::Poisoned)}
        else if lba>=self.sectors {Err(Error::Bounds)}else{Ok(())}
    }
    fn finish<T>(&mut self,result:Result<T,Error>)->Result<T,Error> {
        if result.is_err(){self.poisoned=true;}result
    }
    pub fn read512(&mut self,lba:u32)->Result<[u8;SECTOR_BYTES],Error> {
        self.check(lba)?;
        let result=(||{
            self.address(lba)?;
            self.io.command(Command::Read).map_err(|_|Error::Transport)?;
            self.delay()?;self.wait(true)?;
            let mut out=[0u8;SECTOR_BYTES];
            for chunk in out.chunks_mut(2) {
                chunk.copy_from_slice(&self.io.read_word().map_err(|_|Error::Transport)?.to_le_bytes());
            }
            self.delay()?;self.wait(false)?;Ok(out)
        })();
        self.finish(result)
    }
    /// Success is command completion, NOT durability. flush() is mandatory.
    /// An error poisons this transport: never silently retry an ambiguous write.
    pub fn write512(&mut self,lba:u32,data:&[u8;SECTOR_BYTES])->Result<(),Error> {
        self.check(lba)?;
        let result=(||{
            self.address(lba)?;
            self.io.command(Command::Write).map_err(|_|Error::Transport)?;
            self.delay()?;self.wait(true)?;
            for chunk in data.chunks_exact(2) {
                self.io.write_word(u16::from_le_bytes([chunk[0],chunk[1]])).map_err(|_|Error::Transport)?;
            }
            self.delay()?;self.wait(false)
        })();
        self.finish(result)
    }
    pub fn flush(&mut self)->Result<(),Error> {
        if self.poisoned{return Err(Error::Poisoned);}
        let result=(||{
            self.wait(false)?;
            self.io.command(Command::Flush).map_err(|_|Error::Transport)?;
            self.delay()?;self.wait(false)
        })();
        self.finish(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[derive(Clone)]
    struct Fake {status:u8,command:Option<Command>,words:usize,reads:usize,yields:usize,
        registers:Vec<(Register,u8)>,written:Vec<u16>,fail_word:Option<usize>,fail_command:bool,
        status_override:Option<(usize,u8)>,fail_status_at:Option<usize>,
        fail_register_at:Option<usize>,fail_yield:bool,busy_until:usize,busy_per_command:usize,identify_words:Option<[u16;256]>}
    impl Fake {
        fn new()->Self {Self {status:0x40,command:None,words:0,reads:0,yields:0,
            registers:vec![],written:vec![],fail_word:None,fail_command:false,status_override:None,
            fail_status_at:None,fail_register_at:None,fail_yield:false,busy_until:0,busy_per_command:0,identify_words:None}}
    }
    impl Io for Fake {
        fn status(&mut self)->Result<u8,()> {
            self.reads+=1;
            if self.fail_status_at==Some(self.reads){return Err(());}
            if self.reads<=self.busy_until{return Ok(0x80);}
            if let Some((from,status))=self.status_override {if self.reads>=from{return Ok(status);}}
            Ok(self.status)
        }
        fn register(&mut self,r:Register,v:u8)->Result<(),()> {
            if self.fail_register_at==Some(self.registers.len()){return Err(());}
            self.registers.push((r,v));Ok(())
        }
        fn command(&mut self,c:Command)->Result<(),()> {
            if self.fail_command{return Err(());}
            self.command=Some(c);self.words=0;self.busy_until=self.reads+self.busy_per_command;
            self.status=if matches!(c,Command::Read|Command::Write|Command::Identify){0x48}else{0x40};Ok(())
        }
        fn read_word(&mut self)->Result<u16,()> {
            if self.fail_word==Some(self.words){return Err(());}
            let word=if self.command==Some(Command::Identify) {
                self.identify_words.as_ref().map_or(self.words as u16,|words|words[self.words])
            }else{self.words as u16};self.words+=1;if self.words==256{self.status=0x40;}Ok(word)
        }
        fn write_word(&mut self,w:u16)->Result<(),()> {
            if self.fail_word==Some(self.words){return Err(());}
            self.written.push(w);self.words+=1;if self.words==256{self.status=0x40;}Ok(())
        }
        fn yield_cpu(&mut self)->Result<(),()> {self.yields+=1;if self.fail_yield{Err(())}else{Ok(())}}
    }

    fn identity_fixture()->(Identity,[u16;256]) {
        let expected=Identity {sectors:32,serial:*b"RAR-DATA-TEST-000001",model:*b"RAR MODERN TEST DATA                    "};
        let mut words=[0u16;256];
        words[0]=0x0040;words[49]=1<<9;words[83]=0x4000|(1<<12)|(1<<10);
        words[86]=(1<<12)|(1<<10);words[87]=0x4000;
        words[60]=32;words[100]=32;words[106]=0x6000;
        for (start,text) in [(10,&expected.serial[..]),(27,&expected.model[..])] {
            for (i,pair) in text.chunks_exact(2).enumerate() {
                words[start+i]=u16::from_be_bytes([pair[0],pair[1]]);
            }
        }
        (expected,words)
    }
    #[test] fn identify_exact_profile_then_permit_sector_io() {
        let (expected,words)=identity_fixture();
        for geometry in [0,0x4000,0x6000] {
            let mut words=words;words[106]=geometry;
            let mut fake=Fake::new();fake.identify_words=Some(words);
            let mut device=Device::identify(fake,expected).unwrap();
            assert_eq!(device.io.command,Some(Command::Identify));
            assert_eq!(device.io.words,256);
            assert_eq!(device.io.registers,vec![(Register::Head,0xa0),(Register::Count,0),
                (Register::LbaLow,0),(Register::LbaMid,0),(Register::LbaHigh,0)]);
            assert_eq!(device.sectors,32);
            device.read512(31).unwrap();
            assert_eq!(device.read512(32),Err(Error::Bounds));
        }
    }
    #[test] fn identify_rejects_wrong_device_features_geometry_and_text() {
        let (expected,words)=identity_fixture();
        for (index,value) in [(0,0x8040),(49,0),(83,0),(83,0xd400),(83,0x4400),
            (86,0),(87,0),(60,31),(61,1),(100,31),(101,1),(102,1),(103,1),
            (106,0x7000),(106,0x6001),(106,0xffff),(117,256),(118,1),(10,0),(27,0)] {
            let mut bad=words;bad[index]=value;
            assert_eq!(identify_matches(&bad,&expected),Err(Error::Identity));
            let mut fake=Fake::new();fake.identify_words=Some(bad);
            assert!(matches!(Device::identify(fake,expected),Err(Error::Identity)));
        }
        let mut lba28=words;lba28[83]&=!(1<<10);lba28[86]&=!(1<<10);lba28[100]=0;
        assert_eq!(identify_matches(&lba28,&expected),Ok(()));
        lba28[100]=32;assert_eq!(identify_matches(&lba28,&expected),Err(Error::Identity));
        for sectors in [0,1<<28,u32::MAX] {
            let mut bad=expected;bad.sectors=sectors;
            assert!(matches!(Device::identify(Fake::new(),bad),Err(Error::Bounds)));
        }
        for byte in [0,0x1f,0x7f,0xff] {
            let mut bad=expected;bad.serial[0]=byte;
            assert!(matches!(Device::identify(Fake::new(),bad),Err(Error::Bounds)));
        }
        let mut bad=expected;bad.model=[b' ';40];
        assert!(matches!(Device::identify(Fake::new(),bad),Err(Error::Bounds)));
    }
    #[test] fn identify_never_constructs_after_transfer_or_phase_failure() {
        let (expected,words)=identity_fixture();
        for stop in 0..256 {
            let mut fake=Fake::new();fake.identify_words=Some(words);fake.fail_word=Some(stop);
            assert!(matches!(Device::identify(fake,expected),Err(Error::Transport)));
        }
        for at in 0..5 {
            let mut fake=Fake::new();fake.fail_register_at=Some(at);
            assert!(matches!(Device::identify(fake,expected),Err(Error::Transport)));
        }
        let mut fake=Fake::new();fake.fail_command=true;
        assert!(matches!(Device::identify(fake,expected),Err(Error::Transport)));
        for (from,status) in [(1,0),(1,255),(1,0x80),(7,0x41),(7,0x40),(12,0x60),(12,0x48)] {
            let mut fake=Fake::new();fake.identify_words=Some(words);
            fake.status_override=Some((from,status));
            assert!(Device::identify(fake,expected).is_err());
        }
    }

    #[test] fn exact_single_sector_little_endian_sequence() {
        let mut d=Device::test_device(Fake::new(),0x01020305).unwrap();
        let bytes=d.read512(0x01020304).unwrap();
        for i in 0..256 {assert_eq!(&bytes[2*i..2*i+2],&(i as u16).to_le_bytes());}
        assert_eq!(d.io.registers,vec![(Register::Head,0xe1),(Register::Count,1),
            (Register::LbaLow,4),(Register::LbaMid,3),(Register::LbaHigh,2)]);
        assert_eq!(d.io.words,256);assert_eq!(d.io.command,Some(Command::Read));
        d.write512(0,&bytes).unwrap();assert_eq!(d.io.written,(0..256).collect::<Vec<u16>>());
        assert_eq!(d.io.command,Some(Command::Write));d.flush().unwrap();
        assert_eq!(d.io.command,Some(Command::Flush));assert!(!d.poisoned());
    }
    #[test] fn invalid_geometry_and_lba_do_not_touch_io() {
        for n in [0,1<<28,u32::MAX] {assert!(Device::test_device(Fake::new(),n).is_err());}
        let mut d=Device::test_device(Fake::new(),32).unwrap();
        assert_eq!(d.read512(32),Err(Error::Bounds));
        assert_eq!(d.write512(u32::MAX,&[0;512]),Err(Error::Bounds));
        assert_eq!(d.io.reads,0);assert!(d.io.registers.is_empty());assert!(!d.poisoned());
    }
    #[test] fn missing_faulted_busy_and_drq_stuck_are_bounded() {
        for (status,error) in [(0,Error::Unavailable),(255,Error::Unavailable),
            (0x41,Error::Device),(0x60,Error::Device),(0x80,Error::Timeout),(0x48,Error::Timeout),
            (0x10,Error::Timeout),(0x02,Error::Timeout)] {
            let mut f=Fake::new();f.status=status;let mut d=Device::test_device(f,32).unwrap();
            assert_eq!(d.read512(0),Err(error));assert!(d.poisoned());
            assert!(d.io.reads<=POLLS);assert!(d.io.yields<=POLLS/16);
            assert!(d.io.registers.is_empty());
            let count=d.io.reads;assert_eq!(d.flush(),Err(Error::Poisoned));assert_eq!(d.io.reads,count);
        }
    }
    #[test] fn every_partial_transfer_error_poisoned_without_retry() {
        for stop in 0..256 {
            let mut f=Fake::new();f.fail_word=Some(stop);
            let mut read=Device::test_device(f.clone(),32).unwrap();
            assert_eq!(read.read512(0),Err(Error::Transport));assert!(read.poisoned());
            assert_eq!(read.io.words,stop);assert_eq!(read.read512(0),Err(Error::Poisoned));
            let mut write=Device::test_device(f,32).unwrap();
            assert_eq!(write.write512(0,&[7;512]),Err(Error::Transport));
            assert_eq!(write.io.written.len(),stop);assert!(write.poisoned());
            assert_eq!(write.write512(0,&[7;512]),Err(Error::Poisoned));
        }
        let mut f=Fake::new();f.fail_command=true;let mut d=Device::test_device(f,32).unwrap();
        assert_eq!(d.flush(),Err(Error::Transport));assert!(d.poisoned());
    }
    #[test] fn data_and_completion_phase_failures_never_report_success() {
        for write in [false,true] {for (from,status,error,words) in [
            (7,0x41,Error::Device,0),(7,0x80,Error::Timeout,0),(7,0x08,Error::Timeout,0),
            (12,0x60,Error::Device,256),(12,0x48,Error::Timeout,256),(12,0x10,Error::Timeout,256),
        ] {
            let mut f=Fake::new();f.status_override=Some((from,status));
            let mut d=Device::test_device(f,32).unwrap();
            let result=if write{d.write512(0,&[9;512])}else{d.read512(0).map(|_|())};
            assert_eq!(result,Err(error));assert!(d.poisoned());assert_eq!(d.io.words,words);
            assert!(d.io.reads<=POLLS+15);
            assert_eq!(d.flush(),Err(Error::Poisoned));
        }}
        for status in [0x41,0x60,0x80,0x48,0x10,0x02] {
            let mut f=Fake::new();f.status_override=Some((2,status));
            let mut d=Device::test_device(f,32).unwrap();
            assert!(d.flush().is_err());assert!(d.poisoned());assert!(d.io.reads<=POLLS+5);
            assert_eq!(d.io.command,Some(Command::Flush));
        }
    }
    #[test] fn every_transport_boundary_fails_closed() {
        for at in [1,7,12] {
            let mut f=Fake::new();f.fail_status_at=Some(at);let mut d=Device::test_device(f,32).unwrap();
            assert_eq!(d.read512(0),Err(Error::Transport));assert!(d.poisoned());
        }
        for at in 0..5 {
            let mut f=Fake::new();f.fail_register_at=Some(at);let mut d=Device::test_device(f,32).unwrap();
            assert_eq!(d.write512(0,&[0;512]),Err(Error::Transport));
            assert!(d.poisoned());assert_eq!(d.io.command,None);
        }
        for write in [false,true] {
            let mut f=Fake::new();f.fail_command=true;let mut d=Device::test_device(f,32).unwrap();
            let result=if write{d.write512(0,&[0;512])}else{d.read512(0).map(|_|())};
            assert_eq!(result,Err(Error::Transport));assert!(d.poisoned());assert_eq!(d.io.words,0);
        }
        let mut f=Fake::new();f.status=0x80;f.fail_yield=true;
        let mut d=Device::test_device(f,32).unwrap();assert_eq!(d.read512(0),Err(Error::Transport));
        assert_eq!(d.io.reads,16);assert_eq!(d.io.yields,1);assert!(d.poisoned());
    }
    #[test] fn busy_transitions_yield_then_complete_without_retry() {
        let mut f=Fake::new();f.busy_until=20;f.busy_per_command=20;
        let mut d=Device::test_device(f,32).unwrap();
        assert!(d.read512(0).is_ok());assert!(d.io.yields>=2);
        d.write512(0,&[1;512]).unwrap();d.flush().unwrap();assert!(!d.poisoned());
    }
}
