//! Native adapters for the private update protocol. No disk path or network input.
use crate::{abi::*,update_wire::{self as wire,Transfer,Mode,Kind,RecordReceiver,PART},
    update_system,update_manager,system_volume};
pub struct StageCopy{handle:u64}
impl StageCopy{pub fn new(boot:&Boot)->Self{Self{handle:boot.caps[STAGE_CAP]}}
    fn call(&mut self,op:u64,seal:u64,offset:usize,bytes:&[u8],length:usize)->Result<(u64,usize),()>{
        let mut reply=[0u8;STAGE_REPLY_BYTES];
        let pointer=if bytes.is_empty(){0}else{bytes.as_ptr()as u64};
        let values=[op,seal,offset as u64,pointer,length as u64,reply.as_mut_ptr()as u64];
        let mut request=[0u8;STAGE_REQUEST_BYTES];
        for (i,value)in values.iter().enumerate(){request[i*8..i*8+8].copy_from_slice(&value.to_le_bytes());}
        if crate::syscall(STAGE_COPY,self.handle,request.as_ptr()as u64,STAGE_REQUEST_BYTES as u64,0)!=0{
            return Err(());
        }
        let actual=u64::from_le_bytes(reply[..8].try_into().unwrap());
        let accepted=usize::try_from(u64::from_le_bytes(reply[8..].try_into().unwrap())).map_err(|_|())?;
        Ok((actual,accepted))
    }
}
impl update_system::Stage for StageCopy{
    fn begin(&mut self,length:usize)->Result<u64,()>{
        let(seal,accepted)=self.call(0,0,0,&[],length)?;
        if seal==0||accepted!=0{crate::fail();}Ok(seal)
    }
    fn append(&mut self,seal:u64,offset:usize,bytes:&[u8])->Result<(),()>{
        let expected=offset.checked_add(bytes.len()).ok_or(())?;
        if self.call(1,seal,offset,bytes,bytes.len())?==(seal,expected){Ok(())}else{Err(())}
    }
    fn finish(&mut self,seal:u64,length:usize)->Result<(),()>{
        if self.call(2,seal,0,&[],0)?==(seal,length){Ok(())}else{Err(())}
    }
    fn abort(&mut self,seal:u64)->Result<(),()>{
        if self.call(3,seal,0,&[],0)?==(seal,0){Ok(())}else{Err(())}
    }
}
pub fn system<I:system_volume::Io>(boot:&Boot,volume:system_volume::Volume<I>,
    input:fn(u64)->Option<&'static[u8]>)->!{
    let mut server=update_system::Server::new(volume,boot.peers[8]).unwrap_or_else(|_|crate::fail());
    let mut stage=StageCopy::new(boot);
    loop{
        let e=crate::receive(boot.caps[SELF_RECV]);
        match server.handle(e.sender,e.generation,&e.bytes,&mut stage,input){
            update_system::Reply::Ignore=>{},
            update_system::Reply::Halt=>crate::fail(),
            // One bounded send. If the channel cannot deliver, stop; never
            // replay publication or convert the uncertain result into success.
            update_system::Reply::Frame(f)=>{
                if crate::send(boot.caps[UPDATE_PEER],&f).is_err(){crate::fail();}
            }
        }
    }
}
use update_manager::{Failure,BootAction,ReleaseAction};
fn send(boot:&Boot,frame:&[u8;128])->Result<(),Failure>{
    crate::send(boot.caps[UPDATE_PEER],frame).map_err(|_|Failure::Channel)
}
fn tick()->Result<u64,Failure>{
    let n=crate::syscall(TICKS,0,0,0,0);
    if n<0{Err(Failure::Native)}else{Ok(n as u64)}
}
fn receive(boot:&Boot)->Result<[u8;128],Failure>{
    let start=tick()?;
    for _ in 0..65_536{
        if tick()?.checked_sub(start).ok_or(Failure::Native)?>1000{return Err(Failure::Channel);}
        match crate::poll_checked(boot.caps[SELF_RECV]).map_err(|_|Failure::Channel)?{
            Some(e)if e.length==128&&e.sender==9&&e.generation==boot.peers[9]=>return Ok(e.bytes),
            _=>crate::yield_now(),
        }
    }
    Err(Failure::Channel)
}
fn exchange(boot:&Boot,frame:&[u8;128])->Result<[u8;128],Failure>{send(boot,frame)?;receive(boot)}
fn control(boot:&Boot,op:u64,arg:u64,last:u64)->Result<(),Failure>{
    if crate::syscall(STAGE_VIEW,boot.caps[MANAGER],op,arg,last)==0{Ok(())}else{Err(Failure::Native)}
}
fn cancel(boot:&Boot,t:Transfer)->Result<(),Failure>{
    let reply=exchange(boot,&t.frame(Kind::Cancel).map_err(|_|Failure::Native)?)?;
    if t.matches(&reply,Kind::Cancelled){Ok(())}else{Err(Failure::Channel)}
}
/// Full one-shot transaction. Rejected/Verify/Trial are cleanly closed and may
/// authorize only the boot coordinator\'s single exact-prior attempt. They do not
/// authorize generic retries. Channel/native/indeterminate errors halt
/// the manager; no automatic retransmission or remount can hide an unknown ACK.
fn transaction(boot:&Boot,requests:&mut wire::Requests,mode:Mode,index:u64)->Result<(),Failure>{
    let id=requests.next().map_err(|_|Failure::Native)?;
    requests.accept(id).map_err(|_|Failure::Native)?;
    let first=exchange(boot,&wire::request(Kind::Start,mode,id,index).map_err(|_|Failure::Native)?)?;
    if wire::parse_request(&first,Kind::Rejected)==Ok((mode,id,index)){return Err(Failure::Rejected);}
    let t=Transfer::parse(&first,Kind::Offer).map_err(|_|Failure::Channel)?;
    if (t.mode,t.request)!=(mode,id){return Err(Failure::Channel);}
    let mut record=RecordReceiver::new(t,Kind::RecordPart).map_err(|_|Failure::Channel)?;
    for offset in (0..512).step_by(PART){
        let reply=exchange(boot,&t.part(Kind::RecordGet,offset,&[]).map_err(|_|Failure::Channel)?)?;
        record.push(&reply).map_err(|_|Failure::Channel)?;
    }
    let record=record.finish().map_err(|_|Failure::Channel)?;
    let mut view=[0u8;STAGE_VIEW_BYTES];
    control(boot,0,view.as_mut_ptr()as u64,STAGE_VIEW_BYTES as u64)?;
    let word=|i:usize|u64::from_le_bytes(view[i*8..i*8+8].try_into().unwrap());
    let length=usize::try_from(word(1)).map_err(|_|Failure::Native)?;
    if word(0)!=t.seal||length!=t.identity.length||word(2)!=STAGE_VIEW_ADDRESS||
        !matches!(word(3),5|7)||!(896..=system_volume::MAX_PACKAGE).contains(&length){
        return Err(Failure::Native);
    }
    let expected_slot=word(3);
    let verified={
        // SAFETY: exact kernel STAGE_VIEW response is checked above. Mapping
        // is read-only/NX and kernel-sealed across every root until this manager
        // explicitly rejects or accepts it. No Rust reference survives accept.
        let bytes=unsafe{core::slice::from_raw_parts(STAGE_VIEW_ADDRESS as *const u8,length)};
        update_manager::verify(t,record,bytes).map(|v|(v.next(),v.expected_ack()))
    };
    let(next,ack)=match verified{
        Ok((next,Ok(ack)))=>(next,ack),
        _=>{control(boot,1,t.seal,0)?;cancel(boot,t)?;return Err(Failure::Verify);}
    };
    let mut trial=[0u8;32];
    control(boot,2,t.seal,trial.as_mut_ptr()as u64)?;
    let word=|i:usize|u64::from_le_bytes(trial[i*8..i*8+8].try_into().unwrap());
    let token=word(1);let incarnation=word(3);
    if word(0)!=t.seal||token==0||incarnation==0||word(2)!=expected_slot{return Err(Failure::Native);}
    let start=tick()?;let mut healthy=false;
    for _ in 0..65_536{
        let mut status=[0u8;32];
        if control(boot,5,t.seal,status.as_mut_ptr()as u64).is_err(){break;}
        let word=|i:usize|u64::from_le_bytes(status[i*8..i*8+8].try_into().unwrap());
        if (word(0),word(1),word(2))!=(t.seal,token,incarnation){return Err(Failure::Native);}
        if word(3)==2{healthy=true;break;}
        if word(3)!=1||tick()?.checked_sub(start).ok_or(Failure::Native)?>1000{break;}
        crate::yield_now();
    }
    if !healthy{
        // A trial fault may have already removed the exact pending trial.
        // Only Stale is tolerated here, and release independently requires the
        // absence of every trial/staged owner plus completed physical retirement.
        match crate::syscall(STAGE_VIEW,boot.caps[MANAGER],4,token,0){
            0|-3=>{},_=>return Err(Failure::Native),
        }
        release(boot,t.seal)?;cancel(boot,t)?;return Err(Failure::Trial);
    }
    if control(boot,6,token,t.seal).is_err(){
        match crate::syscall(STAGE_VIEW,boot.caps[MANAGER],4,token,0){
            0|-3=>{},_=>return Err(Failure::Native),
        }
        release(boot,t.seal)?;cancel(boot,t)?;return Err(Failure::Trial);
    }
    if let Some(next)=next{
        let bytes=next.encode();
        for offset in (0..512).step_by(PART){
            let n=(512-offset).min(PART);
            let reply=exchange(boot,&t.part(Kind::PublishPart,offset,&bytes[offset..offset+n])
                .map_err(|_|Failure::Native)?)?;
            t.check_part(&reply,Kind::PartAck,offset).map_err(|_|Failure::Channel)?;
        }
    }
    send(boot,&t.frame(Kind::Commit).map_err(|_|Failure::Native)?)?;
    let actual=receive(boot).map_err(|_|Failure::Indeterminate)?;
    if actual!=ack{return Err(Failure::Indeterminate);}
    // Every fallible construction/grant was prepared before Commit. Native
    // invariants after this durable ACK reconcile-halt; they cannot undo disk.
    control(boot,7,token,t.seal).map_err(|_|Failure::Indeterminate)?;
    release(boot,t.seal).map_err(|_|Failure::Indeterminate)
}
fn release(boot:&Boot,seal:u64)->Result<(),Failure>{
    for attempt in 0..256{
        let status=crate::syscall(STAGE_VIEW,boot.caps[MANAGER],3,seal,0);
        match update_manager::release_action(status,attempt){
            ReleaseAction::Done=>return Ok(()),
            ReleaseAction::Yield=>crate::yield_now(),
            ReleaseAction::Stop=>return Err(Failure::Native),
        }
    }
    Err(Failure::Native)
}
/// Whole-guest stop, never a manager EXIT or a selector undo. The fixed kernel
/// branch validates the Manager grant then disables interrupts and halts.
fn reconcile(boot:&Boot)->!{
    let _=crate::syscall(STAGE_VIEW,boot.caps[MANAGER],8,0,0);
    // Unreachable with the matching kernel ABI. Do not continue a transaction.
    crate::fail()
}
pub fn boot_selected(boot:&Boot,requests:&mut wire::Requests){
    let first=transaction(boot,requests,Mode::Boot,0);
    let action=update_manager::boot_action(first,false);
    let action=if action==BootAction::PriorOnce{
        update_manager::boot_action(transaction(boot,requests,Mode::Fallback,0),true)
    }else{action};
    match action{BootAction::Active=>{},_=>reconcile(boot)}
}
/// Future native callers must use this terminal wrapper, never handle an
/// Indeterminate result as an ordinary process-level error.
pub fn install(boot:&Boot,requests:&mut wire::Requests,index:u64)->Result<(),Failure>{
    match transaction(boot,requests,Mode::Install,index){
        Err(Failure::Indeterminate|Failure::Channel|Failure::Native)=>reconcile(boot),
        result=>result,
    }
}
