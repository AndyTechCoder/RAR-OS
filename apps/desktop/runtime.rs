//! Desktop application policy. Device access stays in separate services.
use crate::{abi::*,services,receive,deliver,send,publish,check,fail};
use services::apps::*;
use services::storage as fs;
fn activated(m:&[u8;128])->bool {m[0]==2&&m[1..].iter().all(|&b|b==0)}
fn event(boot:&Boot,pending:&mut Pending)->[u8;128] {
    if let Some(m)=pending.pop(){return m;}
    loop {let e=receive(boot.caps[SELF_RECV]);if e.sender==0&&e.generation==1{return e.bytes;}}
}
fn call(boot:&Boot,pending:&mut Pending,op:u8,name:&[u8],data:&[u8])->[u8;128] {
    let request=fs::request(op,name,data).unwrap_or_else(||fail());
    deliver(boot.caps[STORAGE],&request);
    loop {
        let e=receive(boot.caps[SELF_RECV]);
        if e.generation!=1{continue;}
        if e.sender==1{return e.bytes;}
        if e.sender==0&&(activated(&e.bytes)||key_decode(&e.bytes).is_some()) {
            check(pending.push(e.bytes));
        }
    }
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
        if e.generation!=1{continue;}
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
fn files_view(boot:&Boot,pending:&mut Pending,selected:&mut usize)->View {
    let reply=call(boot,pending,fs::LIST,b"",b"");
    let names=Names::decode(&reply).unwrap_or_else(||fail());
    if *selected>=names.count{*selected=0;}
    let mut view=View::EMPTY;
    view.line(0,b"TEMPORARY WORKSPACE");view.lines[1]=names.display();
    if names.count>0 {
        let name=names.name(*selected);let mut label=Text::new(b"SELECTED: ");label.append(name);view.lines[2]=label;
        let data=call(boot,pending,fs::READ,name,b"");
        if data[0]==fs::OK&&data[2]<=64 {view.line(3,&data[16..16+data[2] as usize]);}
        else {view.line(3,b"READ FAILED");}
    } else {view.line(2,b"NO FILES");}
    view.line(4,b"UP/DOWN SELECT  F1 REFRESH");view.line(5,b"RAM ONLY - LOST ON STOP");view
}
pub fn files(boot:&Boot)->! {
    let mut pending=Pending::new();let mut selected=0;let mut version=0;
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
    let mut light=false;let mut version=0;let mut pending=Pending::new();
    let mut view=View::EMPTY;view.line(0,b"APPEARANCE");view.line(1,b"DARK");
    view.line(2,b"SPACE TO CHANGE THEME");view.line(3,b"SESSION ONLY");publish(boot,&mut version,&view);
    loop {
        let m=event(boot,&mut pending);
        if key_decode(&m)==Some(b' ') {
            light=!light;view.line(1,if light{b"LIGHT"}else{b"DARK"});
            publish(boot,&mut version,&view);
            let mut theme=[0;128];theme[0]=0x12;theme[1]=light as u8;
            deliver(boot.caps[SHELL],&theme);
        } else if activated(&m){publish(boot,&mut version,&view);}
    }
}
pub fn terminal(boot:&Boot)->! {
    let mut editor=Editor::new();let mut version=0;let mut pending=Pending::new();
    let mut view=View::EMPTY;view.line(0,b"RAR TERMINAL");view.line(1,b"HELP LIST READ WRITE CRASH");
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
                    Command::Help=>{view.line(1,b"HELP LIST READ WRITE CRASH");view.line(2,b"WRITE NAME TEXT - TEMPORARY FILES");}
                    Command::List=>{
                        let reply=call(boot,&mut pending,fs::LIST,b"",b"");
                        let names=Names::decode(&reply).unwrap_or_else(||fail());view.lines[1]=names.display();
                    }
                    Command::Read(name)=>{
                        let reply=call(boot,&mut pending,fs::READ,name,b"");
                        if reply[0]==fs::OK&&reply[2]<=64 {
                            view.line(1,name);view.line(2,&reply[16..16+reply[2] as usize]);
                        }else{view.line(1,b"READ FAILED");}
                    }
                    Command::Write(name,data)=>{
                        let created=call(boot,&mut pending,fs::CREATE,name,b"");
                        if created[0]==fs::OK||created[0]==fs::EXISTS {
                            let reply=call(boot,&mut pending,fs::WRITE,name,data);
                            if reply[0]==fs::OK {
                                let mut label=Text::new(b"SAVED ");label.append(name);view.lines[1]=label;view.line(2,data);
                            }else{view.line(1,b"WRITE FAILED");}
                        }else{view.line(1,b"CREATE FAILED");}
                    }
                    Command::Crash=>{
                        // Deliberate userspace invalid instruction; the kernel
                        // revokes this process. No application announces success.
                        unsafe{core::arch::asm!("ud2",options(noreturn));}
                    }
                    Command::Invalid=>view.line(1,b"UNKNOWN COMMAND"),
                }
                editor.clear();editor.prompt(&mut view);
            }
        }
        publish(boot,&mut version,&view);
    }
}
