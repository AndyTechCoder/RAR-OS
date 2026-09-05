//! Private Platform-v0 mechanism model; no stable public ABI or allocation.
pub const TASKS: usize = 16;
pub const CAP_SLOTS: usize = 11;
pub const MESSAGE_BYTES: usize = 128;
pub const QUEUE_DEPTH: usize = 4;
pub const SEND: u8 = 1;
pub const RECEIVE: u8 = 2;
pub const PORT_READ: u8 = 4;
pub const DRAW: u8 = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error { Invalid, Denied, Stale, Full, Empty, Exhausted }
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Object { None, Endpoint { task: u8, generation: u32 }, Input, Framebuffer }
#[derive(Clone, Copy)]
struct Slot { generation: u32, rights: u8, object: Object, retired: bool }
impl Slot {
    const EMPTY: Self = Self { generation: 1, rights: 0, object: Object::None, retired: false };
}
#[derive(Clone, Copy)]
pub struct Caps { slots: [Slot; CAP_SLOTS] }
impl Caps {
    pub const fn new() -> Self { Self { slots: [Slot::EMPTY; CAP_SLOTS] } }
    /// Only the nucleus may grant. Handles are interpreted in the caller's table.
    pub fn grant(&mut self, index: usize, object: Object, rights: u8) -> Result<u64, Error> {
        let slot = self.slots.get_mut(index).ok_or(Error::Invalid)?;
        if slot.retired || slot.object != Object::None || object == Object::None ||
           rights == 0 || rights & !(SEND | RECEIVE | PORT_READ | DRAW) != 0 {
            return Err(Error::Denied);
        }
        let allowed = match object {
            Object::Endpoint { task, generation } if (task as usize) < TASKS && generation != 0 => SEND | RECEIVE,
            Object::Input => PORT_READ,
            Object::Framebuffer => DRAW,
            _ => return Err(Error::Invalid),
        };
        if rights & !allowed != 0 { return Err(Error::Denied); }
        slot.object = object; slot.rights = rights;
        Ok((slot.generation as u64) << 32 | (index as u64 + 1))
    }
    pub fn resolve(&self, handle: u64, right: u8) -> Result<Object, Error> {
        let index = (handle as u32).checked_sub(1).ok_or(Error::Invalid)? as usize;
        let slot = self.slots.get(index).ok_or(Error::Invalid)?;
        if slot.retired || slot.generation != (handle >> 32) as u32 || slot.object == Object::None {
            return Err(Error::Stale);
        }
        if right == 0 || right & slot.rights != right { return Err(Error::Denied); }
        Ok(slot.object)
    }
    pub fn revoke(&mut self, index: usize) -> Result<(), Error> {
        let slot = self.slots.get_mut(index).ok_or(Error::Invalid)?;
        slot.object = Object::None; slot.rights = 0;
        if let Some(next) = slot.generation.checked_add(1) { slot.generation = next; }
        else { slot.retired = true; }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Message { pub sender: u8, pub generation: u32, pub length: u8, pub bytes: [u8; MESSAGE_BYTES] }
impl Message {
    const EMPTY: Self = Self { sender: 0, generation: 0, length: 0, bytes: [0; MESSAGE_BYTES] };
    pub fn from_kernel_sender(sender: usize, generation: u32, bytes: &[u8]) -> Result<Self, Error> {
        if sender >= TASKS || generation == 0 || bytes.is_empty() || bytes.len() > MESSAGE_BYTES {
            return Err(Error::Invalid);
        }
        let mut value = Self { sender: sender as u8, generation, length: bytes.len() as u8, ..Self::EMPTY };
        value.bytes[..bytes.len()].copy_from_slice(bytes);
        Ok(value)
    }
}
#[derive(Clone, Copy)]
pub struct Queue { entries: [Message; QUEUE_DEPTH], head: usize, length: usize, sender_limit:usize }
impl Queue {
    pub const fn new() -> Self { Self { entries: [Message::EMPTY; QUEUE_DEPTH], head: 0, length: 0, sender_limit:QUEUE_DEPTH } }
    pub fn with_sender_limit(limit:usize)->Result<Self,Error>{
        if limit==0||limit>QUEUE_DEPTH{return Err(Error::Invalid);}
        Ok(Self{sender_limit:limit,..Self::new()})
    }
    pub fn push(&mut self, value: Message) -> Result<(), Error> {
        let same=(0..self.length).filter(|&i|{
            let old=self.entries[(self.head+i)%QUEUE_DEPTH];
            old.sender==value.sender&&old.generation==value.generation
        }).count();
        if self.length == QUEUE_DEPTH || same>=self.sender_limit { return Err(Error::Full); }
        self.entries[(self.head + self.length) % QUEUE_DEPTH] = value; self.length += 1; Ok(())
    }
    pub fn peek(&self) -> Result<Message, Error> {
        if self.length == 0 { Err(Error::Empty) } else { Ok(self.entries[self.head]) }
    }
    pub fn pop(&mut self) -> Result<Message, Error> {
        let value = self.peek()?;
        self.entries[self.head] = Message::EMPTY;
        self.head = (self.head + 1) % QUEUE_DEPTH; self.length -= 1; Ok(value)
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State { Runnable, Blocked, Dead }
pub fn next(states: &[State; TASKS], current: usize) -> Result<usize, Error> {
    if current >= TASKS { return Err(Error::Invalid); }
    (1..=TASKS).map(|offset| (current + offset) % TASKS)
        .find(|&index| states[index] == State::Runnable).ok_or(Error::Empty)
}
#[derive(Clone, Copy, Debug)]
pub struct UserRange { pub start: u64, pub end: u64, pub writable: bool, pub executable: bool }
pub fn user_buffer(ranges: &[UserRange], pointer: u64, length: usize, write: bool) -> Result<(), Error> {
    if length == 0 || length > MESSAGE_BYTES + 16 || pointer < 4096 { return Err(Error::Invalid); }
    let end = pointer.checked_add(length as u64).ok_or(Error::Invalid)?;
    if end > 0x0000_8000_0000_0000 { return Err(Error::Denied); }
    if ranges.iter().any(|r| r.start <= pointer && end <= r.end && r.start < r.end &&
                          !(r.writable && r.executable) && (!write || r.writable)) { Ok(()) }
    else { Err(Error::Denied) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn rights_and_process_local_tables() {
        let mut a = Caps::new(); let b = Caps::new();
        let h = a.grant(0, Object::Endpoint { task: 1, generation: 1 }, SEND).unwrap();
        assert!(a.resolve(h, SEND).is_ok()); assert_eq!(a.resolve(h, RECEIVE), Err(Error::Denied));
        assert_eq!(b.resolve(h, SEND), Err(Error::Stale));
        assert!(a.grant(1, Object::Input, SEND).is_err());
        assert!(a.grant(1, Object::Endpoint { task: 255, generation: 1 }, SEND).is_err());
    }
    #[test] fn revoked_generation_never_redeems() {
        let mut c = Caps::new(); let h = c.grant(0, Object::Input, PORT_READ).unwrap();
        c.revoke(0).unwrap(); let h2 = c.grant(0, Object::Input, PORT_READ).unwrap();
        assert_ne!(h, h2); assert_eq!(c.resolve(h, PORT_READ), Err(Error::Stale));
        c.slots[0].generation = u32::MAX; c.revoke(0).unwrap();
        assert!(c.grant(0, Object::Input, PORT_READ).is_err());
    }
    #[test] fn malformed_handle_rejected() {
        let c = Caps::new();
        for h in [0, 1, u64::MAX, 1u64 << 32, (1u64 << 32) | 9] { assert!(c.resolve(h, SEND).is_err()); }
    }
    #[test] fn bounded_fifo_and_sender() {
        let mut q = Queue::new();
        for i in 0..QUEUE_DEPTH { q.push(Message::from_kernel_sender(i, 1, &[i as u8]).unwrap()).unwrap(); }
        assert_eq!(q.push(Message::from_kernel_sender(0, 1, &[9]).unwrap()), Err(Error::Full));
        for i in 0..QUEUE_DEPTH { let m=q.pop().unwrap(); assert_eq!(m.sender, i as u8); assert_eq!(m.bytes[0], i as u8); }
        assert_eq!(q.pop(), Err(Error::Empty));
        assert!(Message::from_kernel_sender(TASKS, 1, &[1]).is_err());
        assert!(Message::from_kernel_sender(0, 0, &[1]).is_err());
        assert!(Message::from_kernel_sender(0, 1, &[0; MESSAGE_BYTES + 1]).is_err());
        for _ in 0..20 { q.push(Message::from_kernel_sender(2, 3, &[4]).unwrap()).unwrap(); assert_eq!(q.pop().unwrap().generation,3); }
    }
    #[test] fn scheduler_round_robin_and_dead_containment() {
        let mut s=[State::Dead; TASKS]; s[1]=State::Runnable; s[4]=State::Runnable;
        assert_eq!(next(&s,1),Ok(4)); assert_eq!(next(&s,4),Ok(1));
        s[4]=State::Blocked; assert_eq!(next(&s,1),Ok(1));
        s[1]=State::Dead; assert_eq!(next(&s,0),Err(Error::Empty));
    }
    #[test] fn copy_bounds_readonly_overflow_and_guards() {
        let ranges=[UserRange{start:0x400000,end:0x401000,writable:false,executable:true},
                    UserRange{start:0x600000,end:0x601000,writable:true,executable:false}];
        assert!(user_buffer(&ranges,0x400000,128,false).is_ok());
        assert!(user_buffer(&ranges,0x400000,1,true).is_err());
        assert!(user_buffer(&ranges,0x600000,128,true).is_ok());
        for (p,n) in [(0,1),(0x5fffff,2),(0x600fff,2),(u64::MAX,2),(0x600000,0),(0x600000,145)] {
            assert!(user_buffer(&ranges,p,n,true).is_err());
        }
    }
}

#[path="pe.rs"]
pub mod pe;


/// Untrusted protocol metadata is reduced to one bounded MMIO span before use.
pub fn framebuffer_span(width:u32,height:u32,pitch:u32,format:u32,base:u64,bytes:u64)->Result<u64,Error>{
    if width!=640||height!=480||!(640..=4096).contains(&pitch)||format>1||
        base==0||base%4096!=0{return Err(Error::Invalid);}
    let span=(pitch as u64).checked_mul(height as u64).and_then(|v|v.checked_mul(4)).ok_or(Error::Invalid)?;
    let rounded=span.div_ceil(4096)*4096;
    if rounded>8*1024*1024||rounded>bytes||base.checked_add(rounded).is_none_or(|end|end>0x1_0000_0000){
        return Err(Error::Denied);
    }
    Ok(rounded)
}
#[cfg(test)]
mod display_tests{
    use super::*;
    #[test]fn validated_fixed_framebuffer_span(){
        assert_eq!(framebuffer_span(640,480,640,0,0x80000000,1228800),Ok(1228800));
        assert!(framebuffer_span(640,480,640,1,0x80000000,1228800).is_ok());
    }
    #[test]fn malformed_framebuffer_metadata(){
        for (w,h,p,f,b,n) in [(800,480,800,0,0x80000000,2000000),(640,600,640,0,0x80000000,2000000),
            (640,480,639,0,0x80000000,2000000),(640,480,u32::MAX,0,0x80000000,u64::MAX),
            (640,480,640,2,0x80000000,2000000),(640,480,640,3,0x80000000,2000000),
            (640,480,640,0,0,2000000),(640,480,640,0,0x80000001,2000000),
            (640,480,640,0,0xfffff000,2000000),(640,480,640,0,0x80000000,100)]{
            assert!(framebuffer_span(w,h,p,f,b,n).is_err());
        }
    }
}


/// Timer evidence distinguishes the two all-register fixture phases by R15.
/// Each phase is credited only while all other sentinel state is live.
pub fn context_phase(rax:u64,rcx:u64,r15:u64,others:bool,flags:u64,mxcsr:u32,simd:bool)->u8{
    if !others||flags&0x401!=0x401||mxcsr!=0x3f80||!simd{return 0;}
    if r15==0xdddd&&rax==0xeeee&&(1..=10_000_000).contains(&rcx){1}
    else if r15==0xddde&&rcx==0xffff&&(1..=10_000_000).contains(&rax){2}
    else{0}
}
#[cfg(test)]
mod context_tests{
    use super::*;
    #[test]fn timer_phase_evidence_cannot_credit_both_phases(){
        assert_eq!(context_phase(0xeeee,100,0xdddd,true,0x401,0x3f80,true),1);
        assert_eq!(context_phase(100,0xffff,0xddde,true,0x401,0x3f80,true),2);
        assert_eq!(context_phase(0xeeee,0xffff,0xdddd,true,0x401,0x3f80,true),1);
        assert_eq!(context_phase(0xeeee,0xffff,0xddde,true,0x401,0x3f80,true),2);
        assert_eq!(context_phase(0xeeee,0,0xdddd,true,0x401,0x3f80,true),0);
        assert_eq!(context_phase(0xeeee,100,0xdddd,false,0x401,0x3f80,true),0);
        assert_eq!(context_phase(0xeeee,100,0xdddd,true,0x400,0x3f80,true),0);
        assert_eq!(context_phase(0xeeee,100,0xdddd,true,0x401,0x1f80,true),0);
        assert_eq!(context_phase(0xeeee,100,0xdddd,true,0x401,0x3f80,false),0);
    }
}

#[cfg(test)]
mod queue_quota_tests{
    use super::*;
    #[test]fn one_sender_cannot_fill_shared_service_queue(){
        let mut q=Queue::with_sender_limit(2).unwrap();
        let a=Message::from_kernel_sender(0,1,&[1]).unwrap();
        let b=Message::from_kernel_sender(10,1,&[2]).unwrap();
        q.push(a).unwrap();q.push(a).unwrap();assert_eq!(q.push(a),Err(Error::Full));
        q.push(b).unwrap();q.push(b).unwrap();
        assert!(Queue::with_sender_limit(0).is_err());
        assert!(Queue::with_sender_limit(5).is_err());
    }
}
