//! Candidate RAR NE2000/DP8390 programmed-I/O driver.
//! No port instructions: only the separately reviewed adapter can provide Io.
//! DP8390D July1995 register/ring specification; NE2000 16-bit I/O aperture.
//! No host or guest-memory bus-master DMA is requested.
#![forbid(unsafe_code)]
use crate::network::{MAX_FRAME,MIN_FRAME};
const START:u8=0x46;
const STOP:u8=0x80;
const TX:u8=0x40;
const POLLS:usize=512;
const TICK_LIMIT:u64=20;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Invalid,Io,Timeout,Identity,Corrupt,Closed}
pub trait Io{
    fn read(&mut self,register:u8)->Result<u8,()>;
    fn write(&mut self,register:u8,value:u8)->Result<(),()>;
    fn read_word(&mut self)->Result<u16,()>;
    fn write_word(&mut self,value:u16)->Result<(),()>;
    fn reset(&mut self)->Result<(),()>;
    fn ticks(&mut self)->Result<u64,()>;
    fn yield_cpu(&mut self)->Result<(),()>;
}
pub struct Device<I:Io>{io:I,next:u8,closed:bool}
impl<I:Io> Device<I>{
    pub fn initialize(io:I,mac:[u8;6])->Result<Self,Error>{
        if mac==[0;6]||mac[0]&1!=0{return Err(Error::Invalid);}
        let mut d=Self{io,next:START+1,closed:false};
        if let Err(e)=d.setup(mac){d.close();return Err(e);}Ok(d)
    }
    fn r(&mut self,r:u8)->Result<u8,Error>{self.io.read(r).map_err(|_|Error::Io)}
    fn w(&mut self,r:u8,v:u8)->Result<(),Error>{self.io.write(r,v).map_err(|_|Error::Io)}
    fn setup(&mut self,mac:[u8;6])->Result<(),Error>{
        self.io.reset().map_err(|_|Error::Io)?;
        self.wait(0x80,0)?;
        self.w(0,0x21)?;self.w(14,0x49)?; // STOP, little-endian word PIO.
        self.w(10,0)?;self.w(11,0)?;self.w(15,0)?; // No interrupts.
        self.w(12,0x20)?;self.w(13,2)?; // Monitor/internal loopback during setup.
        self.w(4,TX)?;self.w(1,START)?;self.w(2,STOP)?;self.w(3,START)?;
        let mut prom=[0u8;12];self.read_linear(0,&mut prom,true)?;
        for i in 0..6 {
            if prom[2*i]!=mac[i]||prom[2*i+1]!=mac[i]{return Err(Error::Identity);}
        }
        self.w(0,0x61)?; // Page1 while stopped.
        for (i,b) in mac.into_iter().enumerate(){self.w(1+i as u8,b)?;}
        self.w(7,self.next)?;
        for i in 8..16 {self.w(i,0)?;} // Empty multicast filter.
        for (i,b) in mac.into_iter().enumerate(){
            if self.r(1+i as u8)?!=b{return Err(Error::Identity);}
        }
        self.w(0,0x21)?;self.w(7,0xff)?;
        self.w(0,0x22)?;self.w(13,0)?;self.w(12,0)?;
        Ok(())
    }
    fn wait(&mut self,success:u8,failure:u8)->Result<(),Error>{
        let start=self.io.ticks().map_err(|_|Error::Io)?;
        let end=start.checked_add(TICK_LIMIT).ok_or(Error::Timeout)?;
        let mut last=start;
        for _ in 0..POLLS{
            let now=self.io.ticks().map_err(|_|Error::Io)?;
            if now<last||now>=end{return Err(Error::Timeout);}last=now;
            let status=self.r(7)?;
            if status&failure!=0{return Err(Error::Io);}
            if status&success!=0{return Ok(());}
            self.io.yield_cpu().map_err(|_|Error::Io)?;
        }
        Err(Error::Timeout)
    }
    fn begin_remote(&mut self,address:u16,length:usize,write:bool)->Result<(),Error>{
        if length==0||length>MAX_FRAME+1||length%2!=0{return Err(Error::Invalid);}
        self.w(0,0x22)?;self.w(7,0x40)?;
        self.w(8,address as u8)?;self.w(9,(address>>8)as u8)?;
        self.w(10,length as u8)?;self.w(11,(length>>8)as u8)?;
        self.w(0,if write{0x12}else{0x0a})
    }
    fn read_linear(&mut self,address:u16,out:&mut[u8],prom:bool)->Result<(),Error>{
        let rounded=out.len().checked_add(1).ok_or(Error::Invalid)?&!1;
        let end=usize::from(address).checked_add(rounded).ok_or(Error::Invalid)?;
        if out.is_empty()||out.len()>MAX_FRAME||
            if prom{address!=0||out.len()!=12}else{address<0x4600||end>0x8000}{
            return Err(Error::Invalid);
        }
        self.begin_remote(address,rounded,false)?;
        for pair in out.chunks_mut(2){
            let w=self.io.read_word().map_err(|_|Error::Io)?.to_le_bytes();
            pair[0]=w[0];if pair.len()==2{pair[1]=w[1];}
        }
        self.wait(0x40,0)?;self.w(7,0x40)?;self.w(0,0x22)
    }
    fn read_ring(&mut self,address:u16,out:&mut[u8])->Result<(),Error>{
        if !(0x4600..0x8000).contains(&address)||out.is_empty()||out.len()>MAX_FRAME{
            return Err(Error::Invalid);
        }
        let first=out.len().min(0x8000-usize::from(address));
        self.read_linear(address,&mut out[..first],false)?;
        if first<out.len(){self.read_linear(0x4600,&mut out[first..],false)?;}Ok(())
    }
    pub fn close(&mut self){
        if !self.closed {self.closed=true;let _=self.io.write(0,0x21);}
    }
    pub fn transmit(&mut self,frame:&[u8])->Result<(),Error>{
        if self.closed{return Err(Error::Closed);}
        if !(MIN_FRAME..=MAX_FRAME).contains(&frame.len()){return Err(Error::Invalid);}
        let result=self.transmit_inner(frame);
        if result.is_err(){self.close();}result
    }
    fn transmit_inner(&mut self,frame:&[u8])->Result<(),Error>{
        if self.r(0)?&4!=0{return Err(Error::Io);}
        self.begin_remote(u16::from(TX)<<8,(frame.len()+1)&!1,true)?;
        for pair in frame.chunks(2){
            self.io.write_word(u16::from_le_bytes([pair[0],if pair.len()==2{pair[1]}else{0}]))
                .map_err(|_|Error::Io)?;
        }
        self.wait(0x40,0)?;self.w(7,0x4a)?;
        self.w(4,TX)?;self.w(5,frame.len()as u8)?;self.w(6,(frame.len()>>8)as u8)?;
        self.w(0,0x26)?;self.wait(2,8)?;self.w(7,0x0a)?;Ok(())
    }
    pub fn receive(&mut self,out:&mut[u8;MAX_FRAME])->Result<Option<usize>,Error>{
        if self.closed{return Err(Error::Closed);}
        let result=self.receive_inner(out);
        if result.is_err(){out.fill(0);self.close();}result
    }
    fn receive_inner(&mut self,out:&mut[u8;MAX_FRAME])->Result<Option<usize>,Error>{
        self.w(0,0x22)?;
        if self.r(7)?&0x10!=0{return Err(Error::Corrupt);} // Overflow: stop, never guess ring state.
        self.w(0,0x62)?;let current=self.r(7)?;self.w(0,0x22)?;
        if !(START..STOP).contains(&current){return Err(Error::Corrupt);}
        if current==self.next{return Ok(None);}
        let mut header=[0u8;4];self.read_ring(u16::from(self.next)<<8,&mut header)?;
        let count=usize::from(u16::from_le_bytes([header[2],header[3]]));
        // Count includes Ethernet CRC but not the four-byte NIC ring header.
        if header[0]&0x1f!=1||!(MIN_FRAME+4..=MAX_FRAME+4).contains(&count){
            return Err(Error::Corrupt);
        }
        let pages=(count+4).div_ceil(256);
        let expected=START+(usize::from(self.next-START)+pages) as u8%(STOP-START);
        if header[1]!=expected||header[1]==self.next{return Err(Error::Corrupt);}
        let length=count-4;
        self.read_ring((u16::from(self.next)<<8)+4,&mut out[..length])?;
        out[length..].fill(0);
        self.next=expected;
        self.w(3,if self.next==START{STOP-1}else{self.next-1})?;
        self.w(7,1)?;Ok(Some(length))
    }
}
impl<I:Io> Drop for Device<I>{fn drop(&mut self){self.close();}}

#[cfg(test)]
mod tests{
    use super::*;
    const MAC:[u8;6]=[2,0,0,0,0,1];
    struct Fake{pages:[[u8;16];2],cmd:u8,ram:[u8;32768],address:usize,left:usize,
        ticks:u64,freeze:bool,hang_tx:bool,writes:usize,transmitted:usize}
    impl Fake{
        fn new()->Self{
            let mut f=Self{pages:[[0;16];2],cmd:0x21,ram:[0;32768],address:0,left:0,
                ticks:0,freeze:false,hang_tx:false,writes:0,transmitted:0};
            for(i,b)in MAC.into_iter().enumerate(){f.ram[2*i]=b;f.ram[2*i+1]=b;}f
        }
        fn done(&mut self){if self.left==0{self.pages[0][7]|=0x40;}}
        fn inject(&mut self,page:u8,frame:&[u8]){
            let count=frame.len()+4;let pages=(count+4).div_ceil(256);
            let next=START+((usize::from(page-START)+pages)%usize::from(STOP-START))as u8;
            let mut bytes=std::vec![1,next,count as u8,(count>>8)as u8];
            bytes.extend_from_slice(frame);bytes.extend_from_slice(&[0;4]);
            for(i,b)in bytes.into_iter().enumerate(){
                let at=0x4600+((usize::from(page)<<8)-0x4600+i)%(0x8000-0x4600);
                self.ram[at]=b;
            }
            self.pages[1][7]=next;self.pages[0][7]|=1;
        }
    }
    impl Io for Fake{
        fn read(&mut self,r:u8)->Result<u8,()>{
            if r>=16{return Err(());}if r==0{return Ok(self.cmd);}
            Ok(self.pages[((self.cmd>>6)&1)as usize][r as usize])
        }
        fn write(&mut self,r:u8,v:u8)->Result<(),()>{
            if r>=16{return Err(());}self.writes+=1;
            if r==0{
                self.cmd=v;
                if v==0x0a||v==0x12{
                    self.address=usize::from(u16::from_le_bytes([self.pages[0][8],self.pages[0][9]]));
                    self.left=usize::from(u16::from_le_bytes([self.pages[0][10],self.pages[0][11]]));
                }
                if v==0x26&&!self.hang_tx{
                    self.transmitted+=1;self.pages[0][7]|=2;self.cmd=0x22;
                }
            }else if self.cmd&0x40==0&&r==7{self.pages[0][7]&=!v;}
            else{self.pages[((self.cmd>>6)&1)as usize][r as usize]=v;}
            Ok(())
        }
        fn read_word(&mut self)->Result<u16,()>{
            if self.left<2||self.address+2>self.ram.len(){return Err(());}
            let w=u16::from_le_bytes([self.ram[self.address],self.ram[self.address+1]]);
            self.address+=2;self.left-=2;self.done();Ok(w)
        }
        fn write_word(&mut self,w:u16)->Result<(),()>{
            if self.left<2||self.address<0x4000||self.address+2>0x4600{return Err(());}
            self.ram[self.address..self.address+2].copy_from_slice(&w.to_le_bytes());
            self.address+=2;self.left-=2;self.done();Ok(())
        }
        fn reset(&mut self)->Result<(),()>{self.cmd=0x21;self.pages[0][7]=0x80;Ok(())}
        fn ticks(&mut self)->Result<u64,()>{Ok(self.ticks)}
        fn yield_cpu(&mut self)->Result<(),()>{if !self.freeze{self.ticks+=1;}Ok(())}
    }
    #[test]fn initialize_proves_mac_and_masks_interrupts(){
        let d=Device::initialize(Fake::new(),MAC).unwrap();
        assert_eq!(d.io.pages[0][15],0);assert_eq!(d.io.pages[0][12],0);
        assert_eq!(d.io.pages[0][14],0x49);assert_eq!(&d.io.pages[1][1..7],&MAC);
        let mut bad=Fake::new();bad.ram[1]=3;
        assert!(matches!(Device::initialize(bad,MAC),Err(Error::Identity)));
    }
    #[test]fn packet_transmit_copies_exact_bytes_and_zero_pads_last_word(){
        let mut d=Device::initialize(Fake::new(),MAC).unwrap();
        let data=[0x5a;61];d.transmit(&data).unwrap();
        assert_eq!(&d.io.ram[0x4000..0x403d],&data);
        assert_eq!(d.io.ram[0x403d],0);assert_eq!(d.io.transmitted,1);
        let writes=d.io.writes;
        assert_eq!(d.transmit(&[0;59]),Err(Error::Invalid));assert_eq!(d.io.writes,writes);
    }
    #[test]fn receive_copies_payload_discards_crc_and_wraps_safely(){
        for page in [START+1,STOP-1]{
            let mut d=Device::initialize(Fake::new(),MAC).unwrap();d.next=page;
            let frame=[0x3c;MAX_FRAME];d.io.inject(page,&frame);
            let mut out=[0xff;MAX_FRAME];
            assert_eq!(d.receive(&mut out),Ok(Some(MAX_FRAME)));assert_eq!(out,frame);
            assert_eq!(d.receive(&mut out),Ok(None));
        }
    }
    #[test]fn invalid_ring_header_or_overflow_stops_instead_of_following_pointer(){
        for field in 0..4{
            let mut d=Device::initialize(Fake::new(),MAC).unwrap();let page=d.next;
            d.io.inject(page,&[0;60]);d.io.ram[(usize::from(page)<<8)+field]=0xff;
            let mut out=[0xaa;MAX_FRAME];
            assert_eq!(d.receive(&mut out),Err(Error::Corrupt));assert_eq!(out,[0;MAX_FRAME]);
            assert_eq!(d.receive(&mut out),Err(Error::Closed));
        }
        let mut d=Device::initialize(Fake::new(),MAC).unwrap();d.io.pages[0][7]|=0x10;
        assert_eq!(d.receive(&mut [0;MAX_FRAME]),Err(Error::Corrupt));
    }
    #[test]fn stalled_clock_and_hung_transmit_are_both_bounded_and_sticky(){
        for freeze in [false,true]{
            let mut d=Device::initialize(Fake::new(),MAC).unwrap();
            d.io.hang_tx=true;d.io.freeze=freeze;
            assert_eq!(d.transmit(&[0;60]),Err(Error::Timeout));
            let writes=d.io.writes;
            assert_eq!(d.transmit(&[0;60]),Err(Error::Closed));assert_eq!(d.io.writes,writes);
            assert_eq!(d.io.cmd,0x21);
        }
    }
}
