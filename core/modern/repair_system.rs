//! Exclusive System bootstrap inspection and repair preparation, never Data I/O.
//! Manager alone validates signed content/root/damage and authorizes exact next.
#![forbid(unsafe_code)]
use crate::{journal::Record,manifest,sha256::{sha256,Sha256},
    system_volume::{Io,Volume,Prepared,InspectionRole},
    repair_wire::{Snapshot,Phase,Inspection,ProposalReceiver,PART},
    update_system::Stage};
pub enum Reply{Ignore,Frame([u8;128]),Prepared{prepared:Prepared,next:Record,request:u64},Halt}
enum State{Record(usize),Ready(Phase),Offered(Inspection),Proposal(ProposalReceiver),Prepare(Record),Done,Halted}
pub struct Session{
    manager:u64,current:Record,snapshot:Snapshot,state:State,
    observations:[Option<Inspection>;3],factory_hash:Option<[u8;32]>,
}
impl Session{
    /// Caller must consume its bootstrap-only one-shot gate before calling.
    pub fn begin<I:Io>(volume:&mut Volume<I>,manager:u64,request:u64)->Result<Self,()>{
        if manager==0{return Err(());}
        let current=volume.inspection_record().map_err(|_|())?;
        let snapshot=Snapshot::from_record(request,current).map_err(|_|())?;
        Ok(Self{manager,current,snapshot,state:State::Record(0),
            observations:[None;3],factory_hash:None})
    }
    pub fn snapshot_frame(&self)->[u8;128]{self.snapshot.frame().unwrap()}
    fn halt(&mut self)->Reply{self.state=State::Halted;Reply::Halt}
    pub fn handle<I:Io,S:Stage,F:FnOnce(u64)->Option<&'static[u8]>>(
        &mut self,sender:u64,incarnation:u64,bytes:&[u8],volume:&mut Volume<I>,stage:&mut S,input:F
    )->Reply{
        if sender!=8||incarnation!=self.manager{return Reply::Ignore;}
        if bytes.len()!=128{return self.halt();}
        match &mut self.state{
            State::Halted|State::Done=>Reply::Halt,
            State::Record(offset)=>{
                let offset=*offset;
                if self.snapshot.check_get(bytes,offset).is_err(){return self.halt();}
                let record=self.current.encode();let n=(512-offset).min(PART);
                let frame=self.snapshot.part(offset,Some(&record[offset..offset+n])).unwrap();
                self.state=if offset+n==512{State::Ready(Phase::Active)}else{State::Record(offset+n)};
                Reply::Frame(frame)
            },
            State::Ready(phase)=>{
                let phase=*phase;
                if self.snapshot.check_control(bytes,phase,None,false).is_err(){return self.halt();}
                let offer=match self.inspect(phase,volume,stage,input){Ok(t)=>t,Err(())=>return self.halt()};
                let frame=match offer.frame(){Ok(f)=>f,Err(_)=>return self.halt()};
                self.state=State::Offered(offer);Reply::Frame(frame)
            },
            State::Offered(offer)=>{
                let offer=*offer;
                // Trusted Manager sends this only after successful VIEW11 scrub.
                if self.snapshot.check_control(bytes,offer.phase,Some(offer.seal),false).is_err(){
                    return self.halt();
                }
                let frame=self.snapshot.control(offer.phase,Some(offer.seal),true).unwrap();
                self.state=match offer.phase.next(self.current.previous().is_some()){
                    Some(phase)=>State::Ready(phase),
                    None=>State::Proposal(ProposalReceiver::new(self.snapshot,self.current).unwrap()),
                };
                Reply::Frame(frame)
            },
            State::Proposal(receiver)=>{
                let offset=receiver.offset();
                if receiver.push(bytes).is_err(){return self.halt();}
                let frame=self.snapshot.proposal_part(offset,None).unwrap();
                if receiver.offset()==512{
                    let State::Proposal(receiver)=core::mem::replace(&mut self.state,State::Halted)else{unreachable!()};
                    let next=match receiver.finish(){Ok(r)=>r,Err(_)=>return self.halt()};
                    self.state=State::Prepare(next);
                }
                Reply::Frame(frame)
            },
            State::Prepare(next)=>{
                let next=*next;
                if self.snapshot.check_prepare(bytes).is_err()||
                    volume.inspection_record()!=Ok(self.current){return self.halt();}
                let Some(package)=input(4)else{return self.halt()};
                let Some(hash)=self.factory_hash else{return self.halt()};
                if sha256(package)!=Ok(hash){return self.halt();}
                // This is the first write, after all six (or four) fresh phases.
                let prepared=match volume.prepare_repair(package,hash,next){
                    Ok(p)=>p,Err(_)=>return self.halt(),
                };
                self.state=State::Done;
                Reply::Prepared{prepared,next,request:self.snapshot.request}
            },
        }
    }
    fn inspect<I:Io,S:Stage,F:FnOnce(u64)->Option<&'static[u8]>>(
        &mut self,phase:Phase,volume:&mut Volume<I>,stage:&mut S,input:F
    )->Result<Inspection,()>{
        if volume.inspection_record()!=Ok(self.current){return Err(());}
        let mut seal=0;
        let result=(||{
            let(slot,generation,digest,length,stored_hash)=match phase{
                Phase::Active|Phase::Prior|Phase::FreshActive|Phase::FreshPrior=>{
                    let prior=matches!(phase,Phase::Prior|Phase::FreshPrior);
                    let role=if prior{InspectionRole::Prior}else{InspectionRole::Active};
                    let layer=if prior{self.current.previous().ok_or(())?}else{self.current.active()};
                    let(length,hash)=volume.inspect_stored(role,|total,offset,bytes|{
                        if offset==0{
                            seal=stage.begin_inspection(total)?;
                            if seal==0{return Err(());}
                        }
                        stage.append(seal,offset,bytes)
                    }).map_err(|_|())?;
                    (layer.slot(),layer.generation(),layer.digest(),length,hash)
                },
                Phase::Factory|Phase::FreshFactory=>{
                    let package=input(4).ok_or(())?;
                    if !(896..=crate::system_volume::MAX_PACKAGE).contains(&package.len()){return Err(());}
                    let parsed=manifest::Manifest::parse(&package[..manifest::SIZE]).map_err(|_|())?;
                    let logical=manifest::SIZE+u32::from_le_bytes(package[56..60].try_into().unwrap())as usize;
                    if logical!=package.len()||parsed.generation()!=1||parsed.digest()==[0;32]{return Err(());}
                    let hash=sha256(package).map_err(|_|())?;
                    if phase==Phase::Factory{
                        if self.factory_hash.is_some(){return Err(());}self.factory_hash=Some(hash);
                    }else if self.factory_hash!=Some(hash){return Err(());}
                    let length=package.len().div_ceil(512)*512;
                    seal=stage.begin_inspection(length)?;if seal==0{return Err(());}
                    let mut hash=Sha256::new();
                    for offset in (0..length).step_by(512){
                        let mut block=[0u8;512];let n=(package.len()-offset).min(512);
                        block[..n].copy_from_slice(&package[offset..offset+n]);
                        hash.update(&block).map_err(|_|())?;stage.append(seal,offset,&block)?;
                    }
                    (self.current.active().slot().other(),1,parsed.digest(),length,hash.finalize())
                },
            };
            if volume.inspection_record()!=Ok(self.current){return Err(());}
            stage.finish(seal,length)?;
            let offer=Inspection{phase,request:self.snapshot.request,seal,sequence:self.snapshot.sequence,
                slot,generation,digest,length,stored_hash};
            if !offer.binds(self.snapshot,self.current,phase){return Err(());}
            let index=match phase{Phase::Active|Phase::FreshActive=>0,
                Phase::Prior|Phase::FreshPrior=>1,_=>2};
            if matches!(phase,Phase::FreshActive|Phase::FreshPrior|Phase::FreshFactory){
                let old=self.observations[index].ok_or(())?;
                if (slot,generation,digest,length,stored_hash)!=
                    (old.slot,old.generation,old.digest,old.length,old.stored_hash){return Err(());}
            }else{
                if self.observations[index].is_some(){return Err(());}
                self.observations[index]=Some(offer);
            }
            Ok(offer)
        })();
        if result.is_err()&&seal!=0{let _=stage.abort(seal);}
        result
    }
}
