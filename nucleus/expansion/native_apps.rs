//! Explicit independent-app kernel adapter; sole CPU/IF=0, cloud only.
//! Fixed immutable public-lab banks, no userspace payload pointers or I/O.
use super::*;
use model::applications::{AppImage,APP_SLOTS};
#[path="../../core/expansion/app_control.rs"] mod control;
#[cfg(not(rar_modern_compile_only))]
fn banks()->[&'static[u8];2]{
    [include_bytes!("/tmp/rar-notes.app"),include_bytes!("/tmp/rar-counter.app")]
}
#[cfg(rar_modern_compile_only)]
fn banks()->[&'static[u8];2]{[&[],&[]]}
fn verified(index:usize)->Result<crate::application_package::app_manifest::VerifiedApp<'static>,Error>{
    let bytes=*banks().get(index).ok_or(Error::Invalid)?;
    if bytes.len()<1024||bytes.len()>131584{return Err(Error::Invalid);}
    let v=crate::application_package::app_manifest::verify(&bytes[..512],&bytes[512..],1,
        if index==0{3}else{1}).map_err(|_|Error::Denied)?;
    let manifest=v.manifest();
    if manifest.application_id()!=if index==0{control::NOTES}else{control::COUNTER}||
        manifest.generation()!=1||manifest.rights()!=if index==0{3}else{1}{return Err(Error::Denied);}
    Ok(v)
}
fn identity(v:&crate::application_package::app_manifest::VerifiedApp<'_>)->Result<AppImage,Error>{
    let m=v.manifest();
    Ok(AppImage{application:m.application_id(),owner:v.document_owner(),
        digest:crate::sha256::sha256(v.payload()).map_err(|_|Error::Invalid)?,
        generation:m.generation(),rights:m.rights()})
}
impl Runtime {
    pub(super) fn application_syscall(&mut self,f:&arch::Trap)->Result<u64,Error>{
        let index=usize::try_from(f.rdx).map_err(|_|Error::Invalid)?;
        match f.rsi {
            0=>{
                if f.rdx!=0||f.r10!=0{return Err(Error::Invalid);}
                self.policy.as_ref().ok_or(Error::Denied)?.stage_view(self.current,f.rdi)?;
                if self.app_catalog.iter().all(Option::is_some){return Ok(0);}
                let images=[identity(&verified(0)?)?,identity(&verified(1)?)?];
                self.policy.as_mut().unwrap().enable_application_controls(self.current,f.rdi)?;
                self.app_catalog=images.map(Some);Ok(0)
            },
            1=>{
                if f.r10!=0{return Err(Error::Invalid);}
                self.policy.as_ref().ok_or(Error::Denied)?.application_channel(self.current,f.rdi,index)
            },
            2=>{
                if index>=2{return Err(Error::Invalid);}
                // Query authenticates actual caller and complete output before copy.
                let policy=self.policy.as_ref().ok_or(Error::Denied)?;
                let record=policy.application_record(self.current,f.rdi,index)?;
                let image=self.app_catalog[index].ok_or(Error::Stale)?;
                self.buffer(f.r10,128,true)?;
                let mut out=control::Record{index,incarnation:record.map_or(0,|r|r.endpoint.incarnation),
                    generation:image.generation,rights:image.rights,state:record.map_or(0,|r|
                        if self.processes[r.endpoint.slot as usize].held{1}else{2}),
                    application:image.application,owner:image.owner,digest:image.digest};
                if !matches!(self.current,1|8){out.owner=[0;32];}
                let raw=out.encode().map_err(|_|Error::Invalid)?;
                // SAFETY: exact owned output checked, sole CPU/IF=0, no yield.
                unsafe{ptr::copy_nonoverlapping(raw.as_ptr(),f.r10 as *mut u8,128);}Ok(0)
            },
            3=>{
                if index>=2||f.r10!=0{return Err(Error::Invalid);}
                self.policy.as_ref().ok_or(Error::Denied)?.stage_reject(self.current,f.rdi)?;
                if self.handover.is_some()||self.desktop.is_some()||self.stage_view{return Err(Error::Busy);}
                let v=verified(index)?;let image=identity(&v)?;
                if self.app_catalog[index]!=Some(image){return Err(Error::Stale);}
                let plan=self.policy.as_ref().unwrap().prepare_application(self.current,f.rdi,index,image)?;
                let app_record=plan.record();
                let handoff=plan.bootstrap(v.layout().entry)?;
                let slot=APP_SLOTS[index];
                if let Err(e)=self.construct_context(v.payload(),slot,app_record.endpoint.incarnation,
                    (v.manifest().stack_bytes()/4096)as u64,loader::NativeBoot::Application(handoff)){
                    self.synchronize_revocations();return Err(e);
                }
                self.processes[slot].held=true;
                if let Err(e)=self.policy.as_mut().unwrap().publish_application(self.current,f.rdi,plan){
                    self.synchronize_revocations();return Err(e);
                }
                record("RAR-EXPANSION:APP-HELD");Ok(0)
            },
            4=>{
                if index>=2||f.r10==0{return Err(Error::Invalid);}
                self.policy.as_ref().ok_or(Error::Denied)?.stage_view(self.current,f.rdi)?;
                let e=self.policy.as_ref().unwrap().application_binding(10+index)?.ok_or(Error::Stale)?;
                if e.incarnation!=f.r10{return Err(Error::Stale);}
                let p=&mut self.processes[e.slot as usize];
                if !p.held||p.state!=State::Blocked||p.memory!=retirement::Memory::Live||
                    p.generation!=e.incarnation{return Err(Error::Stale);}
                p.held=false;p.state=State::Runnable;record("RAR-EXPANSION:APP-STARTED");Ok(0)
            },
            5=>{
                if index>=2||f.r10==0{return Err(Error::Invalid);}
                self.policy.as_mut().ok_or(Error::Denied)?.close_application(self.current,f.rdi,index,f.r10)?;
                self.synchronize_revocations();self.retire_pending();
                record("RAR-EXPANSION:APP-CLOSED");Ok(0)
            },
            7=>{
                if index>=2{return Err(Error::Invalid);}
                self.buffer(f.r10,136,false)?;
                let mut request=[0u8;136];
                // SAFETY: complete caller span checked, copy before authority
                // use, sole CPU/IF=0. No query/SEND scheduling race is possible.
                unsafe{ptr::copy_nonoverlapping(f.r10 as *const u8,request.as_mut_ptr(),136);}
                let inc=u64::from_le_bytes(request[..8].try_into().unwrap());
                let policy=self.policy.as_ref().ok_or(Error::Denied)?;
                let current=policy.application_record(self.current,f.rdi,index)?.ok_or(Error::Stale)?;
                if inc==0||current.endpoint.incarnation!=inc{return Err(Error::Stale);}
                let cap=policy.application_channel(self.current,f.rdi,10+index)?;
                self.policy.as_mut().unwrap().send(self.current,cap,&request[8..])?;
                let target=current.endpoint.slot as usize;
                if self.processes[target].state==State::Blocked&&!self.processes[target].held{
                    self.processes[target].state=State::Runnable;
                }
                Ok(0)
            },
            6=>{
                if !matches!(index,0|1|3|8|9)||f.r10!=0{return Err(Error::Invalid);}
                let policy=self.policy.as_ref().ok_or(Error::Denied)?;
                policy.application_record(self.current,f.rdi,0)?;
                Ok(policy.binding(index)?.ok_or(Error::Stale)?.incarnation)
            },
            _=>Err(Error::Invalid),
        }
    }
}
