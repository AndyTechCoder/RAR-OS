//! Modern-only logical IPC and replacement mechanism model.
//! Kernel-owned state; callers/handles must come from the real trap context.
//! No signature policy, page mapping, process execution or disk I/O here.
#![forbid(unsafe_code)]

pub const TASKS:usize=16;
pub const PRINCIPALS:usize=10;
pub const CAP_SLOTS:usize=12;
pub const MESSAGE_BYTES:usize=128;
pub const QUEUE_DEPTH:usize=4;
pub const SEND:u8=1;
pub const RECEIVE:u8=2;
pub const HEALTH:u8=4;
pub const MANAGE:u8=8;
pub const SELF_CAP:usize=0;
pub const SHELL_CAP:usize=1;
pub const COMPOSITOR_CAP:usize=2;
pub const HEALTH_CAP:usize=9;
pub const MANAGER_CAP:usize=10;

#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error {Invalid,Denied,Stale,Full,Empty,Exhausted,Busy}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Endpoint {pub slot:u8,pub incarnation:u64}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Object {
    None, NamedSend {principal:u8}, Receive(Endpoint),
    TrialHealth {endpoint:Endpoint,token:u64}, Manager,
}
#[derive(Clone,Copy)]
struct Cap {generation:u32,rights:u8,object:Object,retired:bool}
impl Cap {const EMPTY:Self=Self {generation:1,rights:0,object:Object::None,retired:false};}
#[derive(Clone,Copy)]
pub struct Caps {slots:[Cap;CAP_SLOTS]}
impl Caps {
    pub const fn new()->Self {Self {slots:[Cap::EMPTY;CAP_SLOTS]}}
    pub fn grant(&mut self,index:usize,object:Object,rights:u8)->Result<u64,Error>{
        let s=self.slots.get_mut(index).ok_or(Error::Invalid)?;
        let allowed=match object {
            Object::NamedSend {principal} if (principal as usize)<PRINCIPALS=>SEND,
            Object::Receive(e) if (e.slot as usize)<TASKS&&e.incarnation!=0=>RECEIVE,
            Object::TrialHealth {endpoint:e,token} if (e.slot as usize)<TASKS&&e.incarnation!=0&&token!=0=>HEALTH,
            Object::Manager=>MANAGE,
            _=>return Err(Error::Invalid),
        };
        if rights!=allowed||s.retired||s.object!=Object::None {return Err(Error::Denied);}
        s.object=object;s.rights=rights;
        Ok((s.generation as u64)<<32 | (index as u64+1))
    }
    pub fn resolve(&self,handle:u64,right:u8)->Result<Object,Error>{
        let i=(handle as u32).checked_sub(1).ok_or(Error::Invalid)? as usize;
        let s=self.slots.get(i).ok_or(Error::Invalid)?;
        if s.retired||s.object==Object::None||s.generation!=(handle>>32)as u32{return Err(Error::Stale);}
        if right==0||right&s.rights!=right{return Err(Error::Denied);}
        Ok(s.object)
    }
    pub fn revoke(&mut self,index:usize)->Result<(),Error>{
        let s=self.slots.get_mut(index).ok_or(Error::Invalid)?;
        s.object=Object::None;s.rights=0;
        if let Some(n)=s.generation.checked_add(1){s.generation=n;}else{s.retired=true;}
        Ok(())
    }
    fn revoke_all(&mut self){for i in 0..CAP_SLOTS {let _=self.revoke(i);}}
    pub fn handle(&self,index:usize)->Result<u64,Error>{
        let s=self.slots.get(index).ok_or(Error::Invalid)?;
        if s.retired||s.object==Object::None{return Err(Error::Stale);}
        Ok((s.generation as u64)<<32|(index as u64+1))
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Message {pub principal:u8,pub incarnation:u64,pub length:u8,pub bytes:[u8;MESSAGE_BYTES]}
impl Message {
    const EMPTY:Self=Self {principal:0,incarnation:0,length:0,bytes:[0;MESSAGE_BYTES]};
    fn stamp(principal:u8,incarnation:u64,bytes:&[u8])->Result<Self,Error>{
        if principal as usize>=PRINCIPALS||incarnation==0||bytes.is_empty()||bytes.len()>MESSAGE_BYTES {
            return Err(Error::Invalid);
        }
        let mut m=Self {principal,incarnation,length:bytes.len()as u8,..Self::EMPTY};
        m.bytes[..bytes.len()].copy_from_slice(bytes);Ok(m)
    }
}
#[derive(Clone,Copy)]
struct Queue {messages:[Message;QUEUE_DEPTH],length:usize}
impl Queue {
    const fn new()->Self {Self {messages:[Message::EMPTY;QUEUE_DEPTH],length:0}}
    fn push(&mut self,m:Message)->Result<(),Error>{
        if self.length==QUEUE_DEPTH||
            self.messages[..self.length].iter().filter(|x|x.principal==m.principal&&x.incarnation==m.incarnation).count()>=2 {
            return Err(Error::Full);
        }
        self.messages[self.length]=m;self.length+=1;Ok(())
    }
    fn pop(&mut self)->Result<Message,Error>{
        if self.length==0{return Err(Error::Empty);}
        let m=self.messages[0];self.messages.copy_within(1..self.length,0);
        self.length-=1;self.messages[self.length]=Message::EMPTY;Ok(m)
    }
    fn purge(&mut self,principal:u8,incarnation:u64){
        let mut n=0;
        for i in 0..self.length{
            let m=self.messages[i];
            if m.principal!=principal||m.incarnation!=incarnation{self.messages[n]=m;n+=1;}
        }
        self.messages[n..].fill(Message::EMPTY);self.length=n;
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum State {Vacant,Trial,Healthy,Active}
#[derive(Clone,Copy)]
struct Process {state:State,principal:Option<u8>,incarnation:u64,caps:Caps,queue:Queue}
impl Process {const EMPTY:Self=Self {state:State::Vacant,principal:None,incarnation:0,caps:Caps::new(),queue:Queue::new()};}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Trial {token:u64,endpoint:Endpoint,previous:Option<Endpoint>,budget:u32,image_seal:u64}
impl Trial {
    pub fn token(&self)->u64 {self.token}
    pub fn endpoint(&self)->Endpoint {self.endpoint}
    pub fn image_seal(&self)->u64 {self.image_seal}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Cutover {pub previous:Option<Endpoint>,pub current:Endpoint}
pub struct Runtime {
    processes:[Process;TASKS],bindings:[Option<Endpoint>;PRINCIPALS],
    clock:u64,next_token:u64,trial:Option<Trial>,
}
impl Runtime {
    /// Initial model graph, before a real kernel connects process construction.
    pub fn new()->Self{
        let mut r=Self {processes:[Process::EMPTY;TASKS],bindings:[None;PRINCIPALS],
            clock:1,next_token:1,trial:None};
        for i in [0,1,2,3,4,5,6,8,9]{
            let e=Endpoint {slot:i as u8,incarnation:1};
            r.processes[i].state=State::Active;r.processes[i].principal=Some(i as u8);r.processes[i].incarnation=1;
            r.bindings[i]=Some(e);
            r.processes[i].caps.grant(SELF_CAP,Object::Receive(e),RECEIVE).unwrap();
        }
        r.processes[8].caps.grant(MANAGER_CAP,Object::Manager,MANAGE).unwrap();
        r.processes[5].caps.grant(SHELL_CAP,Object::NamedSend {principal:0},SEND).unwrap();
        r.processes[5].caps.grant(COMPOSITOR_CAP,Object::NamedSend {principal:3},SEND).unwrap();
        // Shell holds a named endpoint; it follows authorized principal5 rebinding.
        r.processes[0].caps.grant(5,Object::NamedSend {principal:5},SEND).unwrap();
        r
    }
    pub fn binding(&self,principal:usize)->Result<Option<Endpoint>,Error>{
        self.bindings.get(principal).copied().ok_or(Error::Invalid)
    }
    pub fn state(&self,slot:usize)->Result<State,Error>{
        self.processes.get(slot).map(|p|p.state).ok_or(Error::Invalid)
    }
    pub fn handle(&self,slot:usize,index:usize)->Result<u64,Error>{
        self.processes.get(slot).ok_or(Error::Invalid)?.caps.handle(index)
    }
    fn manager(&self,caller:usize,handle:u64)->Result<(),Error>{
        let p=self.processes.get(caller).ok_or(Error::Invalid)?;
        if p.state!=State::Active||p.principal!=Some(8)||
            p.caps.resolve(handle,MANAGE)?!=Object::Manager{return Err(Error::Denied);}
        Ok(())
    }
    fn endpoint_alive(&self,e:Endpoint)->bool{
        self.processes.get(e.slot as usize).is_some_and(|p|p.state==State::Active&&p.incarnation==e.incarnation)
    }
    /// image_seal must name an actual kernel-owned immutable staging object in
    /// the future runtime. A nonzero integer here is MODEL INPUT, not seal proof.
    pub fn begin_trial(&mut self,caller:usize,handle:u64,image_seal:u64,budget:u32)->Result<Trial,Error>{
        self.manager(caller,handle)?;
        if self.trial.is_some(){return Err(Error::Busy);}
        if image_seal==0||!(1..=100).contains(&budget){return Err(Error::Invalid);}
        let previous=self.bindings[5];
        if previous.is_some_and(|e|!self.endpoint_alive(e)){return Err(Error::Stale);}
        let index=[5usize,7].into_iter().find(|&i|self.processes[i].state==State::Vacant).ok_or(Error::Busy)?;
        let incarnation=self.clock.checked_add(1).ok_or(Error::Exhausted)?;
        let next_token=self.next_token.checked_add(1).ok_or(Error::Exhausted)?;
        let endpoint=Endpoint {slot:index as u8,incarnation};
        let token=self.next_token;
        let mut caps=self.processes[index].caps;
        caps.grant(HEALTH_CAP,Object::TrialHealth {endpoint,token},HEALTH)?;
        let trial=Trial {token,endpoint,previous,budget,image_seal};
        // No fallible work after publishing model state.
        self.clock=incarnation;self.next_token=next_token;
        self.processes[index]=Process {state:State::Trial,principal:None,incarnation,caps,queue:Queue::new()};
        self.trial=Some(trial);Ok(trial)
    }
    pub fn ready(&mut self,caller:usize,handle:u64,token:u64)->Result<(),Error>{
        let trial=self.trial.ok_or(Error::Stale)?;
        let p=self.processes.get_mut(caller).ok_or(Error::Invalid)?;
        if caller!=trial.endpoint.slot as usize||p.state!=State::Trial||
            p.incarnation!=trial.endpoint.incarnation||token!=trial.token||
            p.caps.resolve(handle,HEALTH)?!=(Object::TrialHealth {endpoint:trial.endpoint,token}) {
            return Err(Error::Denied);
        }
        p.caps.revoke(HEALTH_CAP)?;
        p.state=State::Healthy;Ok(())
    }
    pub fn send(&mut self,caller:usize,handle:u64,bytes:&[u8])->Result<(),Error>{
        let p=self.processes.get(caller).ok_or(Error::Invalid)?;
        if p.state!=State::Active{return Err(Error::Denied);}
        let Object::NamedSend {principal}=p.caps.resolve(handle,SEND)? else{return Err(Error::Denied);};
        let sender=p.principal.ok_or(Error::Denied)?;
        let e=self.bindings[principal as usize].ok_or(Error::Stale)?;
        if !self.endpoint_alive(e){return Err(Error::Stale);}
        let m=Message::stamp(sender,p.incarnation,bytes)?;
        self.processes[e.slot as usize].queue.push(m)
    }
    pub fn receive(&mut self,caller:usize,handle:u64)->Result<Message,Error>{
        let p=self.processes.get_mut(caller).ok_or(Error::Invalid)?;
        if p.state!=State::Active{return Err(Error::Denied);}
        let expected=Object::Receive(Endpoint {slot:caller as u8,incarnation:p.incarnation});
        if p.caps.resolve(handle,RECEIVE)?!=expected{return Err(Error::Denied);}
        p.queue.pop()
    }
    fn destroy(&mut self,index:usize){
        if let Some(principal)=self.processes[index].principal{
            let incarnation=self.processes[index].incarnation;
            if self.bindings[principal as usize]==Some(Endpoint {slot:index as u8,incarnation}){
                self.bindings[principal as usize]=None;
            }
            for p in &mut self.processes{p.queue.purge(principal,incarnation);}
        }
        let p=&mut self.processes[index];
        p.state=State::Vacant;p.principal=None;p.queue=Queue::new();p.caps.revoke_all();
    }
    pub fn abort(&mut self,caller:usize,handle:u64,token:u64)->Result<(),Error>{
        self.manager(caller,handle)?;
        let t=self.trial.ok_or(Error::Stale)?;
        if t.token!=token{return Err(Error::Stale);}
        self.destroy(t.endpoint.slot as usize);self.trial=None;Ok(())
    }
    /// Real caller must hold IF=0 and finish process sealing/construction first.
    /// This atomically switches model bindings/caps/queues, NOT durable storage.
    pub fn cutover(&mut self,caller:usize,handle:u64,token:u64)->Result<Cutover,Error>{
        self.manager(caller,handle)?;
        let t=self.trial.ok_or(Error::Stale)?;
        let index=t.endpoint.slot as usize;
        if token!=t.token||self.bindings[5]!=t.previous||self.processes[index].state!=State::Healthy||
            self.processes[index].incarnation!=t.endpoint.incarnation{return Err(Error::Stale);}
        if t.previous.is_some_and(|e|!self.endpoint_alive(e)){return Err(Error::Stale);}
        let mut caps=self.processes[index].caps;
        caps.grant(SELF_CAP,Object::Receive(t.endpoint),RECEIVE)?;
        caps.grant(SHELL_CAP,Object::NamedSend {principal:0},SEND)?;
        caps.grant(COMPOSITOR_CAP,Object::NamedSend {principal:3},SEND)?;
        // All grant checks finished. Mutation below is bounded and infallible.
        if let Some(old)=t.previous{self.destroy(old.slot as usize);}
        let p=&mut self.processes[index];
        p.queue=Queue::new();p.caps=caps;p.principal=Some(5);p.state=State::Active;
        self.bindings[5]=Some(t.endpoint);self.trial=None;
        Ok(Cutover {previous:t.previous,current:t.endpoint})
    }
    /// Timer/fault hooks are kernel-only; no syscall may choose an arbitrary slot.
    pub fn preempt(&mut self,slot:usize)->Result<bool,Error>{
        if slot>=TASKS{return Err(Error::Invalid);}
        let Some(mut t)=self.trial else{return Ok(false);};
        if slot!=t.endpoint.slot as usize||self.processes[slot].state!=State::Trial{return Ok(false);}
        t.budget-=1;
        if t.budget==0{self.destroy(slot);self.trial=None;Ok(true)}
        else{self.trial=Some(t);Ok(false)}
    }
    pub fn fault(&mut self,slot:usize)->Result<(),Error>{
        if slot>=TASKS{return Err(Error::Invalid);}
        if self.trial.is_some_and(|t|t.endpoint.slot as usize==slot){self.trial=None;}
        self.destroy(slot);Ok(())
    }
}
impl Default for Caps {fn default()->Self{Self::new()}}
impl Default for Runtime {fn default()->Self{Self::new()}}

#[cfg(test)]
mod tests {
    use super::*;
    fn manager(r:&Runtime)->u64{r.handle(8,MANAGER_CAP).unwrap()}
    fn healthy(r:&mut Runtime)->Trial{
        let t=r.begin_trial(8,manager(r),23,50).unwrap();
        r.ready(t.endpoint.slot as usize,r.handle(t.endpoint.slot as usize,HEALTH_CAP).unwrap(),t.token).unwrap();t
    }
    #[test] fn trial_has_no_production_authority_and_health_is_one_shot(){
        let mut r=Runtime::new();let m=manager(&r);
        let t=r.begin_trial(8,m,1,2).unwrap();let slot=t.endpoint.slot as usize;
        assert_eq!(slot,7);assert_eq!(r.binding(5).unwrap().unwrap().slot,5);
        for h in [0,1,r.handle(slot,HEALTH_CAP).unwrap()]{
            assert!(r.send(slot,h,b"x").is_err());assert!(r.receive(slot,h).is_err());
        }
        let h=r.handle(slot,HEALTH_CAP).unwrap();
        assert!(r.ready(5,h,t.token).is_err());assert!(r.ready(slot,h,t.token+1).is_err());
        r.ready(slot,h,t.token).unwrap();assert!(r.ready(slot,h,t.token).is_err());
        assert_eq!(r.state(slot),Ok(State::Healthy));assert!(!r.preempt(slot).unwrap());
    }
    #[test] fn only_manager_can_prepare_commit_or_abort(){
        let mut r=Runtime::new();let m=manager(&r);
        for caller in 0..TASKS {if caller!=8 {assert!(r.begin_trial(caller,m,1,1).is_err());}}
        for handle in [0,u64::MAX,1,m^(1<<32)]{assert!(r.begin_trial(8,handle,1,1).is_err());}
        for budget in [0,101,u32::MAX]{assert!(r.begin_trial(8,m,1,budget).is_err());}
        assert!(r.begin_trial(8,m,0,1).is_err());
        let t=r.begin_trial(8,m,1,10).unwrap();
        assert_eq!(r.begin_trial(8,m,1,10),Err(Error::Busy));
        assert!(r.cutover(8,m,t.token).is_err());
        assert!(r.abort(8,m,t.token+1).is_err());
        r.abort(8,m,t.token).unwrap();assert!(r.abort(8,m,t.token).is_err());
    }
    #[test] fn named_endpoints_follow_cutover_old_queues_and_handles_do_not(){
        let mut r=Runtime::new();let shell_send=r.handle(0,5).unwrap();
        let old_recv=r.handle(5,SELF_CAP).unwrap();let old_send=r.handle(5,SHELL_CAP).unwrap();
        r.send(0,shell_send,b"old request").unwrap();
        r.send(5,old_send,b"old response").unwrap();
        let t=healthy(&mut r);let c=r.cutover(8,manager(&r),t.token).unwrap();
        assert_eq!(c.previous,Some(Endpoint {slot:5,incarnation:1}));
        assert_eq!(c.current,t.endpoint);assert_eq!(r.state(5),Ok(State::Vacant));
        assert!(r.receive(5,old_recv).is_err());assert!(r.send(5,old_send,b"stale").is_err());
        assert_eq!(r.receive(0,r.handle(0,SELF_CAP).unwrap()),Err(Error::Empty));
        assert_eq!(r.receive(7,r.handle(7,SELF_CAP).unwrap()),Err(Error::Empty));
        r.send(0,shell_send,b"new request").unwrap();
        let msg=r.receive(7,r.handle(7,SELF_CAP).unwrap()).unwrap();
        assert_eq!(&msg.bytes[..msg.length as usize],b"new request");
        r.send(7,r.handle(7,SHELL_CAP).unwrap(),b"new response").unwrap();
        let msg=r.receive(0,r.handle(0,SELF_CAP).unwrap()).unwrap();
        assert_eq!((msg.principal,msg.incarnation),(5,2));
    }
    #[test] fn abort_timeout_and_candidate_fault_preserve_active_service(){
        for mode in 0..3{
            let mut r=Runtime::new();let t=r.begin_trial(8,manager(&r),1,1).unwrap();
            match mode{0=>r.abort(8,manager(&r),t.token).unwrap(),1=>assert!(r.preempt(7).unwrap()),_=>r.fault(7).unwrap()}
            assert_eq!(r.binding(5),Ok(Some(Endpoint {slot:5,incarnation:1})));
            assert_eq!(r.state(7),Ok(State::Vacant));
            assert!(r.cutover(8,manager(&r),t.token).is_err());
        }
    }
    #[test] fn post_cutover_failure_requires_fresh_incarnation_not_resurrection(){
        let mut r=Runtime::new();let t=healthy(&mut r);r.cutover(8,manager(&r),t.token).unwrap();
        let old_handle=r.handle(7,SHELL_CAP).unwrap();r.fault(7).unwrap();
        assert_eq!(r.binding(5),Ok(None));
        let recovery=healthy(&mut r);assert_eq!(recovery.endpoint.slot,5);assert_eq!(recovery.endpoint.incarnation,3);
        r.cutover(8,manager(&r),recovery.token).unwrap();
        assert!(r.send(7,old_handle,b"x").is_err());
        assert_eq!(r.binding(5),Ok(Some(recovery.endpoint)));
        for i in [0,1,2,3,4,6,8,9]{assert_eq!(r.binding(i).unwrap().unwrap().incarnation,1);}
    }
    #[test] fn exhaustion_and_stale_trial_never_wrap_or_partially_cutover(){
        let mut r=Runtime::new();r.clock=u64::MAX;
        assert_eq!(r.begin_trial(8,manager(&r),1,1),Err(Error::Exhausted));
        assert_eq!(r.state(7),Ok(State::Vacant));
        r.clock=1;r.next_token=u64::MAX;
        assert_eq!(r.begin_trial(8,manager(&r),1,1),Err(Error::Exhausted));
        r.next_token=1;let t=healthy(&mut r);r.fault(5).unwrap();
        assert!(r.cutover(8,manager(&r),t.token).is_err());
        r.abort(8,manager(&r),t.token).unwrap();
        assert_eq!(r.binding(5),Ok(None));
    }
    #[test] fn retired_caps_and_queue_limits_remain_fail_closed(){
        let mut caps=Caps::new();
        caps.slots[0].generation=u32::MAX;
        let h=caps.grant(0,Object::NamedSend {principal:5},SEND).unwrap();
        caps.revoke(0).unwrap();
        assert!(caps.resolve(h,SEND).is_err());assert!(caps.grant(0,Object::Manager,MANAGE).is_err());
        let mut r=Runtime::new();let h=r.handle(0,5).unwrap();
        r.send(0,h,b"1").unwrap();r.send(0,h,b"2").unwrap();
        assert_eq!(r.send(0,h,b"3"),Err(Error::Full));
    }
}
