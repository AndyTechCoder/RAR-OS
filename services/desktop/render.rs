//! RAR-owned provisional bitmap compositor. Only role 3 owns this mapping.
use crate::{abi::Boot,check};
use crate::services::Compositor;
fn glyph(byte:u8)->[u8;7] {match byte.to_ascii_uppercase(){48=>[14,17,19,21,25,17,14],
49=>[4,12,4,4,4,4,14],
50=>[14,17,1,2,4,8,31],
51=>[30,1,1,14,1,1,30],
52=>[2,6,10,18,31,2,2],
53=>[31,16,16,30,1,1,30],
54=>[14,16,16,30,17,17,14],
55=>[31,1,2,4,8,8,8],
56=>[14,17,17,14,17,17,14],
57=>[14,17,17,15,1,1,14],
65=>[14,17,17,31,17,17,17],
66=>[30,17,17,30,17,17,30],
67=>[15,16,16,16,16,16,15],
68=>[30,17,17,17,17,17,30],
69=>[31,16,16,30,16,16,31],
70=>[31,16,16,30,16,16,16],
71=>[15,16,16,23,17,17,15],
72=>[17,17,17,31,17,17,17],
73=>[31,4,4,4,4,4,31],
74=>[7,2,2,2,18,18,12],
75=>[17,18,20,24,20,18,17],
76=>[16,16,16,16,16,16,31],
77=>[17,27,21,21,17,17,17],
78=>[17,25,21,19,17,17,17],
79=>[14,17,17,17,17,17,14],
80=>[30,17,17,30,16,16,16],
81=>[14,17,17,17,21,18,13],
82=>[30,17,17,30,20,18,17],
83=>[15,16,16,14,1,1,30],
84=>[31,4,4,4,4,4,4],
85=>[17,17,17,17,17,17,14],
86=>[17,17,17,17,17,10,4],
87=>[17,17,17,21,21,21,10],
88=>[17,17,10,4,10,17,17],
89=>[17,17,10,4,4,4,4],
90=>[31,1,2,4,8,16,31],
32=>[0,0,0,0,0,0,0],
45=>[0,0,0,31,0,0,0],
46=>[0,0,0,0,0,6,6],
58=>[0,6,6,0,6,6,0],
47=>[1,2,2,4,8,8,16],
62=>[16,8,4,2,4,8,16],
63=>[14,17,1,2,4,0,4],
_=>[14,17,1,2,4,0,4]}}
type Color=(u32,u32,u32);
struct Canvas<'a>{boot:&'a Boot}
impl Canvas<'_> {
    fn rect(&self,x:usize,y:usize,w:usize,h:usize,c:Color) {
        // All primitive callers are compositor policy. Still clip every write.
        let end_x=x.saturating_add(w).min(640);let end_y=y.saturating_add(h).min(480);
        let pixel=if self.boot.format==0{c.0|(c.1<<8)|(c.2<<16)}else{c.2|(c.1<<8)|(c.0<<16)};
        for yy in y.min(480)..end_y {for xx in x.min(640)..end_x {
            // Kernel validates pitch/span before granting this role-only RW+NX
            // mapping. Coordinates are clipped; no app supplies a pointer.
            unsafe{((self.boot.framebuffer as usize+(yy*self.boot.pitch as usize+xx)*4) as *mut u32).write_volatile(pixel);}
        }}
    }
    fn text(&self,x:usize,y:usize,value:&[u8],c:Color,scale:usize) {
        for (i,&byte) in value.iter().take(48).enumerate() {
            for (yy,row) in glyph(byte).iter().enumerate() {for xx in 0..5 {
                if row&(1<<(4-xx))!=0{self.rect(x+(i*6+xx)*scale,y+yy*scale,scale,scale,c);}
            }}
        }
    }
}
pub fn draw(boot:&Boot,state:&Compositor) {
    check(boot.framebuffer==0x800000&&boot.width==640&&boot.height==480&&
        boot.pitch>=640&&boot.pitch<=4096&&boot.format<=1&&boot.caps[crate::abi::FRAMEBUFFER]!=0);
    let c=Canvas{boot};let light=state.windows.light;let focus=state.windows.focus();
    let bg=if light{(224,233,240)}else{(12,18,30)};
    let panel=if light{(250,252,255)}else{(19,29,45)};
    let ink=if light{(24,36,52)}else{(230,240,250)};
    let content=if light{(255,255,255)}else{(24,36,52)};
    let accent=(44,110,160);let white=(255,255,255);
    c.rect(0,0,640,480,bg);c.rect(0,0,640,40,panel);
    c.text(16,12,b"RAR OS",ink,2);c.text(454,16,b"USABLE ALPHA",ink,1);
    c.text(28,60,b"YOUR RAR WORKSPACE",ink,2);
    c.text(28,86,b"F1 FILES   F2 SETTINGS   F3 TERMINAL",ink,1);
    c.text(28,104,b"KEYBOARD FIRST - CLOUD DEVELOPMENT ALPHA",ink,1);
    c.text(28,126,b"RAM FILES ARE TEMPORARY",ink,1);
    for &role in &state.windows.order[..state.windows.count] {
        let (x,y,title):(usize,usize,&[u8])=match role{4=>(24,152,b"FILES"),5=>(44,170,b"SETTINGS"),6=>(64,188,b"TERMINAL"),_=>continue};
        let active=focus==Some(role);let header=if active{white}else{ink};
        c.rect(x+4,y+4,548,232,(8,12,20));c.rect(x,y,548,232,content);
        c.rect(x,y,548,30,if active{accent}else{panel});
        c.text(x+12,y+8,title,header,2);c.text(x+426,y+11,b"ESC CLOSE",header,1);
        if let Some(view)=state.view(role) {for (row,line) in view.lines.iter().enumerate(){
            c.text(x+14,y+48+row*28,line.as_bytes(),ink,1);
        }}
    }
    c.rect(0,440,640,40,panel);
    for (x,label,role) in [(16,b"F1 FILES".as_slice(),4),(224,b"F2 SETTINGS".as_slice(),5),(432,b"F3 TERMINAL".as_slice(),6)] {
        let active=focus==Some(role);c.rect(x,448,192,24,if active{accent}else{content});
        c.text(x+12,456,label,if active{white}else{ink},1);
    }
    if state.windows.stopped {
        c.rect(364,46,260,18,(160,54,54));c.text(374,52,b"TERMINAL STOPPED - FILES SAFE",white,1);
    }
}
