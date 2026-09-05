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
pub enum Error {Bounds,Unavailable,Device,Timeout,Transport,Poisoned}
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
pub struct Device<I:Io> {io:I,sectors:u32,poisoned:bool}
impl<I:Io> Device<I> {
    /// capacity must come from the fixed profile AND verified IDENTIFY, not IPC.
    /// Initialization/IDENTIFY validation and the syscall bridge remain pending.
    pub fn new(io:I,sectors:u32)->Result<Self,Error> {
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
        fail_register_at:Option<usize>,fail_yield:bool,busy_until:usize,busy_per_command:usize}
    impl Fake {
        fn new()->Self {Self {status:0x40,command:None,words:0,reads:0,yields:0,
            registers:vec![],written:vec![],fail_word:None,fail_command:false,status_override:None,
            fail_status_at:None,fail_register_at:None,fail_yield:false,busy_until:0,busy_per_command:0}}
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
            let word=self.words as u16;self.words+=1;if self.words==256{self.status=0x40;}Ok(word)
        }
        fn write_word(&mut self,w:u16)->Result<(),()> {
            if self.fail_word==Some(self.words){return Err(());}
            self.written.push(w);self.words+=1;if self.words==256{self.status=0x40;}Ok(())
        }
        fn yield_cpu(&mut self)->Result<(),()> {self.yields+=1;if self.fail_yield{Err(())}else{Ok(())}}
    }
    #[test] fn exact_single_sector_little_endian_sequence() {
        let mut d=Device::new(Fake::new(),0x01020305).unwrap();
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
        for n in [0,1<<28,u32::MAX] {assert!(Device::new(Fake::new(),n).is_err());}
        let mut d=Device::new(Fake::new(),32).unwrap();
        assert_eq!(d.read512(32),Err(Error::Bounds));
        assert_eq!(d.write512(u32::MAX,&[0;512]),Err(Error::Bounds));
        assert_eq!(d.io.reads,0);assert!(d.io.registers.is_empty());assert!(!d.poisoned());
    }
    #[test] fn missing_faulted_busy_and_drq_stuck_are_bounded() {
        for (status,error) in [(0,Error::Unavailable),(255,Error::Unavailable),
            (0x41,Error::Device),(0x60,Error::Device),(0x80,Error::Timeout),(0x48,Error::Timeout),
            (0x10,Error::Timeout),(0x02,Error::Timeout)] {
            let mut f=Fake::new();f.status=status;let mut d=Device::new(f,32).unwrap();
            assert_eq!(d.read512(0),Err(error));assert!(d.poisoned());
            assert!(d.io.reads<=POLLS);assert!(d.io.yields<=POLLS/16);
            assert!(d.io.registers.is_empty());
            let count=d.io.reads;assert_eq!(d.flush(),Err(Error::Poisoned));assert_eq!(d.io.reads,count);
        }
    }
    #[test] fn every_partial_transfer_error_poisoned_without_retry() {
        for stop in 0..256 {
            let mut f=Fake::new();f.fail_word=Some(stop);
            let mut read=Device::new(f.clone(),32).unwrap();
            assert_eq!(read.read512(0),Err(Error::Transport));assert!(read.poisoned());
            assert_eq!(read.io.words,stop);assert_eq!(read.read512(0),Err(Error::Poisoned));
            let mut write=Device::new(f,32).unwrap();
            assert_eq!(write.write512(0,&[7;512]),Err(Error::Transport));
            assert_eq!(write.io.written.len(),stop);assert!(write.poisoned());
            assert_eq!(write.write512(0,&[7;512]),Err(Error::Poisoned));
        }
        let mut f=Fake::new();f.fail_command=true;let mut d=Device::new(f,32).unwrap();
        assert_eq!(d.flush(),Err(Error::Transport));assert!(d.poisoned());
    }
    #[test] fn data_and_completion_phase_failures_never_report_success() {
        for write in [false,true] {for (from,status,error,words) in [
            (7,0x41,Error::Device,0),(7,0x80,Error::Timeout,0),(7,0x08,Error::Timeout,0),
            (12,0x60,Error::Device,256),(12,0x48,Error::Timeout,256),(12,0x10,Error::Timeout,256),
        ] {
            let mut f=Fake::new();f.status_override=Some((from,status));
            let mut d=Device::new(f,32).unwrap();
            let result=if write{d.write512(0,&[9;512])}else{d.read512(0).map(|_|())};
            assert_eq!(result,Err(error));assert!(d.poisoned());assert_eq!(d.io.words,words);
            assert!(d.io.reads<=POLLS+15);
            assert_eq!(d.flush(),Err(Error::Poisoned));
        }}
        for status in [0x41,0x60,0x80,0x48,0x10,0x02] {
            let mut f=Fake::new();f.status_override=Some((2,status));
            let mut d=Device::new(f,32).unwrap();
            assert!(d.flush().is_err());assert!(d.poisoned());assert!(d.io.reads<=POLLS+5);
            assert_eq!(d.io.command,Some(Command::Flush));
        }
    }
    #[test] fn every_transport_boundary_fails_closed() {
        for at in [1,7,12] {
            let mut f=Fake::new();f.fail_status_at=Some(at);let mut d=Device::new(f,32).unwrap();
            assert_eq!(d.read512(0),Err(Error::Transport));assert!(d.poisoned());
        }
        for at in 0..5 {
            let mut f=Fake::new();f.fail_register_at=Some(at);let mut d=Device::new(f,32).unwrap();
            assert_eq!(d.write512(0,&[0;512]),Err(Error::Transport));
            assert!(d.poisoned());assert_eq!(d.io.command,None);
        }
        for write in [false,true] {
            let mut f=Fake::new();f.fail_command=true;let mut d=Device::new(f,32).unwrap();
            let result=if write{d.write512(0,&[0;512])}else{d.read512(0).map(|_|())};
            assert_eq!(result,Err(Error::Transport));assert!(d.poisoned());assert_eq!(d.io.words,0);
        }
        let mut f=Fake::new();f.status=0x80;f.fail_yield=true;
        let mut d=Device::new(f,32).unwrap();assert_eq!(d.read512(0),Err(Error::Transport));
        assert_eq!(d.io.reads,16);assert_eq!(d.io.yields,1);assert!(d.poisoned());
    }
    #[test] fn busy_transitions_yield_then_complete_without_retry() {
        let mut f=Fake::new();f.busy_until=20;f.busy_per_command=20;
        let mut d=Device::new(f,32).unwrap();
        assert!(d.read512(0).is_ok());assert!(d.io.yields>=2);
        d.write512(0,&[1;512]).unwrap();d.flush().unwrap();assert!(!d.poisoned());
    }
}
