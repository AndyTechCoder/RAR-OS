//! Safe child of repair: the sole native CompleteRead issuer.
//! No raw-slice issuer, callback-based issuer, I/O-error damage or borrowed Plan.
use super::{CompleteRead,Factory,Inspection,Plan,Role,inspect};
use crate::{repair_wire::Phase,update_manager::Failure,update_runtime::RepairRuntime,manifest};
pub(crate) fn stored(runtime:&mut RepairRuntime<'_>,phase:Phase,role:Role)->Result<Inspection,Failure>{
    if !matches!((phase,role),(Phase::Active|Phase::FreshActive,Role::Active)|
        (Phase::Prior|Phase::FreshPrior,Role::Prior)){return Err(Failure::Native);}
    let current=runtime.current();let lease=runtime.acquire(phase)?;
    let result=(||{
        let bytes=lease.checked_bytes()?;
        inspect(current,role,CompleteRead{bytes}).map_err(|_|Failure::Verify)
    })();
    match result{Ok(observation)=>{lease.release()?;Ok(observation)},
        Err(error)=>{lease.abort()?;Err(error)}}
}
fn logical_factory(stored:&[u8])->Result<&[u8],Failure>{
    if stored.len()<512||stored.len()%512!=0{return Err(Failure::Verify);}
    manifest::Manifest::parse(&stored[..manifest::SIZE]).map_err(|_|Failure::Verify)?;
    let payload=u32::from_le_bytes(stored[56..60].try_into().unwrap())as usize;
    if !(512..=manifest::MAX_PAYLOAD).contains(&payload){return Err(Failure::Verify);}
    let length=manifest::SIZE+payload;
    if length.div_ceil(512)*512!=stored.len()||stored[length..].iter().any(|&b|b!=0){
        return Err(Failure::Verify);
    }
    Ok(&stored[..length])
}
pub(crate) fn plan(runtime:&mut RepairRuntime<'_>,root:[u8;32],active:Inspection,prior:Option<Inspection>)
    ->Result<Plan,Failure>{
    let current=runtime.current();let lease=runtime.acquire(Phase::Factory)?;
    let result=(||{
        let factory=Factory::verify(logical_factory(lease.checked_bytes()?)?,root).map_err(|_|Failure::Verify)?;
        Plan::new(current,&factory,active,prior).map_err(|_|Failure::Verify)
    })();
    match result{Ok(plan)=>{lease.release()?;Ok(plan)},Err(error)=>{lease.abort()?;Err(error)}}
}
pub(crate) fn recheck(runtime:&mut RepairRuntime<'_>,plan:&Plan,root:[u8;32],active:Inspection,prior:Option<Inspection>)
    ->Result<(),Failure>{
    let current=runtime.current();let lease=runtime.acquire(Phase::FreshFactory)?;
    let result=(||{
        let factory=Factory::verify(logical_factory(lease.checked_bytes()?)?,root).map_err(|_|Failure::Verify)?;
        plan.matches_observations(current,&factory,active,prior).map_err(|_|Failure::Verify)
    })();
    match result{Ok(())=>lease.release(),Err(error)=>{lease.abort()?;Err(error)}}
}
