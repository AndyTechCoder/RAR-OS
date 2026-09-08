//! Modern ring3 service loops. Raw device grants never reach applications.
use crate::{abi::*,pio,store,transport,session,services,
    receive,deliver,syscall,check,report,yield_now};
#[path="render.rs"] mod render;
struct Ports {handle:u64}
fn operation(handle:u64,op:u64,value:u64)->Result<u64,()>{
    let result=syscall(DEVICE,handle,op,value,0);
    if result<0{Err(())}else{Ok(result as u64)}
}
impl pio::Io for Ports {
    fn status(&mut self)->Result<u8,()>{u8::try_from(operation(self.handle,0,0)?).map_err(|_|())}
    fn register(&mut self,reg:pio::Register,value:u8)->Result<(),()>{
        let op=match reg{pio::Register::Count=>1,pio::Register::LbaLow=>2,
            pio::Register::LbaMid=>3,pio::Register::LbaHigh=>4,pio::Register::Head=>5};
        if operation(self.handle,op,value as u64)?==0{Ok(())}else{Err(())}
    }
    fn command(&mut self,command:pio::Command)->Result<(),()>{
        let op=match command{pio::Command::Identify=>6,pio::Command::Read=>7,
            pio::Command::Write=>8,pio::Command::Flush=>9};
        if operation(self.handle,op,0)?==0{Ok(())}else{Err(())}
    }
    fn read_word(&mut self)->Result<u16,()>{u16::try_from(operation(self.handle,10,0)?).map_err(|_|())}
    fn write_word(&mut self,word:u16)->Result<(),()>{
        if operation(self.handle,11,word as u64)?==0{Ok(())}else{Err(())}
    }
    fn yield_cpu(&mut self)->Result<(),()>{
        if syscall(YIELD,0,0,0,0)==0{Ok(())}else{Err(())}
    }
}
fn identify(boot:&Boot)->Result<pio::Device<Ports>,()>{
    if !matches!(boot.role,1|9)||boot.caps[DEVICE_CAP]==0{return Err(());}
    let expected=pio::Identity{sectors:u32::try_from(boot.device_sectors).map_err(|_|())?,
        serial:boot.device_serial,model:boot.device_model};
    pio::Device::identify(Ports{handle:boot.caps[DEVICE_CAP]},expected).map_err(|_|())
}
pub fn storage(boot:&Boot)->!{
    // One identify/mount only. Never seed, format, remount or retry on failure.
    let mut server=identify(boot).ok().and_then(|device|{
        store::Store::mount(device,boot.device_sectors as u32,boot.peers[4],boot.peers[6]).ok()
    }).map(transport::Server::new);
    loop{
        let e=receive(boot.caps[SELF_RECV]);
        if !matches!(e.sender,4|6)||e.generation!=boot.peers[e.sender as usize]{continue;}
        let reply=match server.as_mut(){
            Some(server)=>server.handle(e.sender,e.generation,&e.bytes),
            None=>transport::unavailable_response(&e.bytes),
        };
        if let Some(reply)=reply{
            let slot=if e.sender==4{FILES}else{TERMINAL};
            // One attempt only: a lost durable ACK remains uncertain at the app.
            match syscall(SEND,boot.caps[slot],reply.as_ptr() as u64,128,0){
                0|-3|-4=>{},_=>crate::fail(),
            }
        }
    }
}
pub fn keyboard(boot:&Boot)->!{
    let mut decoder=services::Keyboard::new();report(1);
    loop{
        let status=syscall(PORT_READ,boot.caps[INPUT],0x64,0,0);check((0..=255).contains(&status));
        if status&1!=0{
            let byte=syscall(PORT_READ,boot.caps[INPUT],0x60,0,0);check((0..=255).contains(&byte));
            if let Some(key)=decoder.feed_status(status as u8,byte as u8){
                if let Some(m)=services::apps::key_wire(key){deliver(boot.caps[SHELL],&m);}
            }
        }
        yield_now();
    }
}
pub fn compositor(boot:&Boot)->!{
    let mut state=services::Compositor::new(boot.peers).unwrap_or_else(|_|crate::fail());
    render::draw(boot,&state);report(2);
    loop{
        let e=receive(boot.caps[SELF_RECV]);
        if state.apply(e.sender,e.generation,&e.bytes)==Ok(true){render::draw(boot,&state);}
    }
}
/// M4.1 has no update messages/loader. Never turn possession of Manager into I/O.
pub fn manager(boot:&Boot)->!{loop{let _=receive(boot.caps[SELF_RECV]);}}
/// Identify exact separate System controller once, without writes. M4.2 will
/// supply the signed System protocol; this initial entry never claims an update.
pub fn system(boot:&Boot)->!{
    let _device=identify(boot);
    loop{let _=receive(boot.caps[SELF_RECV]);}
}
pub struct FileRuntime<'a>{boot:&'a Boot}
impl<'a> FileRuntime<'a>{
    pub fn new(boot:&'a Boot)->Result<Self,()>{
        if !matches!(boot.role,4|6)||boot.caps[STORAGE]==0||boot.caps[SELF_RECV]==0{return Err(());}
        Ok(Self{boot})
    }
}
impl session::Runtime for FileRuntime<'_>{
    fn now(&mut self)->Result<u64,()>{
        let value=syscall(TICKS,0,0,0,0);if value<0{Err(())}else{Ok(value as u64)}
    }
    fn send_once(&mut self,frame:&[u8;128])->Result<(),()>{
        if syscall(SEND,self.boot.caps[STORAGE],frame.as_ptr() as u64,128,0)==0{Ok(())}else{Err(())}
    }
    fn poll(&mut self)->Result<Option<session::Envelope>,()>{
        crate::poll_checked(self.boot.caps[SELF_RECV])?.map(|e|{
            Ok(session::Envelope{sender:e.sender,generation:e.generation,
                length:usize::try_from(e.length).map_err(|_|())?,bytes:e.bytes})
        }).transpose()
    }
    fn yield_now(&mut self)->Result<(),()>{
        if syscall(YIELD,0,0,0,0)==0{Ok(())}else{Err(())}
    }
}
