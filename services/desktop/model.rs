//! Desktop-v0 bounded service models. No device I/O or host execution.
#[path="../../apps/desktop/model.rs"]
pub mod apps;
#[path="../platform/model.rs"]
pub mod storage;
use apps::{View,Text,Windows};
#[derive(Clone,Copy)]
struct Surface {
    committed:View, staged:View, committed_version:u32, staged_version:u32,
    count:u8, seen:u8, open:bool,
}
impl Surface {
    const EMPTY:Self=Self{committed:View::EMPTY,staged:View::EMPTY,committed_version:0,
        staged_version:0,count:0,seen:0,open:false};
    fn message(&mut self,m:&[u8;128])->Result<bool,()> {
        let version=u32::from_le_bytes(m[4..8].try_into().map_err(|_|())?);
        match m[0] {
            0x20=>{
                if m[1]>6||m[2]!=0||m[3]!=0||m[8..].iter().any(|&b|b!=0)||
                    version==0||version<=self.committed_version||version<=self.staged_version{return Err(());}
                self.staged=View::EMPTY;self.staged_version=version;self.count=m[1];self.seen=0;self.open=true;Ok(false)
            }
            0x21=>{
                let row=m[1] as usize;let len=m[2] as usize;
                if !self.open||version!=self.staged_version||row>=self.count as usize||len>48||m[3]!=0||
                    self.seen&(1<<row)!=0||m[8+len..].iter().any(|&b|b!=0)||
                    m[8..8+len].iter().any(|&b|!(32..=126).contains(&b)){return Err(());}
                self.staged.lines[row]=Text::new(&m[8..8+len]);self.seen|=1<<row;Ok(false)
            }
            0x22=>{
                if !self.open||version!=self.staged_version||m[1..4].iter().any(|&b|b!=0)||
                    m[8..].iter().any(|&b|b!=0)||self.seen!=(1u8<<self.count)-1{return Err(());}
                self.committed=self.staged;self.committed_version=version;self.open=false;Ok(true)
            }
            _=>Err(()),
        }
    }
}
pub struct Compositor { pub windows:Windows,surfaces:[Surface;3] }
impl Compositor {
    pub const fn new()->Self {Self{windows:Windows::new(),surfaces:[Surface::EMPTY;3]}}
    pub fn view(&self,role:u8)->Option<&View> {self.surfaces.get(role.checked_sub(4)? as usize).map(|s|&s.committed)}
    pub fn apply(&mut self,sender:u64,generation:u32,m:&[u8;128])->Result<bool,()> {
        if generation!=1{return Err(());}
        if sender==0 {
            self.windows=Windows::decode(m).ok_or(())?;return Ok(true);
        }
        if !(4..=6).contains(&sender){return Err(());}
        self.surfaces[(sender-4) as usize].message(m)
    }
}
pub fn begin(version:u32)->[u8;128] {
    let mut m=[0;128];m[0]=0x20;m[1]=6;m[4..8].copy_from_slice(&version.to_le_bytes());m
}
pub fn line(version:u32,index:usize,text:&Text)->Option<[u8;128]> {
    if index>=6||text.len>48{return None;}let mut m=[0;128];m[0]=0x21;m[1]=index as u8;m[2]=text.len as u8;
    m[4..8].copy_from_slice(&version.to_le_bytes());m[8..8+text.len].copy_from_slice(text.as_bytes());Some(m)
}
pub fn commit(version:u32)->[u8;128] {let mut m=[0;128];m[0]=0x22;m[4..8].copy_from_slice(&version.to_le_bytes());m}
/// Separate profile adapter: Platform's Store and its per-owner contract stay unchanged.
pub struct DesktopStore { inner:storage::Store }
impl DesktopStore {
    pub fn new()->Self {
        let mut value=Self{inner:storage::Store::new()};
        let create=storage::request(storage::CREATE,b"welcome",b"").unwrap();
        let write=storage::request(storage::WRITE,b"welcome",b"RAR OS ALPHA").unwrap();
        assert_eq!(value.process(4,1,&create)[0],storage::OK);
        assert_eq!(value.process(4,1,&write)[0],storage::OK);value
    }
    pub fn process(&mut self,sender:u64,generation:u32,m:&[u8;128])->[u8;128] {
        if !matches!(sender,4|6)||generation!=1 {let mut out=[0;128];out[0]=storage::INVALID;return out;}
        self.inner.process(storage::Owner{task:4,generation:1},m)
    }
}
pub struct Keyboard { extended:bool,skip:u8,down:[bool;256],caps:bool }
impl Keyboard {
    pub const fn new()->Self {Self{extended:false,skip:0,down:[false;256],caps:false}}
    pub fn reset(&mut self) {self.extended=false;self.skip=0;self.down=[false;256];}
    pub fn feed(&mut self,byte:u8)->Option<u8> {
        if self.skip>0 {self.skip-=1;return None;}
        if byte==0xe1 {self.extended=false;self.skip=5;return None;}
        if byte==0xe0 {self.extended=true;return None;}
        if byte==0||byte==255 {self.reset();return None;}
        let ext=self.extended;self.extended=false;
        let scan=byte&0x7f;let index=scan as usize+if ext{128}else{0};
        if byte&0x80!=0 {self.down[index]=false;return None;}
        if self.down[index] {return None;}self.down[index]=true;
        if ext {return match scan{0x48=>Some(0x84),0x50=>Some(0x85),_=>None};}
        if scan==0x3a {self.caps=!self.caps;return None;}
        let shift=self.down[0x2a]||self.down[0x36];
        let byte=match scan {
            1=>27,14=>8,28=>13,57=>b' ',59=>0x81,60=>0x82,61=>0x83,
            2..=11=>b"1234567890"[(scan-2) as usize],
            16..=25=>b"qwertyuiop"[(scan-16) as usize],
            30..=38=>b"asdfghjkl"[(scan-30) as usize],
            44..=50=>b"zxcvbnm"[(scan-44) as usize],
            12=>b'-',13=>b'=',26=>b'[',27=>b']',39=>b';',40=>b'\'',41=>b'`',
            43=>b'\\',51=>b',',52=>b'.',53=>b'/',_=>return None,
        };
        if byte.is_ascii_lowercase() {return Some(if shift^self.caps{byte.to_ascii_uppercase()}else{byte});}
        if shift {
            return Some(match byte {
                b'1'=>b'!',b'2'=>b'@',b'3'=>b'#',b'4'=>b'$',b'5'=>b'%',
                b'6'=>b'^',b'7'=>b'&',b'8'=>b'*',b'9'=>b'(',b'0'=>b')',
                b'-'=>b'_',b'='=>b'+',b'['=>b'{',b']'=>b'}',b';'=>b':',
                39=>34,96=>b'~',92=>b'|',b','=>b'<',b'.'=>b'>',b'/'=>b'?',_=>byte,
            });
        }
        Some(byte)
    }
}
#[cfg(test)] mod tests {
    use super::*;
    fn put(c:&mut Compositor,role:u64,version:u32,text:&[u8]) {
        c.apply(role,1,&begin(version)).unwrap();
        for i in 0..6 {c.apply(role,1,&line(version,i,&Text::new(if i==0{text}else{b""})).unwrap()).unwrap();}
        assert_eq!(c.apply(role,1,&commit(version)),Ok(true));
    }
    #[test] fn surfaces_commit_atomically_by_kernel_sender() {
        let mut c=Compositor::new();put(&mut c,4,1,b"FILES");put(&mut c,6,1,b"TERM");
        assert_eq!(c.view(4).unwrap().lines[0].as_bytes(),b"FILES");
        c.apply(4,1,&begin(2)).unwrap();
        assert!(c.apply(4,1,&commit(2)).is_err());
        assert_eq!(c.view(4).unwrap().lines[0].as_bytes(),b"FILES");
        assert!(c.apply(5,1,&Windows::new().wire()).is_err());
        assert!(c.apply(2,1,&begin(1)).is_err());assert!(c.apply(4,2,&begin(3)).is_err());
    }
    #[test] fn malformed_and_duplicate_surface_lines_preserve_commit() {
        let mut c=Compositor::new();put(&mut c,4,1,b"OLD");c.apply(4,1,&begin(2)).unwrap();
        let l=line(2,0,&Text::new(b"NEW")).unwrap();c.apply(4,1,&l).unwrap();
        assert!(c.apply(4,1,&l).is_err());let mut bad=l;bad[1]=255;assert!(c.apply(4,1,&bad).is_err());
        bad=l;bad[2]=255;assert!(c.apply(4,1,&bad).is_err());bad=l;bad[127]=1;assert!(c.apply(4,1,&bad).is_err());
        assert!(c.apply(4,1,&begin(1)).is_err());assert!(c.apply(4,1,&begin(0)).is_err());
        assert_eq!(c.view(4).unwrap().lines[0].as_bytes(),b"OLD");
    }
    #[test] fn shared_workspace_is_explicit_and_other_apps_are_denied() {
        let mut s=DesktopStore::new();
        let create=storage::request(storage::CREATE,b"note",b"").unwrap();
        let write=storage::request(storage::WRITE,b"note",b"typed").unwrap();
        let read=storage::request(storage::READ,b"note",b"").unwrap();
        assert_eq!(s.process(6,1,&create)[0],0);assert_eq!(s.process(6,1,&write)[0],0);
        assert_eq!(&s.process(4,1,&read)[16..21],b"typed");
        for role in [0,1,2,3,5,7,255] {assert_eq!(s.process(role,1,&read)[0],storage::INVALID);}
        assert_eq!(s.process(4,2,&read)[0],storage::INVALID);
    }
    #[test] fn key_make_break_extended_and_repeat() {
        let mut k=Keyboard::new();assert_eq!(k.feed(0x1e),Some(b'a'));assert_eq!(k.feed(0x1e),None);
        assert_eq!(k.feed(0x9e),None);assert_eq!(k.feed(0x1e),Some(b'a'));
        k.feed(0xe0);assert_eq!(k.feed(0x50),Some(0x85));k.feed(0xe0);assert_eq!(k.feed(0xd0),None);
        assert_eq!(k.feed(0x3b),Some(0x81));
    }
    #[test] fn shift_caps_pause_and_reset_are_bounded() {
        let mut k=Keyboard::new();k.feed(0x2a);assert_eq!(k.feed(0x1e),Some(b'A'));
        k.feed(0x9e);k.feed(0xaa);assert_eq!(k.feed(0x1e),Some(b'a'));
        k.reset();k.feed(0xe1);for b in [0x1d,0x45,0xe1,0x9d,0xc5]{assert_eq!(k.feed(b),None);}
        assert_eq!(k.feed(0x1e),Some(b'a'));k.reset();k.feed(0x3a);assert_eq!(k.feed(0x1e),Some(b'A'));
    }
}
