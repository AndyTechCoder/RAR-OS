//! Modern ring3 file-session loop. Runtime implements only stamped IPC and ticks.
//! This is used by apps, not a kernel/device capability boundary.
#![forbid(unsafe_code)]
use crate::transport::{Client,Outcome,StartError,MESSAGE_BUDGET};
#[derive(Clone,Copy)]
pub struct Envelope {pub sender:u64,pub generation:u64,pub length:usize,pub bytes:[u8;128]}
/// Adapters must use the current process's fixed storage/receive grants.
/// send_once returns Ok only after the kernel accepted exactly this frame.
/// An error (including queue-full) closes the session; it is never retried.
/// now is a checked monotonic kernel clock, not an app-supplied counter.
/// poll dequeues at most one message without blocking; yield_now relinquishes CPU.
pub trait Runtime {
    fn now(&mut self)->Result<u64,()>;
    fn send_once(&mut self,frame:&[u8;128])->Result<(),()>;
    fn poll(&mut self)->Result<Option<Envelope>,()>;
    fn yield_now(&mut self)->Result<(),()>;
}
/// UI receives only complete current-shell envelopes. Return false on a full
/// bounded input queue. The caller can display input loss; never replay a save.
pub trait Input {fn push(&mut self,frame:[u8;128])->bool;}
pub struct Session {client:Client,shell_generation:u64,closed:bool,input_lost:bool,last_tick:u64}
impl Session {
    /// Incarnations are trusted bootstrap material. Recreating Session under
    /// unchanged live grants is forbidden, including after a failed send.
    pub fn new(storage_generation:u64,shell_generation:u64)->Result<Self,StartError> {
        if shell_generation==0 {return Err(StartError::Invalid);}
        Ok(Self {client:Client::new(storage_generation)?,shell_generation,
            closed:false,input_lost:false,last_tick:0})
    }
    pub fn closed(&self)->bool {self.closed}
    pub fn writes_locked(&self)->bool {self.closed||self.client.writes_locked()}
    pub fn input_lost(&self)->bool {self.input_lost}
    fn clock<R:Runtime>(&mut self,runtime:&mut R)->Result<u64,()> {
        let now=runtime.now()?;
        if now<self.last_tick {return Err(());}
        self.last_tick=now;Ok(now)
    }
    fn abort(&mut self)->Outcome {
        self.closed=true;
        self.client.expire().unwrap_or(Outcome::Unavailable)
    }
    /// One request, one send attempt, bounded absolute deadline and message work.
    /// Only a transport-validated durable success may reach the UI as Reply(OK).
    /// A successful read never clears a prior uncertain mutation or closed grant.
    pub fn call<R:Runtime,I:Input>(&mut self,runtime:&mut R,input:&mut I,body:&[u8;128])
        ->Result<Outcome,StartError> {
        if self.closed {return Ok(Outcome::Unavailable);}
        let now=match self.clock(runtime) {Ok(now)=>now,Err(())=>return Ok(self.abort())};
        let frame=self.client.begin(now,body)?;
        if runtime.send_once(&frame).is_err() {return Ok(self.abort());}
        for _ in 0..MESSAGE_BUDGET {
            let now=match self.clock(runtime) {Ok(now)=>now,Err(())=>return Ok(self.abort())};
            if let Some(outcome)=self.client.tick(now) {return Ok(outcome);}
            let envelope=match runtime.poll() {Ok(envelope)=>envelope,Err(())=>return Ok(self.abort())};
            // Preserve dequeued current-shell input before any timeout/error
            // return. Storage replies are still checked only after fresh time.
            if let Some(envelope)=envelope {
                if envelope.sender==0 && envelope.generation==self.shell_generation &&
                    envelope.length==128 && !input.push(envelope.bytes) {self.input_lost=true;}
            }
            let now=match self.clock(runtime) {Ok(now)=>now,Err(())=>return Ok(self.abort())};
            if let Some(outcome)=self.client.tick(now) {return Ok(outcome);}
            if let Some(envelope)=envelope {
                // A checked length prevents slicing malformed envelopes. All
                // messages, including input and malformed traffic, spend budget.
                let bytes=if envelope.length<=128 {&envelope.bytes[..envelope.length]}else{&[]};
                if let Some(outcome)=self.client.receive(now,envelope.sender,envelope.generation,bytes) {
                    return Ok(outcome);
                }
            }
            if runtime.yield_now().is_err() {return Ok(self.abort());}
        }
        // Defense in depth if ticks stop advancing despite the clock contract.
        // A stalled clock must not cause an unbounded empty-mailbox spin.
        Ok(self.abort())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{desktop_wire as wire,transport::{MAGIC,DEADLINE_TICKS}};
    use std::collections::VecDeque;
    struct Fake {now:u64,step:u64,sends:usize,polls:usize,yields:usize,
        send_error:bool,poll_error:bool,yield_error:bool,clock_error:bool,
        backwards:bool,post_poll_clock:Option<Result<u64,()>>,messages:VecDeque<Envelope>}
    impl Fake {fn new()->Self {Self {now:0,step:1,sends:0,polls:0,yields:0,
        send_error:false,poll_error:false,yield_error:false,clock_error:false,
        backwards:false,post_poll_clock:None,messages:VecDeque::new()}}}
    impl Runtime for Fake {
        fn now(&mut self)->Result<u64,()> {
            if self.clock_error {return Err(());}
            if self.polls>0 {if let Some(value)=self.post_poll_clock {return value;}}
            if self.backwards&&self.sends>0 {return Ok(0);}
            let now=self.now;self.now=self.now.saturating_add(self.step);Ok(now)
        }
        fn send_once(&mut self,_:&[u8;128])->Result<(),()> {
            self.sends+=1;if self.send_error {Err(())}else{Ok(())}
        }
        fn poll(&mut self)->Result<Option<Envelope>,()> {
            self.polls+=1;if self.poll_error {Err(())}else{Ok(self.messages.pop_front())}
        }
        fn yield_now(&mut self)->Result<(),()> {
            self.yields+=1;if self.yield_error {Err(())}else{Ok(())}
        }
    }
    struct Ui {count:usize,accept:bool}
    impl Input for Ui {fn push(&mut self,_:[u8;128])->bool {self.count+=1;self.accept}}
    fn request(op:u8)->[u8;128] {
        wire::request(op,if op==wire::LIST{b""}else{b"note"},b"").unwrap()
    }
    fn response(sender:u64,generation:u64,id:u64,status:u8)->Envelope {
        let mut bytes=[0;128];bytes[0]=status;
        bytes[112..120].copy_from_slice(&id.to_le_bytes());bytes[120..].copy_from_slice(&MAGIC);
        Envelope {sender,generation,length:128,bytes}
    }
    fn ui()->Ui {Ui{count:0,accept:true}}
    #[test] fn full_width_bootstrap_and_stamped_shell_storage_identities() {
        let storage=(1u64<<40)|3;let shell=u64::MAX;
        let mut r=Fake::new();let mut s=Session::new(storage,shell).unwrap();let mut input=ui();
        r.messages.extend([response(0,u32::MAX as u64,0,0),response(0,shell,0,0),
            response(1,3,1,0),response(1,storage,1,0)]);
        assert_eq!(s.call(&mut r,&mut input,&request(wire::LIST)),Ok(Outcome::Reply([0;128])));
        assert_eq!(input.count,1);assert_eq!(r.polls,4);assert!(!s.writes_locked());
    }
    #[test] fn accepted_durable_reply_after_input_and_stale_traffic() {
        let mut r=Fake::new();let mut s=Session::new(3,2).unwrap();let mut input=ui();
        r.messages.extend([response(0,1,1,0),response(0,2,1,0),response(1,1,1,0),
            response(1,3,2,0),response(1,3,1,0)]);
        assert_eq!(s.call(&mut r,&mut input,&request(wire::WRITE)),Ok(Outcome::Reply([0;128])));
        assert_eq!((r.sends,r.polls,r.yields,input.count),(1,5,4,1));
        assert!(!s.closed());assert!(!s.writes_locked());
    }
    #[test] fn send_failure_closes_without_retry_or_sequence_gap_use() {
        for op in [wire::CREATE,wire::WRITE,wire::READ,wire::LIST] {
            let mut r=Fake::new();r.send_error=true;let mut s=Session::new(1,1).unwrap();
            let expected=if matches!(op,wire::CREATE|wire::WRITE){Outcome::SaveUncertain}else{Outcome::Unavailable};
            assert_eq!(s.call(&mut r,&mut ui(),&request(op)),Ok(expected));
            assert!(s.closed());assert!(s.writes_locked());assert_eq!((r.sends,r.polls,r.yields),(1,0,0));
            r.send_error=false;
            assert_eq!(s.call(&mut r,&mut ui(),&request(wire::LIST)),Ok(Outcome::Unavailable));
            assert_eq!(r.sends,1);
        }
    }
    #[test] fn lost_ack_timeout_locks_writes_but_read_can_inspect() {
        let mut r=Fake::new();r.step=DEADLINE_TICKS;let mut s=Session::new(1,1).unwrap();
        assert_eq!(s.call(&mut r,&mut ui(),&request(wire::WRITE)),Ok(Outcome::SaveUncertain));
        assert!(!s.closed());assert!(s.writes_locked());assert_eq!(r.sends,1);
        assert_eq!(s.call(&mut r,&mut ui(),&request(wire::CREATE)),Err(StartError::WritesLocked));
        r.step=1;r.messages.extend([response(1,1,1,0),response(1,1,2,0)]);
        assert_eq!(s.call(&mut r,&mut ui(),&request(wire::READ)),Ok(Outcome::Reply([0;128])));
        assert_eq!(r.sends,2);assert!(s.writes_locked());
    }
    #[test] fn malformed_lengths_input_overflow_and_deadline_keep_bounded() {
        let mut r=Fake::new();let mut s=Session::new(1,1).unwrap();let mut input=Ui{count:0,accept:false};
        let mut short=response(1,1,1,0);short.length=127;
        let mut long=short;long.length=usize::MAX;
        r.messages.extend([short,long,response(0,1,0,0),response(1,1,1,0)]);
        assert_eq!(s.call(&mut r,&mut input,&request(wire::READ)),Ok(Outcome::Reply([0;128])));
        assert!(s.input_lost());assert_eq!(input.count,1);assert!(!s.closed());
        let mut r=Fake::new();
        assert_eq!(s.call(&mut r,&mut input,&request(wire::READ)),Ok(Outcome::Unavailable));
        assert!(s.closed()); // new runtime clock regressed: no fresh session silently
    }
    #[test] fn runtime_failures_and_regression_never_claim_a_save() {
        for failure in 0..4 {
            let mut r=Fake::new();r.now=10;
            match failure {0=>r.poll_error=true,1=>r.yield_error=true,2=>r.backwards=true,_=>r.clock_error=true}
            let mut s=Session::new(1,1).unwrap();
            let expected=if failure==3 {Outcome::Unavailable}else{Outcome::SaveUncertain};
            assert_eq!(s.call(&mut r,&mut ui(),&request(wire::WRITE)),Ok(expected));
            assert!(s.closed());assert_eq!(r.sends,if failure==3{0}else{1});
        }
    }
    #[test] fn dequeued_shell_input_survives_post_poll_deadline_and_clock_error() {
        for post in [Ok(DEADLINE_TICKS),Err(())] {for accept in [false,true] {
            let mut r=Fake::new();r.post_poll_clock=Some(post);
            r.messages.push_back(response(0,2,0,0));
            let mut s=Session::new(3,2).unwrap();let mut input=Ui{count:0,accept};
            assert_eq!(s.call(&mut r,&mut input,&request(wire::WRITE)),Ok(Outcome::SaveUncertain));
            assert_eq!(input.count,1);assert_eq!(s.input_lost(),!accept);
            assert_eq!(r.polls,1);assert!(s.writes_locked());
        }}
        // The same post-poll deadline must NOT accept a late storage success.
        let mut r=Fake::new();r.post_poll_clock=Some(Ok(DEADLINE_TICKS));
        r.messages.push_back(response(1,3,1,0));let mut s=Session::new(3,2).unwrap();
        assert_eq!(s.call(&mut r,&mut ui(),&request(wire::WRITE)),Ok(Outcome::SaveUncertain));
    }
    #[test] fn final_budget_shell_envelope_is_offered_once_or_loss_flagged() {
        for accept in [false,true] {
            let mut r=Fake::new();r.step=0;
            r.messages.extend((1..MESSAGE_BUDGET).map(|_|response(99,1,1,0)));
            r.messages.push_back(response(0,2,0,0));
            let mut s=Session::new(3,2).unwrap();let mut input=Ui{count:0,accept};
            assert_eq!(s.call(&mut r,&mut input,&request(wire::WRITE)),Ok(Outcome::SaveUncertain));
            assert_eq!(r.polls,MESSAGE_BUDGET as usize);assert_eq!(input.count,1);
            assert_eq!(s.input_lost(),!accept);assert!(s.writes_locked());
        }
    }
    #[test] fn stalled_clock_and_empty_mailbox_have_finite_work() {
        let mut r=Fake::new();r.step=0;let mut s=Session::new(1,1).unwrap();
        assert_eq!(s.call(&mut r,&mut ui(),&request(wire::WRITE)),Ok(Outcome::SaveUncertain));
        assert!(s.closed());assert_eq!(r.sends,1);
        assert_eq!(r.polls,MESSAGE_BUDGET as usize);
        assert_eq!(r.yields,MESSAGE_BUDGET as usize);
    }
    #[test] fn unavailable_readonly_and_uncertain_server_states_remain_distinct() {
        for (status,outcome) in [(5,Outcome::ReadOnly),(6,Outcome::Unavailable),(7,Outcome::SaveUncertain)] {
            let mut r=Fake::new();r.messages.push_back(response(1,1,1,status));
            let mut s=Session::new(1,1).unwrap();
            assert_eq!(s.call(&mut r,&mut ui(),&request(wire::WRITE)),Ok(outcome));
            assert!(s.writes_locked());assert_eq!(r.sends,1);
        }
        assert!(Session::new(0,1).is_err());assert!(Session::new(1,0).is_err());
    }
}
