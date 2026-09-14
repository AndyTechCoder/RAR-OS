//! Fixed independently-installed app lifecycle candidate. No syscall adapter,
//! signature verification, page mapping, persistence or scheduling here.
//! Callers are kernel-owned trap identities. A plan is not launch authority.
use super::*;
#[path="../../sdk/alpha/rust/wire.rs"]
pub mod sdk;

pub const APP_COUNT:usize=2;
pub const APP_FIRST:usize=10;
pub const APP_SLOTS:[usize;2]=[11,12];
const PEERS:[usize;5]=[0,1,3,8,9];
// Fixed additional channels. Never insert these into the old 368-byte Boot.
const CHANNELS:[(usize,usize,u8);8]=[
    (0,7,8),(8,2,0),(8,3,1),(1,9,8),(3,1,8),
    (3,9,10),(3,10,11),(1,7,10),
];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct AppImage{
    pub application:[u8;16],pub owner:[u8;32],pub digest:[u8;32],
    pub generation:u64,pub rights:u32,
}
impl AppImage{
    fn validate(self,index:usize)->Result<(),Error>{
        // This first integration deliberately grants no network or broker access.
        // Notes is the sole private-document owner; the C app is UI-only.
        if self.application==[0;16]||self.owner==[0;32]||self.digest==[0;32]||
            self.generation==0||self.rights!=if index==0{3}else{1}{
            return Err(Error::Invalid);
        }
        Ok(())
    }
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct AppRecord{pub endpoint:Endpoint,pub image:AppImage}
#[derive(Clone,Copy)]
pub(super) struct Applications{
    enabled:bool,controls:bool,epoch:u64,records:[Option<AppRecord>;APP_COUNT],
}
impl Applications{
    pub(super) const fn new()->Self{Self{enabled:false,controls:false,epoch:0,records:[None;APP_COUNT]}}
    pub(super) fn binding(&self,principal:usize)->Option<Endpoint>{
        principal.checked_sub(APP_FIRST).and_then(|i|self.records.get(i))
            .copied().flatten().map(|r|r.endpoint)
    }
    pub(super) fn clear(&mut self,principal:usize,endpoint:Endpoint){
        if let Some(record)=principal.checked_sub(APP_FIRST).and_then(|i|self.records.get_mut(i)){
            if record.is_some_and(|r|r.endpoint==endpoint){*record=None;}
        }
    }
}
/// Private preparation record. Native adapter must hold IF=0, construct a
/// non-runnable root from the verified immutable image, then publish immediately.
/// Do not hold this over IPC or disk I/O; do not copy unrelated Runtime state.
pub struct AppHandover{
    index:usize,record:AppRecord,caps:Caps,clock:u64,epoch:u64,peers:[Endpoint;5],
}
impl AppHandover{
    /// Exact separate app bootstrap from precomputed kernel authority. This is
    /// data only until the native root builder maps it read-only for this task.
    pub fn bootstrap(&self,entry:u64)->Result<[u8;256],Error>{
        let mut caps=[0u64;5];
        for(i,cap)in caps.iter_mut().enumerate(){
            if i==0||self.record.image.rights&(1<<(i-1))!=0{*cap=self.caps.handle(i)?;}
        }
        sdk::Boot{application:self.record.image.application,
            incarnation:self.record.endpoint.incarnation,principal:self.principal()as u32,
            rights:self.record.image.rights,entry,caps,
            peer_principals:[3,if self.record.image.rights&2!=0{1}else{0},0,0],
            peer_incarnations:self.peer_incarnations()}.encode().map_err(|_|Error::Invalid)
    }
    pub fn record(&self)->AppRecord{self.record}
    pub fn principal(&self)->usize{APP_FIRST+self.index}
    pub fn handle(&self,index:usize)->Result<u64,Error>{self.caps.handle(index)}
    pub fn peer_incarnations(&self)->[u64;4]{
        [self.peers[2].incarnation,
         if self.record.image.rights&2!=0{self.peers[1].incarnation}else{0},0,0]
    }
}
impl Runtime{
    /// Source candidate only: no existing native entry selects this composition.
    pub fn applications_bootstrap()->Self{
        let mut r=Self::expansion_bootstrap();r.apps.enabled=true;r
    }
    fn application_peers(&self)->Result<[Endpoint;5],Error>{
        if !self.apps.enabled||self.bootstrapping||self.recovery_required{return Err(Error::Denied);}
        let mut peers=[Endpoint{slot:0,incarnation:0};5];
        for(n,role)in PEERS.into_iter().enumerate(){
            let e=self.bindings[role].ok_or(Error::Stale)?;
            if !self.endpoint_alive(e)||self.processes[e.slot as usize].principal!=Some(role as u8){
                return Err(Error::Stale);
            }
            peers[n]=e;
        }
        Ok(peers)
    }
    /// Explicit Manager-only post-desktop channel publication. Precompute all
    /// grants before mutation. Repeated invocation does not mint fresh handles.
    pub fn enable_application_controls(&mut self,caller:usize,handle:u64)->Result<(),Error>{
        self.manager(caller,handle)?;self.application_peers()?;
        if self.apps.controls{return Ok(());}
        let epoch=self.apps.epoch.checked_add(1).ok_or(Error::Exhausted)?;
        let mut caps=[Caps::new();5];
        for(n,role)in PEERS.into_iter().enumerate(){caps[n]=self.processes[role].caps;}
        for(role,slot,destination)in CHANNELS{
            let n=PEERS.iter().position(|&r|r==role).ok_or(Error::Invalid)?;
            caps[n].grant(slot,Object::NamedSend{principal:destination},SEND)?;
        }
        for(n,role)in PEERS.into_iter().enumerate(){self.processes[role].caps=caps[n];}
        self.apps.controls=true;self.apps.epoch=epoch;Ok(())
    }
    fn application_query(&self,caller:usize,handle:u64)->Result<usize,Error>{
        self.application_peers()?;
        if !self.apps.controls{return Err(Error::Denied);}
        let p=self.processes.get(caller).ok_or(Error::Invalid)?;
        let role=p.principal.ok_or(Error::Denied)? as usize;
        if !matches!(role,0|1|3|8)||p.state!=State::Active||
            p.caps.resolve(handle,RECEIVE)?!=Object::Receive(Endpoint{slot:caller as u8,incarnation:p.incarnation}){
            return Err(Error::Denied);
        }
        Ok(role)
    }
    /// Returns only the actual service's pre-granted fixed control handle.
    /// This does not mint handles, choose arbitrary destinations or expose Data.
    pub fn application_channel(&self,caller:usize,self_receive:u64,destination:usize)->Result<u64,Error>{
        let role=self.application_query(caller,self_receive)?;
        let (_,slot,_)=CHANNELS.iter().find(|&&(r,_,d)|r==role&&d as usize==destination)
            .ok_or(Error::Denied)?;
        self.processes[caller].caps.handle(*slot)
    }
    /// Fixed app status for existing trusted services. The owner namespace digest
    /// is needed only by Storage/Manager and is redacted for shell/compositor.
    pub fn application_record(&self,caller:usize,self_receive:u64,index:usize)->Result<Option<AppRecord>,Error>{
        let role=self.application_query(caller,self_receive)?;
        let mut record=*self.apps.records.get(index).ok_or(Error::Invalid)?;
        if !matches!(role,1|8){if let Some(r)=&mut record{r.image.owner=[0;32];}}
        Ok(record)
    }
    pub fn application_binding(&self,principal:usize)->Result<Option<Endpoint>,Error>{
        if !(APP_FIRST..APP_FIRST+APP_COUNT).contains(&principal){return Err(Error::Invalid);}
        Ok(self.apps.binding(principal))
    }
    /// Kernel-internal bridge, not a userspace-supplied manifest. The native
    /// caller must derive AppImage from VerifiedApp for the exact immutable bank,
    /// enforce its generation floor, and prove the fixed physical slot is Clean.
    pub fn prepare_application(&self,caller:usize,handle:u64,index:usize,image:AppImage)
        ->Result<AppHandover,Error>{
        self.manager(caller,handle)?;let peers=self.application_peers()?;
        if !self.apps.controls{return Err(Error::Denied);}
        let slot=*APP_SLOTS.get(index).ok_or(Error::Invalid)?;image.validate(index)?;
        if self.apps.records[index].is_some()||self.processes[slot].state!=State::Vacant{
            return Err(Error::Busy);
        }
        let incarnation=self.clock.checked_add(1).ok_or(Error::Exhausted)?;
        let endpoint=Endpoint{slot:slot as u8,incarnation};
        let mut caps=self.processes[slot].caps;
        caps.grant(0,Object::Receive(endpoint),RECEIVE)?;
        caps.grant(1,Object::NamedSend{principal:3},SEND)?;
        if image.rights&2!=0{caps.grant(2,Object::NamedSend{principal:1},SEND)?;}
        Ok(AppHandover{index,record:AppRecord{endpoint,image},caps,clock:self.clock,epoch:self.apps.epoch,peers})
    }
    /// Same-trap publication after successful private construction. All refusal
    /// precedes mutation. Native scheduling must happen only after success.
    pub fn publish_application(&mut self,caller:usize,handle:u64,plan:AppHandover)->Result<(),Error>{
        self.manager(caller,handle)?;
        if self.application_peers()?!=plan.peers||!self.apps.controls||self.clock!=plan.clock||self.apps.epoch!=plan.epoch{
            return Err(Error::Stale);
        }
        let slot=APP_SLOTS[plan.index];
        if self.apps.records[plan.index].is_some()||self.processes[slot].state!=State::Vacant{
            return Err(Error::Busy);
        }
        // Slot generations may change without advancing the global clock (for
        // example an internal retirement). Never restore stale prepared grants.
        let mut current=self.processes[slot].caps;
        current.grant(0,Object::Receive(plan.record.endpoint),RECEIVE)?;
        current.grant(1,Object::NamedSend{principal:3},SEND)?;
        if plan.record.image.rights&2!=0{current.grant(2,Object::NamedSend{principal:1},SEND)?;}
        if current!=plan.caps{return Err(Error::Stale);}
        self.clock=plan.record.endpoint.incarnation;
        self.processes[slot]=Process{state:State::Active,principal:Some((APP_FIRST+plan.index)as u8),
            incarnation:self.clock,caps:plan.caps,queue:Queue::new()};
        self.apps.records[plan.index]=Some(plan.record);Ok(())
    }
    /// Manager can close only a currently matching app incarnation. No arbitrary
    /// process selector; delayed close cannot destroy a newly launched app.
    pub fn close_application(&mut self,caller:usize,handle:u64,index:usize,incarnation:u64)->Result<(),Error>{
        self.manager(caller,handle)?;self.application_peers()?;
        let record=self.apps.records.get(index).copied().flatten().ok_or(Error::Stale)?;
        if incarnation==0||record.endpoint.incarnation!=incarnation{return Err(Error::Stale);}
        self.destroy(record.endpoint.slot as usize);Ok(())
    }
    pub(super) fn revoke_application_peers(&mut self){
        if !self.apps.enabled{return;}
        for i in APP_SLOTS{
            if self.processes[i].state!=State::Vacant{self.destroy(i);}
        }
        if self.apps.controls{
            for(role,slot,_)in CHANNELS{let _=self.processes[role].caps.revoke(slot);}
        }
        self.apps.controls=false;
    }
}

#[cfg(test)]
mod tests{
    use super::*;
    fn image(index:usize)->AppImage{
        AppImage{application:[index as u8+1;16],owner:[index as u8+1;32],
            digest:[index as u8+2;32],generation:1,rights:if index==0{3}else{1}}
    }
    fn runtime()->Runtime{
        let mut r=Runtime::applications_bootstrap();let h=r.handle(8,MANAGER_CAP).unwrap();
        r.authenticated_stage(8,h,1,5,[1;32],1,20).unwrap();
        let t=r.begin_trial(8,h,1).unwrap();
        r.ready(5,r.handle(5,HEALTH_CAP).unwrap(),t.token()).unwrap();
        let p=r.prepare_desktop(8,h,t.token()).unwrap();r.publish_desktop(8,h,p).unwrap();
        r.enable_application_controls(8,h).unwrap();r
    }
    fn launch(r:&mut Runtime,index:usize)->AppRecord{
        let h=r.handle(8,MANAGER_CAP).unwrap();
        let p=r.prepare_application(8,h,index,image(index)).unwrap();let record=p.record();
        r.publish_application(8,h,p).unwrap();record
    }
    #[test]fn legacy_bindings_and_boot_graph_stay_separate(){
        assert_eq!(PRINCIPALS,10);
        for mut r in [Runtime::new(),Runtime::bootstrap(),Runtime::expansion_bootstrap()]{
            let h=r.handle(8,MANAGER_CAP).unwrap();
            assert!(r.enable_application_controls(8,h).is_err());
            assert!(r.prepare_application(8,h,0,image(0)).is_err());
            assert_eq!(r.binding(10),Err(Error::Invalid));
            assert_eq!(r.application_binding(10),Ok(None));
        }
        let mut r=runtime();let before=r.binding_generations();let a=launch(&mut r,0);
        assert_eq!(r.binding_generations(),before);assert_eq!(a.endpoint.slot,11);
        assert_eq!(r.binding(10),Err(Error::Invalid));
        assert_eq!(r.application_binding(10),Ok(Some(a.endpoint)));
    }
    #[test]fn real_graph_channels_full_identity_and_least_authority(){
        let mut r=runtime();let a=launch(&mut r,0);let b=launch(&mut r,1);
        for (record,principal)in [(a,10),(b,11)]{
            let slot=record.endpoint.slot as usize;
            r.send(slot,r.handle(slot,1).unwrap(),b"paint").unwrap();
            let m=r.receive(3,r.handle(3,0).unwrap()).unwrap();
            assert_eq!((m.principal,m.incarnation),(principal,record.endpoint.incarnation));
            for i in 3..CAP_SLOTS{assert!(r.handle(slot,i).is_err());}
            for h in [r.handle(slot,0).unwrap(),r.handle(slot,1).unwrap()]{
                assert!(r.device(slot,h).is_err());assert!(r.framebuffer(slot,h).is_err());
                assert!(r.input(slot,h).is_err());assert!(r.network(slot,h).is_err());
                assert!(r.stage_view(slot,h).is_err());
                assert!(r.application_channel(slot,h,8).is_err());
            }
        }
        assert!(r.handle(12,2).is_err());
        r.send(11,r.handle(11,2).unwrap(),b"private").unwrap();
        assert_eq!(r.receive(1,r.handle(1,0).unwrap()).unwrap().principal,10);
        let h=r.application_channel(3,r.handle(3,0).unwrap(),10).unwrap();
        r.send(3,h,b"input").unwrap();
        assert_eq!(r.receive(11,r.handle(11,0).unwrap()).unwrap().principal,3);
        assert!(r.application_channel(0,r.handle(0,0).unwrap(),10).is_err());
        for role in [0,3]{
            assert_eq!(r.application_record(role,r.handle(role,0).unwrap(),0).unwrap().unwrap().image.owner,[0;32]);
        }
        assert_eq!(r.application_record(1,r.handle(1,0).unwrap(),0).unwrap().unwrap().image.owner,a.image.owner);
    }
    #[test]fn preparation_is_unpublished_atomic_and_clock_checked(){
        let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
        let p=r.prepare_application(8,h,0,image(0)).unwrap();
        assert_eq!(r.application_binding(10),Ok(None));assert_eq!(r.state(11),Ok(State::Vacant));
        assert!(r.handle(11,0).is_err());
        let b=sdk::Boot::decode(&p.bootstrap(0x401000).unwrap()).unwrap();
        assert_eq!(b.principal,10);assert_eq!(b.rights,3);
        assert_eq!(b.peer_incarnations[0],r.binding(3).unwrap().unwrap().incarnation);
        assert!(p.bootstrap(0x420000).is_err());assert!(p.bootstrap(0).is_err());
        launch(&mut r,1);assert_eq!(r.publish_application(8,h,p),Err(Error::Stale));
        assert_eq!(r.application_binding(10),Ok(None));
        let p=r.prepare_application(8,h,0,image(0)).unwrap();
        r.processes[11].caps.revoke(0).unwrap();
        assert_eq!(r.publish_application(8,h,p),Err(Error::Stale));
        assert_eq!(r.state(11),Ok(State::Vacant));
        r.clock=u64::MAX;assert!(matches!(r.prepare_application(8,h,0,image(0)),Err(Error::Exhausted)));
    }
    #[test]fn app_failure_purges_queues_revokes_handles_and_does_not_recover_os(){
        let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
        let before=r.binding_generations();let a=launch(&mut r,0);let old=r.handle(11,1).unwrap();
        r.send(11,old,b"old").unwrap();r.fault(a.endpoint).unwrap();
        assert_eq!(r.receive(3,r.handle(3,0).unwrap()),Err(Error::Empty));
        assert_eq!(r.binding_generations(),before);assert!(!r.recovery_required());
        assert_eq!(r.application_binding(10),Ok(None));
        let b=launch(&mut r,0);assert_ne!(a.endpoint.incarnation,b.endpoint.incarnation);
        assert!(r.send(11,old,b"late").is_err());
        assert_eq!(r.close_application(8,h,0,a.endpoint.incarnation),Err(Error::Stale));
        assert_eq!(r.fault(a.endpoint),Err(Error::Stale));
        assert_eq!(r.application_binding(10),Ok(Some(b.endpoint)));
        r.close_application(8,h,0,b.endpoint.incarnation).unwrap();
    }
    #[test]fn every_required_peer_failure_retires_both_apps_and_control_grants(){
        for role in PEERS{
            let mut r=runtime();launch(&mut r,0);launch(&mut r,1);
            let e=r.binding(role).unwrap().unwrap();r.fault(e).unwrap();
            for principal in [10,11]{assert_eq!(r.application_binding(principal),Ok(None));}
            for slot in APP_SLOTS{assert_eq!(r.state(slot),Ok(State::Vacant));assert!(r.handle(slot,0).is_err());}
            assert!(!r.apps.controls);
            for(caller,slot,_)in CHANNELS{assert!(r.handle(caller,slot).is_err());}
        }
    }
    #[test]fn control_grants_and_prepared_app_refuse_without_partial_publication(){
        let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
        let original=r.application_channel(0,r.handle(0,0).unwrap(),8).unwrap();
        r.enable_application_controls(8,h).unwrap();
        assert_eq!(r.application_channel(0,r.handle(0,0).unwrap(),8),Ok(original));
        for caller in 0..TASKS{if caller!=8{
            assert!(r.prepare_application(caller,h,0,image(0)).is_err());
            assert!(r.enable_application_controls(caller,h).is_err());
        }}
        for rights in [0,1,2,4,7,8,15,u32::MAX]{
            let mut i=image(0);i.rights=rights;
            assert!(r.prepare_application(8,h,0,i).is_err());
        }
        for peer in PEERS{
            let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
            let p=r.prepare_application(8,h,0,image(0)).unwrap();
            r.fault(r.binding(peer).unwrap().unwrap()).unwrap();
            assert!(r.publish_application(8,h,p).is_err());
            assert_eq!(r.application_binding(10),Ok(None));
        }
    }
    #[test]fn every_prepared_cap_generation_including_empty_slots_is_revalidated(){
        for slot in 0..CAP_SLOTS{
            let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
            let p=r.prepare_application(8,h,0,image(0)).unwrap();
            r.processes[11].caps.revoke(slot).unwrap();
            assert!(r.publish_application(8,h,p).is_err(),"{slot}");
            assert_eq!(r.application_binding(10),Ok(None));
        }
    }
    #[test]fn revoked_control_epoch_cannot_revive_a_prepared_app(){
        let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
        let p=r.prepare_application(8,h,0,image(0)).unwrap();
        r.revoke_application_peers();r.enable_application_controls(8,h).unwrap();
        assert_eq!(r.publish_application(8,h,p),Err(Error::Stale));
        r.revoke_application_peers();
        r.processes[3].caps.slots[10].retired=true;
        assert!(r.enable_application_controls(8,h).is_err());
        for(role,slot,_)in CHANNELS{assert!(r.handle(role,slot).is_err());}
        assert!(!r.apps.controls);
        let mut r=runtime();let h=r.handle(8,MANAGER_CAP).unwrap();
        r.revoke_application_peers();r.apps.epoch=u64::MAX;
        assert_eq!(r.enable_application_controls(8,h),Err(Error::Exhausted));
        assert!(!r.apps.controls);
    }
    #[test]fn independent_sender_queue_budget_and_handle_exhaustion_fail_closed(){
        let mut r=runtime();launch(&mut r,0);launch(&mut r,1);
        for slot in APP_SLOTS{
            let h=r.handle(slot,1).unwrap();r.send(slot,h,b"one").unwrap();r.send(slot,h,b"two").unwrap();
            assert_eq!(r.send(slot,h,b"three"),Err(Error::Full));
        }
        let h=r.handle(8,MANAGER_CAP).unwrap();
        let e=r.application_binding(10).unwrap().unwrap();r.fault(e).unwrap();
        r.processes[11].caps.slots[0].generation=u32::MAX;
        r.processes[11].caps.revoke(0).unwrap();
        assert!(r.prepare_application(8,h,0,image(0)).is_err());
        assert_eq!(r.application_binding(10),Ok(None));
    }
}
