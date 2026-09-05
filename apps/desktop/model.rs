//! Pure application/session models for Desktop-v0. No device or kernel authority.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Text { pub bytes: [u8; 48], pub len: usize }
impl Text {
    pub const EMPTY: Self = Self { bytes: [0; 48], len: 0 };
    pub fn new(value: &[u8]) -> Self {
        let mut out=Self::EMPTY;
        for &b in value.iter().take(48) { out.bytes[out.len]=if (32..=126).contains(&b){b}else{b'?'}; out.len+=1; }
        out
    }
    pub fn append(&mut self, value: &[u8]) {
        for &b in value { if self.len==48 {break;} self.bytes[self.len]=if (32..=126).contains(&b){b}else{b'?'}; self.len+=1; }
    }
    pub fn as_bytes(&self)->&[u8] { &self.bytes[..self.len] }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct View { pub lines: [Text; 6] }
impl View {
    pub const EMPTY: Self=Self { lines: [Text::EMPTY;6] };
    pub fn line(&mut self, index:usize, value:&[u8]) { if index<6 {self.lines[index]=Text::new(value);} }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Windows { pub order:[u8;3], pub count:usize, pub light:bool, pub stopped:bool }
impl Windows {
    pub const fn new()->Self { Self{order:[0;3],count:0,light:false,stopped:false} }
    pub fn focus(&self)->Option<u8> { if self.count==0{None}else{Some(self.order[self.count-1])} }
    pub fn hide(&mut self,role:u8) {
        if let Some(i)=self.order[..self.count].iter().position(|&r|r==role) {
            for j in i..self.count-1 {self.order[j]=self.order[j+1];}
            self.count-=1;self.order[self.count]=0;
        }
    }
    pub fn show(&mut self,role:u8)->bool {
        if !(4..=6).contains(&role)||(role==6&&self.stopped) {return false;}
        self.hide(role);
        self.order[self.count]=role;self.count+=1;true
    }
    /// Caller must have received kernel Stale for the Terminal endpoint.
    pub fn terminal_stale(&mut self) {self.hide(6);self.stopped=true;}
    pub fn wire(&self)->[u8;128] {
        let mut m=[0;128];m[0]=0x10;m[1]=self.count as u8;
        m[2..5].copy_from_slice(&self.order);m[5]=self.light as u8;m[6]=self.stopped as u8;m
    }
    pub fn decode(m:&[u8;128])->Option<Self> {
        let n=m[1] as usize;
        if m[0]!=0x10||n>3||m[5]>1||m[6]>1||m[7..].iter().any(|&x|x!=0) {return None;}
        if m[2+n..5].iter().any(|&x|x!=0) {return None;}
        let mut order=[0;3];
        for i in 0..n {
            let r=m[2+i];
            if !(4..=6).contains(&r)||order[..i].contains(&r)||(r==6&&m[6]!=0) {return None;}
            order[i]=r;
        }
        Some(Self{order,count:n,light:m[5]!=0,stopped:m[6]!=0})
    }
}
pub fn key_allowed(k:u8)->bool { (32..=126).contains(&k)||matches!(k,8|13|27|0x81..=0x85) }
pub fn key_wire(k:u8)->Option<[u8;128]> {
    if !key_allowed(k) {return None;} let mut m=[0;128];m[0]=1;m[1]=k;Some(m)
}
pub fn key_decode(m:&[u8;128])->Option<u8> {
    if m[0]!=1||!key_allowed(m[1])||m[2..].iter().any(|&b|b!=0){None}else{Some(m[1])}
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edit { Changed, Submit, Full, Ignored }
pub struct Editor { pub bytes:[u8;64], pub len:usize }
impl Editor {
    pub const fn new()->Self {Self{bytes:[0;64],len:0}}
    pub fn clear(&mut self) {self.bytes=[0;64];self.len=0;}
    pub fn key(&mut self,key:u8)->Edit {
        match key {
            13=>Edit::Submit,
            8 if self.len>0=>{self.len-=1;self.bytes[self.len]=0;Edit::Changed},
            32..=126 if self.len<64=>{self.bytes[self.len]=key;self.len+=1;Edit::Changed},
            32..=126=>Edit::Full,
            _=>Edit::Ignored,
        }
    }
    pub fn prompt(&self,view:&mut View) {
        let mut first=Text::new(b"> ");
        first.append(&self.bytes[..self.len.min(46)]);
        view.lines[3]=first;
        view.lines[4]=Text::new(&self.bytes[self.len.min(46)..self.len]);
    }
}
#[derive(Debug,PartialEq,Eq)]
pub enum Command<'a> { Help, List, Read(&'a [u8]), Write(&'a [u8],&'a [u8]), Crash, Invalid }
fn name_ok(name:&[u8])->bool {
    !name.is_empty()&&name.len()<=12&&name!=b"."&&name!=b".."&&
    name.iter().all(|b|b.is_ascii_alphanumeric()||matches!(b,b'.'|b'-'))
}
pub fn command(line:&[u8])->Command<'_> {
    if line.len()>64||line.iter().any(|&b|!(32..=126).contains(&b)){return Command::Invalid;}
    let line=line.trim_ascii();
    if line.eq_ignore_ascii_case(b"help"){return Command::Help;}
    if line.eq_ignore_ascii_case(b"list"){return Command::List;}
    if line.eq_ignore_ascii_case(b"crash"){return Command::Crash;}
    let Some(space)=line.iter().position(|&b|b==b' ') else{return Command::Invalid;};
    let verb=&line[..space];let args=line[space+1..].trim_ascii_start();
    if verb.eq_ignore_ascii_case(b"read")&&name_ok(args){return Command::Read(args);}
    if verb.eq_ignore_ascii_case(b"write"){
        let split=args.iter().position(|&b|b==b' ').unwrap_or(args.len());
        let name=&args[..split];let data=if split==args.len(){&[][..]}else{&args[split+1..]};
        if name_ok(name)&&data.len()<=64 {return Command::Write(name,data);}
    }
    Command::Invalid
}
#[derive(Clone, Copy, Debug,PartialEq,Eq)]
pub struct Names { pub bytes:[[u8;12];4],pub lengths:[usize;4],pub count:usize }
impl Names {
    pub const EMPTY:Self=Self{bytes:[[0;12];4],lengths:[0;4],count:0};
    pub fn decode(reply:&[u8;128])->Option<Self> {
        if reply[0]!=0||reply[1]>4||reply[2]!=0||reply[3]!=0{return None;}
        let mut names=Self::EMPTY;let mut offset=4;
        for i in 0..reply[1] as usize {
            let len=reply[offset] as usize;offset+=1;
            if len==0||len>12||offset+len>reply.len()||!name_ok(&reply[offset..offset+len]){return None;}
            names.bytes[i][..len].copy_from_slice(&reply[offset..offset+len]);names.lengths[i]=len;
            if (0..i).any(|j|names.name(j)==&reply[offset..offset+len]){return None;}
            offset+=len;names.count+=1;
        }
        if reply[offset..].iter().any(|&b|b!=0){return None;} Some(names)
    }
    pub fn name(&self,index:usize)->&[u8] {
        if index>=self.count {&[]}else{&self.bytes[index][..self.lengths[index]]}
    }
    pub fn display(&self)->Text {
        let mut text=Text::EMPTY;
        for i in 0..self.count {if i!=0{text.append(b" ");}text.append(self.name(i));}
        text
    }
}
/// Bound input received while waiting for a storage reply; no unbounded allocation.
pub struct Pending { values:[[u8;128];16],head:usize,len:usize }
impl Pending {
    pub const fn new()->Self {Self{values:[[0;128];16],head:0,len:0}}
    pub fn push(&mut self,m:[u8;128])->bool {
        if self.len==16{return false;}self.values[(self.head+self.len)%16]=m;self.len+=1;true
    }
    pub fn pop(&mut self)->Option<[u8;128]> {
        if self.len==0{return None;}
        let value=self.values[self.head];self.values[self.head]=[0;128];
        self.head=(self.head+1)%16;self.len-=1;Some(value)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn window_focus_hide_reopen_and_stopped() {
        let mut w=Windows::new();assert_eq!(w.focus(),None);
        for r in [4,5,6,4] {assert!(w.show(r));}
        assert_eq!(w.order,[5,6,4]);w.hide(4);assert_eq!(w.focus(),Some(6));
        w.light=true;w.terminal_stale();assert_eq!(w.focus(),Some(5));assert!(!w.show(6));
        assert!(w.show(4));assert!(w.light);assert_eq!(Windows::decode(&w.wire()),Some(w));
    }
    #[test] fn forged_window_state_rejected() {
        let w=Windows::new();let mut m=w.wire();m[1]=4;assert!(Windows::decode(&m).is_none());
        m=w.wire();m[1]=2;m[2]=4;m[3]=4;assert!(Windows::decode(&m).is_none());
        m=w.wire();m[2]=4;assert!(Windows::decode(&m).is_none());
        m=w.wire();m[127]=1;assert!(Windows::decode(&m).is_none());
    }
    #[test] fn input_bounds_backspace_and_submit() {
        let mut e=Editor::new();for _ in 0..64{assert_eq!(e.key(b'a'),Edit::Changed);}
        assert_eq!(e.key(b'b'),Edit::Full);assert_eq!(e.len,64);
        assert_eq!(e.key(8),Edit::Changed);assert_eq!(e.key(b'b'),Edit::Changed);
        assert_eq!(e.bytes[63],b'b');assert_eq!(e.key(13),Edit::Submit);
        assert_eq!(e.key(0x81),Edit::Ignored);e.clear();assert_eq!(e.key(8),Edit::Ignored);
    }
    #[test] fn parser_is_generic_and_bounded() {
        assert_eq!(command(b"write note abcdefgh"),Command::Write(b"note",b"abcdefgh"));
        assert_eq!(command(b"read note"),Command::Read(b"note"));
        assert_eq!(command(b"HELP"),Command::Help);assert_eq!(command(b"crash"),Command::Crash);
        for s in [b"read ../x".as_slice(),b"write /disk nope",b"read x extra",b"foo",b"read "] {
            assert_eq!(command(s),Command::Invalid);
        }
        assert_eq!(command(&[b'a';65]),Command::Invalid);
    }
    #[test] fn pending_input_fifo_bound() {
        let mut p=Pending::new();
        for i in 0..16{assert!(p.push(key_wire(b'a'+i).unwrap()));}
        assert!(!p.push([0;128]));
        for i in 0..16{assert_eq!(p.pop().unwrap()[1],b'a'+i);}
        assert_eq!(p.pop(),None);
    }
    #[test] fn text_and_keyboard_wire_validation() {
        assert_eq!(Text::new(&[b'x';49]).len,48);
        let mut m=key_wire(b'a').unwrap();assert_eq!(key_decode(&m),Some(b'a'));
        m[127]=1;assert_eq!(key_decode(&m),None);assert!(key_wire(0).is_none());
    }
    #[test] fn list_reply_does_not_trust_counts_names_or_padding() {
        let mut m=[0;128];m[1]=1;m[4]=4;m[5..9].copy_from_slice(b"note");
        assert_eq!(Names::decode(&m).unwrap().name(0),b"note");
        m[127]=1;assert!(Names::decode(&m).is_none());m[127]=0;m[4]=255;
        assert!(Names::decode(&m).is_none());m[4]=4;m[1]=255;assert!(Names::decode(&m).is_none());
    }
}
