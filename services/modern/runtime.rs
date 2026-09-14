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
struct SystemIo(pio::Device<Ports>);
impl crate::system_volume::Io for SystemIo{
    fn read(&mut self,sector:u32)->Result<[u8;512],()>{self.0.read512(sector).map_err(|_|())}
    fn write(&mut self,sector:u32,bytes:&[u8;512])->Result<(),()>{self.0.write512(sector,bytes).map_err(|_|())}
    fn flush(&mut self)->Result<(),()>{self.0.flush().map_err(|_|())}
}
/// Selected only together with reviewed immutable System provisioning. The
/// legacy entry below remains unchanged until that native boot composition lands.
pub fn update_system(boot:&Boot,input:fn(u64)->Option<&'static[u8]>)->!{
    if boot.role!=9{crate::fail();}
    let device=identify(boot).unwrap_or_else(|_|crate::fail());
    let volume=crate::system_volume::Volume::mount(SystemIo(device),boot.device_sectors as u32)
        .unwrap_or_else(|_|crate::fail());
    crate::update_runtime::system(boot,volume,input)
}
pub fn storage(boot:&Boot)->!{
    #[cfg(rar_applications)] {application_storage(boot)}
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
                #[cfg(rar_applications)]
                if matches!(key,0x86..=0x89){
                    let mut m=[0;128];m[0]=1;m[1]=key;deliver(boot.caps[SHELL],&m);continue;
                }
                if let Some(m)=services::apps::key_wire(key){deliver(boot.caps[SHELL],&m);}
            }
        }
        yield_now();
    }
}
pub fn compositor(boot:&Boot)->!{
    #[cfg(rar_applications)] {application_compositor(boot)}
    let mut state=services::Compositor::new(boot.peers).unwrap_or_else(|_|crate::fail());
    render::draw(boot,&state);report(2);
    loop{
        let e=receive(boot.caps[SELF_RECV]);
        if state.settings_binding(crate::settings_binding(boot)).is_err(){crate::fail();}
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

#[cfg(rar_applications)]
fn application_storage(boot:&Boot)->!{
    use crate::{application_runtime as a,app_control as c,app_documents::{Owner,Grant}};
    let catalog=a::wait_catalog(boot);
    let owner=Owner::from_verified_identity(catalog.owner).unwrap_or_else(|_|crate::fail());
    let mut server=identify(boot).ok().and_then(|device|
        store::Store::mount_applications(device,boot.device_sectors as u32,boot.peers[4],boot.peers[6],owner).ok()
    ).map(transport::Server::new);
    let mut incarnation=0u64;
    loop {
        let binding=a::query(boot,0);
        if binding.is_none_or(|r|r.incarnation!=incarnation){
            if let Some(s)=&mut server{s.app_store().revoke_private();}
            incarnation=0;
        }
        let e=match crate::poll_checked(boot.caps[SELF_RECV]){
            Ok(Some(e))if e.length==128=>e,
            Ok(_)=>{crate::yield_now();continue;},
            Err(())=>crate::fail(),
        };
        if e.sender==8&&Some(e.generation)==a::peer(boot,8){
            if let Ok((4,0,inc))=c::parse(&e.bytes){
                let current=a::query(boot,0);
                if current.is_some_and(|r|r.incarnation==inc&&r.state==1&&r.owner==catalog.owner){
                    let ok=if let Some(s)=&mut server{
                        if incarnation==0{
                            let result=Grant::new(owner,10,inc).ok().and_then(|g|s.app_store().rebind_private(g).ok());
                            if result.is_some(){incarnation=inc;}
                        }
                        incarnation==inc&&s.app_store().install_private().is_ok()
                    }else{false};
                    let response=c::control(if ok{5}else{6},0,inc).unwrap();
                    // An uncertain install is never retried automatically.
                    let _=a::send(boot,8,&response);
                }
            }continue;
        }
        if e.sender==10&&e.generation==incarnation&&
            a::query(boot,0).is_some_and(|r|r.incarnation==e.generation&&r.state==2){
            if let Some(s)=&mut server{
                if let Some(reply)=s.app_store().process_app(e.sender,e.generation,&e.bytes){
                    if s.app_store().app_reply_current(e.sender,e.generation){
                        // Expected recipient is checked atomically with SEND in
                        // kernel op7; a query-before-SEND alone would race reuse.
                        let _=a::send_app(boot,0,e.generation,&reply);
                    }
                }
            }continue;
        }
        if !matches!(e.sender,4|6)||e.generation!=boot.peers[e.sender as usize]{continue;}
        let reply=match &mut server {
            Some(s)=>s.handle(e.sender,e.generation,&e.bytes),
            None=>transport::unavailable_response(&e.bytes),
        };
        if let Some(reply)=reply{
            let slot=if e.sender==4{FILES}else{TERMINAL};
            match syscall(SEND,boot.caps[slot],reply.as_ptr()as u64,128,0){0|-3|-4=>{},_=>crate::fail()}
        }
    }
}
#[cfg(rar_applications)]
fn application_compositor(boot:&Boot)->!{
    use crate::{application_runtime as a,app_control as c};
    let _=a::wait_catalog(boot);
    let mut state=services::Compositor::new(boot.peers).unwrap_or_else(|_|crate::fail());
    let mut incarnations=[0u64;2];let mut sequences=[0u32;2];
    render::draw(boot,&state);report(2);
    loop{
        let mut dirty=false;
        for index in 0..2{
            let inc=a::query(boot,index).map_or(0,|r|r.incarnation);
            if inc!=incarnations[index]{
                state.app_binding(index,inc).unwrap_or_else(|_|crate::fail());
                incarnations[index]=inc;sequences[index]=0;dirty=true;
            }
        }
        if state.settings_binding(crate::settings_binding(boot)).is_err(){crate::fail();}
        match crate::poll_checked(boot.caps[SELF_RECV]){
            Ok(Some(e))if e.length==128=>{
                if e.sender==0&&e.generation==boot.peers[0]{
                    if let Ok(compact)=c::parse_profile(&e.bytes){
                        state.compact=compact;dirty=true;
                    }else if let Ok((3,index,inc))=c::parse(&e.bytes){
                        if incarnations[index]==inc&&inc!=0{state.app_focus=Some(10+index as u8);dirty=true;}
                    }else if let Ok((index,inc,key))=c::parse_input(&e.bytes){
                        if state.app_focus==Some(10+index as u8)&&incarnations[index]==inc{
                            if let Some(sequence)=sequences[index].checked_add(1){
                                sequences[index]=sequence;
                                if let Ok(m)=services::app_sdk::protocol::input(sequence,key){
                                    let _=a::send_app(boot,index,inc,&m.encode());
                                }
                            }
                        }
                    }else if state.apply(e.sender,e.generation,&e.bytes)==Ok(true){dirty=true;}
                }else if state.apply(e.sender,e.generation,&e.bytes)==Ok(true){dirty=true;}
            },
            Ok(_)=>{},
            Err(())=>crate::fail(),
        }
        if dirty{render::draw(boot,&state);}
        crate::yield_now();
    }
}
