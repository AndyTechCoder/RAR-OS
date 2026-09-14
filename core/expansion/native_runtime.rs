//! Trusted native service helpers. No untrusted app receives these controls.
use crate::{abi::*,app_control as control,syscall};
pub fn query(boot:&Boot,index:usize)->Option<control::Record>{
    let mut b=[0;128];
    if syscall(control::SYSCALL,boot.caps[SELF_RECV],2,index as u64,b.as_mut_ptr()as u64)!=0{return None;}
    let r=control::Record::decode(&b).ok()?;if r.index!=index{return None;}Some(r)
}
pub fn channel(boot:&Boot,destination:usize)->Option<u64>{
    let r=syscall(control::SYSCALL,boot.caps[SELF_RECV],1,destination as u64,0);
    if r<=0{None}else{Some(r as u64)}
}
pub fn peer(boot:&Boot,role:usize)->Option<u64>{
    let r=syscall(control::SYSCALL,boot.caps[SELF_RECV],6,role as u64,0);
    if r<=0{None}else{Some(r as u64)}
}
pub fn send(boot:&Boot,destination:usize,m:&[u8;128])->bool{
    channel(boot,destination).is_some_and(|h|crate::send(h,m).is_ok())
}
pub fn manager_call(boot:&Boot,op:u64,index:usize,incarnation:u64)->bool{
    syscall(control::SYSCALL,boot.caps[MANAGER],op,index as u64,incarnation)==0
}
pub fn now()->Option<u64>{let n=syscall(TICKS,0,0,0,0);if n<0{None}else{Some(n as u64)}}
pub fn wait_catalog(boot:&Boot)->control::Record{
    let start=now().unwrap_or_else(||crate::fail());
    for _ in 0..65536{
        if let Some(r)=query(boot,0){return r;}
        if now().is_none_or(|n|n<start||n-start>=4096){crate::fail();}
        crate::yield_now();
    }crate::fail()
}
pub struct Manager { pending:[Option<(u64,u64)>;2] }
impl Manager {
    pub fn new(boot:&Boot)->Self{
        if !manager_call(boot,0,0,0){crate::fail();}
        Self{pending:[None;2]}
    }
    pub fn tick(&mut self,boot:&Boot){
        for index in 0..2{
            if let Some((inc,deadline))=self.pending[index]{
                if query(boot,index).is_none_or(|r|r.incarnation!=inc)||now().is_none_or(|n|n>=deadline){
                    let _=manager_call(boot,5,index,inc);self.pending[index]=None;
                }
            }
        }
    }
    pub fn message(&mut self,boot:&Boot,e:&Envelope)->bool{
        let Ok((op,index,inc))=control::parse(&e.bytes)else{return false;};
        if e.length!=128{return true;}
        if e.sender==0&&Some(e.generation)==peer(boot,0){
            match op {
                1=>{
                    if self.pending[index].is_some(){return true;}
                    let Some(r)=query(boot,index)else{return true;};
                    if r.state!=0{return true;}
                    if !manager_call(boot,3,index,0){return true;}
                    let Some(r)=query(boot,index)else{return true;};
                    if r.state!=1{return true;}
                    if index==1{
                        if !manager_call(boot,4,index,r.incarnation){let _=manager_call(boot,5,index,r.incarnation);}
                    }else{
                        let Some(deadline)=now().and_then(|n|n.checked_add(4096))else{
                            let _=manager_call(boot,5,index,r.incarnation);return true;
                        };
                        let m=control::control(4,index,r.incarnation).unwrap();
                        if send(boot,1,&m){self.pending[index]=Some((r.incarnation,deadline));}
                        else{let _=manager_call(boot,5,index,r.incarnation);}
                    }
                },
                2=>{
                    let _=manager_call(boot,5,index,inc);
                    if self.pending[index].is_some_and(|p|p.0==inc){self.pending[index]=None;}
                },
                _=>{},
            }
        }else if e.sender==1&&Some(e.generation)==peer(boot,1)&&matches!(op,5|6)&&index==0{
            if self.pending[index].is_some_and(|p|p.0==inc){
                if op!=5||!manager_call(boot,4,index,inc){let _=manager_call(boot,5,index,inc);}
                self.pending[index]=None;
            }
        }
        true
    }
}

pub fn send_app(boot:&Boot,index:usize,incarnation:u64,frame:&[u8;128])->bool{
    let mut request=[0u8;136];request[..8].copy_from_slice(&incarnation.to_le_bytes());
    request[8..].copy_from_slice(frame);
    syscall(control::SYSCALL,boot.caps[SELF_RECV],7,index as u64,request.as_ptr()as u64)==0
}
