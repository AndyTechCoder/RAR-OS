//! Candidate native service bridge, selectable only by reviewed Expansion builds.
//! No direct port instructions; all operations cross the dedicated kernel grant.
use super::{abi,syscall,fail,yield_now,poll_checked,ne2k,network_service};
use super::channel::Budget;
use super::network::Endpoint;
struct Io{handle:u64}
impl Io{
    fn call(&self,op:u64,value:u64,extra:u64)->Result<u64,()>{
        let r=syscall(abi::NETWORK,self.handle,op,value,extra);
        if r<0{Err(())}else{Ok(r as u64)}
    }
}
impl ne2k::Io for Io{
    fn read(&mut self,r:u8)->Result<u8,()>{self.call(0,u64::from(r),0)?.try_into().map_err(|_|())}
    fn write(&mut self,r:u8,v:u8)->Result<(),()>{if self.call(1,u64::from(r),u64::from(v))?==0{Ok(())}else{Err(())}}
    fn read_word(&mut self)->Result<u16,()>{self.call(2,0,0)?.try_into().map_err(|_|())}
    fn write_word(&mut self,v:u16)->Result<(),()>{if self.call(3,u64::from(v),0)?==0{Ok(())}else{Err(())}}
    fn reset(&mut self)->Result<(),()>{if self.call(4,0,0)?==0{Ok(())}else{Err(())}}
    fn ticks(&mut self)->Result<u64,()>{
        let n=syscall(abi::TICKS,0,0,0,0);if n<0{Err(())}else{Ok(n as u64)}
    }
    fn yield_cpu(&mut self)->Result<(),()>{if syscall(abi::YIELD,0,0,0,0)==0{Ok(())}else{Err(())}}
}
pub fn network(b:&abi::Boot)->!{
    if b.version!=abi::EXPANSION_VERSION||b.role!=7||b.peers[6]==0{fail();}
    let a=Endpoint{mac:[2,0,0,0,0,1],ip:[10,42,0,1],port:4000};
    let z=Endpoint{mac:[2,0,0,0,0,2],ip:[10,42,0,2],port:4001};
    // Fixed build-profile fixture, not production discovery or authenticated pairing.
    let(local,peer)=if cfg!(rar_network_peer_b){(z,a)}else{(a,z)};
    let mut io=Io{handle:b.caps[abi::DEVICE_CAP]};
    let now=ne2k::Io::ticks(&mut io).unwrap_or_else(|_|fail());
    let expires=now.checked_add(60_000).unwrap_or_else(||fail());
    let device=ne2k::Device::initialize(io,local.mac).unwrap_or_else(|_|fail());
    let policy=network_service::Policy{principal:6,incarnation:b.peers[6],interface:1,
        local,peer,issued:now,expires,tx:Budget{packets:1024,bytes:1_048_576},
        rx:Budget{packets:4096,bytes:2_097_152}};
    let mut service=network_service::Service::new(device,policy).unwrap_or_else(|_|fail());
    let mut reply:Option<network_service::Reply>=None;
    loop{
        // Poll before at most one client request, even under IPC backpressure.
        if service.poll().is_err(){service.close();fail();}
        if let Some(pending)=&reply{
            let bytes=pending.bytes();
            match syscall(abi::SEND,b.caps[1],bytes.as_ptr()as u64,bytes.len()as u64,0){
                0=>reply=None,-4=>{},_=>{service.close();fail();}
            }
        }
        if reply.is_none(){
            match poll_checked(b.caps[abi::SELF_RECV]){
                Ok(Some(e))=>{
                    if e.sender==6&&e.generation==b.peers[6]{
                        reply=Some(service.request(e.sender,e.generation,&e.bytes[..e.length as usize]));
                    }
                },
                Ok(None)=>{},Err(())=>{service.close();fail();}
            }
        }
        yield_now();
    }
}
