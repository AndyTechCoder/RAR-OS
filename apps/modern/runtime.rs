//! Modern application loops: durable replies and explicit uncertain-save UI.
use crate::{abi::*,services,receive,deliver,send,publish,fail,session,transport,drivers};
use services::apps::*;
use crate::desktop_wire as fs;
use transport::Outcome;
use crate::file_ui::{self,problem};
fn activated(m:&[u8;128])->bool {m[0]==2&&m[1..].iter().all(|&b|b==0)}
struct Io {pending:Pending,session:session::Session,state:file_ui::State}
impl Io {
    fn new(boot:&Boot)->Self{
        Self{pending:Pending::new(),
            session:session::Session::new(boot.peers[1],boot.peers[0]).unwrap_or_else(|_|fail()),
            state:file_ui::State::new()}
    }
    fn label(&self)->&'static [u8]{
        self.state.label(self.session.closed(),self.session.writes_locked(),self.session.input_lost())
    }
}
impl session::Input for Pending{
    fn push(&mut self,m:[u8;128])->bool{
        if activated(&m)||key_decode(&m).is_some(){Pending::push(self,m)}else{true}
    }
}
fn shell_event(boot:&Boot)->[u8;128]{
    loop{let e=receive(boot.caps[SELF_RECV]);
        if e.sender==0&&e.generation==boot.peers[0]{return e.bytes;}
    }
}
fn event(boot:&Boot,io:&mut Io)->[u8;128]{
    io.pending.pop().unwrap_or_else(||shell_event(boot))
}
fn call(boot:&Boot,io:&mut Io,op:u8,name:&[u8],data:&[u8])->Outcome{
    let mut invalid=[0;128];invalid[0]=fs::INVALID;
    if name.len()>12||data.len()>64{return Outcome::Reply(invalid);}
    let mut body=[0;128];body[0]=op;body[1]=name.len() as u8;body[2]=data.len() as u8;
    body[4..4+name.len()].copy_from_slice(name);body[16..16+data.len()].copy_from_slice(data);
    if crate::store::decode(&body).is_none(){return Outcome::Reply(invalid);}
    let mut runtime=drivers::FileRuntime::new(boot).unwrap_or_else(|_|fail());
    let result=match io.session.call(&mut runtime,&mut io.pending,&body){
        Ok(result)=>result,
        Err(transport::StartError::Invalid)=>Outcome::Reply(invalid),
        Err(transport::StartError::WritesLocked)=>{
            io.state.locked_outcome(io.session.closed())
        },
        Err(_)=>Outcome::Unavailable,
    };
    io.state.note(result)
}
fn route(boot:&Boot,w:&mut Windows,role:u8,m:&[u8;128])->bool {
    let slot=match role{4=>FILES,5=>SETTINGS,6=>TERMINAL,_=>fail()};
    match send(boot.caps[slot],m) {
        Ok(())=>true,
        Err(-3) if role==6=>{w.terminal_stale();false},
        _=>fail(),
    }
}
pub fn shell(boot:&Boot)->! {
    let mut w=Windows::new();
    deliver(boot.caps[COMPOSITOR],&w.wire());
    loop {
        let e=receive(boot.caps[SELF_RECV]);
        if !matches!(e.sender,2|5)||e.generation!=boot.peers[e.sender as usize]{continue;}
        if e.sender==5&&e.bytes[0]==0x12&&e.bytes[1]<=1&&e.bytes[2..].iter().all(|&b|b==0) {
            w.light=e.bytes[1]!=0;deliver(boot.caps[COMPOSITOR],&w.wire());continue;
        }
        if e.sender!=2{continue;}
        let Some(key)=key_decode(&e.bytes) else{continue;};
        match key {
            0x81..=0x83=>{
                let role=key-0x81+4;
                let mut activate=[0;128];activate[0]=2;
                if route(boot,&mut w,role,&activate){w.show(role);}
                deliver(boot.caps[COMPOSITOR],&w.wire());
            }
            27=>{if let Some(role)=w.focus(){w.hide(role);deliver(boot.caps[COMPOSITOR],&w.wire());}}
            _=>{if let Some(role)=w.focus(){
                if !route(boot,&mut w,role,&e.bytes){deliver(boot.caps[COMPOSITOR],&w.wire());}
            }}
        }
    }
}
fn files_view(boot:&Boot,io:&mut Io,selected:&mut usize)->View{
    let mut view=View::EMPTY;view.line(0,b"DATA VAULT - UP/DOWN SELECT, F1 REFRESH");
    let reply=call(boot,io,fs::LIST,b"",b"");
    let names=match reply{
        Outcome::Reply(r)=>Names::decode(&r),
        _=>None,
    };
    let Some(names)=names else{
        view.line(1,problem(reply));view.line(5,io.label());return view;
    };
    if *selected>=names.count{*selected=0;}
    view.lines[1]=names.display();
    if names.count>0{
        let name=names.name(*selected);let mut label=Text::new(b"SELECTED: ");
        label.append(name);view.lines[2]=label;
        let reply=call(boot,io,fs::READ,name,b"");
        match reply{
            Outcome::Reply(r) if r[0]==fs::OK&&r[2]<=64=>{
                let n=r[2] as usize;view.line(3,&r[16..16+n.min(48)]);
                view.line(4,&r[16+n.min(48)..16+n]);
            },
            _=>view.line(3,problem(reply)),
        }
    }else{view.line(2,b"NO FILES");}
    view.line(5,io.label());view
}
pub fn files(boot:&Boot)->! {
    let mut pending=Io::new(boot);let mut selected=0;let mut version=0;
    let view=files_view(boot,&mut pending,&mut selected);publish(boot,&mut version,&view);
    loop {
        let m=event(boot,&mut pending);
        let key=key_decode(&m);
        if key==Some(0x84){selected=selected.saturating_sub(1);}
        if key==Some(0x85){selected=(selected+1).min(3);}
        if activated(&m)||matches!(key,Some(0x84|0x85)){
            let view=files_view(boot,&mut pending,&mut selected);publish(boot,&mut version,&view);
        }
    }
}
pub fn settings(boot:&Boot)->! {
    let mut light=false;let mut version=0;
    let mut view=View::EMPTY;view.line(0,b"APPEARANCE");view.line(1,b"DARK");
    view.line(2,b"SPACE TO CHANGE THEME");view.line(3,b"SESSION ONLY");publish(boot,&mut version,&view);
    loop {
        let m=shell_event(boot);
        if key_decode(&m)==Some(b' ') {
            light=!light;view.line(1,if light{b"LIGHT"}else{b"DARK"});
            publish(boot,&mut version,&view);
            let mut theme=[0;128];theme[0]=0x12;theme[1]=light as u8;
            deliver(boot.caps[SHELL],&theme);
        } else if activated(&m){publish(boot,&mut version,&view);}
    }
}
pub fn terminal(boot:&Boot)->! {
    let mut editor=Editor::new();let mut version=0;let mut pending=Io::new(boot);
    let mut view=View::EMPTY;view.line(0,b"RAR TERMINAL");view.line(1,b"HELP LIST READ WRITE CRASH");
    view.line(2,b"CREATE + WRITE ARE SEPARATE COMMITS");
    editor.prompt(&mut view);publish(boot,&mut version,&view);
    loop {
        let m=event(boot,&mut pending);
        if activated(&m){publish(boot,&mut version,&view);continue;}
        let Some(key)=key_decode(&m) else{continue;};
        match editor.key(key) {
            Edit::Ignored|Edit::Full=>continue,
            Edit::Changed=>editor.prompt(&mut view),
            Edit::Submit=>{
                view=View::EMPTY;view.line(0,b"RAR TERMINAL");
                match command(&editor.bytes[..editor.len]) {
                    Command::Help=>{view.line(1,b"HELP LIST READ WRITE CRASH");view.line(2,b"CREATE + WRITE MAY LEAVE EMPTY FILE");}
                    Command::List=>{
                        let reply=call(boot,&mut pending,fs::LIST,b"",b"");
                        if let Outcome::Reply(r)=reply{
                            if let Some(names)=Names::decode(&r){view.lines[1]=names.display();}
                            else{view.line(1,problem(reply));}
                        }else{view.line(1,problem(reply));}
                    }
                    Command::Read(name)=>{
                        let reply=call(boot,&mut pending,fs::READ,name,b"");
                        match reply{
                            Outcome::Reply(r) if r[0]==fs::OK&&r[2]<=64=>{
                                let n=r[2] as usize;view.line(1,name);
                                view.line(2,&r[16..16+n.min(48)]);
                                view.line(5,&r[16+n.min(48)..16+n]);
                            },
                            _=>view.line(1,problem(reply)),
                        }
                    }
                    Command::Write(name,data)=>{
                        let created=call(boot,&mut pending,fs::CREATE,name,b"");
                        match created{
                            _ if file_ui::can_write_after_create(created)=>{
                                let reply=call(boot,&mut pending,fs::WRITE,name,data);
                                match reply{
                                    _ if file_ui::saved(reply)=>{
                                        let mut label=Text::new(b"SAVED ");label.append(name);
                                        view.lines[1]=label;view.line(2,data);
                                    },
                                    _=>view.line(1,problem(reply)),
                                }
                            },
                            _=>view.line(1,problem(created)),
                        }
                    }
                    Command::Crash=>{
                        // Deliberate userspace invalid instruction; the kernel
                        // revokes this process. No application announces success.
                        unsafe{core::arch::asm!("ud2",options(noreturn));}
                    }
                    Command::Invalid=>view.line(1,b"UNKNOWN COMMAND"),
                }
                if view.lines[5].len==0{view.line(5,pending.label());}
                editor.clear();editor.prompt(&mut view);
            }
        }
        publish(boot,&mut version,&view);
    }
}
