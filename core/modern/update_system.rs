//! System-owned transaction session. No app authority, arbitrary LBA, or retry.
#![forbid(unsafe_code)]
use crate::{journal::Record,system_volume::{Io,Volume,Prepared},
    update_wire::{self as wire,Transfer,Mode,Kind,Requests,RecordReceiver,PART}};
/// Native adapter implements the exclusive kernel STAGE_COPY grant. An error
/// cannot mean successful Finish: native wrappers validate replies exactly.
/// Begin Err MUST leave no live or partially reserved staging object (consuming
/// a monotonic seal value is allowed). Without a returned seal there is no safe
/// abort identity. A native adapter must fail-stop on an invalid successful reply.
pub trait Stage {
    fn begin(&mut self,length:usize)->Result<u64,()>;
    fn append(&mut self,seal:u64,offset:usize,bytes:&[u8])->Result<(),()>;
    fn finish(&mut self,seal:u64,length:usize)->Result<(),()>;
    fn abort(&mut self,seal:u64)->Result<(),()>;
}
struct Pending {
    prepared:Prepared,transfer:Transfer,record:[u8;512],read_offset:usize,
    publish:Option<RecordReceiver>,next:Option<Record>,
}
#[derive(Debug,PartialEq,Eq)]
pub enum Reply { Ignore, Frame([u8;128]), Halt }
pub struct Server<I:Io> {
    volume:Volume<I>,manager:u64,requests:Requests,pending:Option<Pending>,halted:bool,
}
impl<I:Io> Server<I> {
    pub fn new(volume:Volume<I>,manager:u64)->Result<Self,()> {
        if manager==0{return Err(());}
        Ok(Self{volume,manager,requests:Requests::new(),pending:None,halted:false})
    }
    fn halt(&mut self)->Reply{self.halted=true;Reply::Halt}
    /// Immutable laboratory inputs are supplied by the native owner, never by
    /// an IPC sender. Lookup occurs only after authenticated canonical Start.
    pub fn handle<S:Stage,F:FnOnce(u64)->Option<&'static[u8]>>(
        &mut self,sender:u64,incarnation:u64,bytes:&[u8],stage:&mut S,input:F
    )->Reply {
        if sender!=8||incarnation!=self.manager||bytes.len()!=128{return Reply::Ignore;}
        if self.halted{return Reply::Halt;}
        if self.pending.is_none() {
            let Ok((mode,id,index))=wire::parse_request(bytes,Kind::Start) else{return Reply::Ignore;};
            if self.requests.accept(id).is_err(){return Reply::Ignore;}
            let prepared=match mode {
                Mode::Boot=>self.volume.prepare_boot(),
                Mode::Fallback=>self.volume.prepare_fallback(),
                Mode::Install=>match input(index) {
                    Some(package)=>self.volume.prepare(package),
                    None=>return Reply::Frame(wire::request(Kind::Rejected,mode,id,index).unwrap()),
                },
            };
            let prepared=match prepared {
                Ok(p)=>p,
                Err(_)=>{if self.volume.is_readonly(){return self.halt();}
                    return Reply::Frame(wire::request(Kind::Rejected,mode,id,index).unwrap());}
            };
            let identity=prepared.identity();
            let seal=match stage.begin(identity.length){Ok(s) if s!=0=>s,_=>{
                if self.volume.cancel(&prepared).is_err(){return self.halt();}
                return Reply::Frame(wire::request(Kind::Rejected,mode,id,index).unwrap());
            }};
            let copied=self.volume.copy_prepared(&prepared,|length,offset,chunk|{
                if length!=identity.length{return Err(());}
                stage.append(seal,offset,chunk)
            });
            if copied.is_err(){
                // A partial prefix is never finished. Irrecoverable volume or
                // kernel state ends this owner; there is no remount/retry.
                let _=stage.abort(seal);return self.halt();
            }
            if stage.finish(seal,identity.length).is_err(){
                let _=stage.abort(seal);return self.halt();
            }
            let record=self.volume.record();
            let transfer=Transfer{mode,request:id,seal,sequence:record.sequence(),identity};
            let frame=match transfer.frame(Kind::Offer){Ok(f)=>f,Err(_)=>return self.halt()};
            self.pending=Some(Pending{prepared,transfer,record:record.encode(),read_offset:0,
                publish:None,next:None});
            return Reply::Frame(frame);
        }
        let p=self.pending.as_mut().unwrap();
        let t=p.transfer;
        // Cancellation is accepted only after the authenticated manager has
        // cleared its matching native trial/seal; this is a trusted peer
        // protocol requirement, not permission delegated to an application.
        if t.matches(bytes,Kind::Cancel) {
            if self.volume.cancel(&p.prepared).is_err(){return self.halt();}
            self.pending=None;
            return Reply::Frame(t.frame(Kind::Cancelled).unwrap());
        }
        if p.read_offset<512 {
            let offset=p.read_offset;
            if t.check_part(bytes,Kind::RecordGet,offset).is_err(){return Reply::Ignore;}
            let n=(512-offset).min(PART);
            let frame=t.part(Kind::RecordPart,offset,&p.record[offset..offset+n]).unwrap();
            p.read_offset+=n;return Reply::Frame(frame);
        }
        if t.mode!=Mode::Boot&&p.next.is_none() {
            let receiver=p.publish.get_or_insert_with(||RecordReceiver::new(t,Kind::PublishPart).unwrap());
            let offset=receiver.offset();
            if receiver.push(bytes).is_err(){return Reply::Ignore;}
            let ack=t.part(Kind::PartAck,offset,&[]).unwrap();
            if receiver.offset()==512 {
                let completed=p.publish.take().unwrap().finish();
                match completed {Ok(record)=>p.next=Some(record),Err(_)=>return self.halt()}
            }
            return Reply::Frame(ack);
        }
        if !t.matches(bytes,Kind::Commit){return Reply::Ignore;}
        let result=if t.mode==Mode::Boot{self.volume.complete_boot(&p.prepared)}
            else{self.volume.publish(&p.prepared,p.next.unwrap())};
        // An uncertain publication can never be retried or acknowledged as
        // healthy/active. Restart recovery must reread and reverify the media.
        if result.is_err(){return self.halt();}
        let sequence=self.volume.record().sequence();
        let ack=match t.committed(sequence).and_then(|t|t.frame(Kind::Committed)){
            Ok(ack)=>ack,Err(_)=>return self.halt()
        };
        self.pending=None;Reply::Frame(ack)
    }
}
