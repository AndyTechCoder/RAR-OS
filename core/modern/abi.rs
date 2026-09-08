//! Private Modern-v1 bootstrap/syscall types. Not a stable SDK or active ABI.
#![forbid(unsafe_code)]
pub const BOOT_ADDRESS:usize=0x700000;
pub const MAGIC:u64=u64::from_le_bytes(*b"RARMOD01");
pub const VERSION:u64=1;
pub const BOOT_BYTES:u64=368;
pub const ENVELOPE_BYTES:u64=152;
pub const ACTIVE:u64=0;
pub const TRIAL:u64=1;
pub const YIELD:u64=0;
pub const SEND:u64=1;
pub const RECEIVE:u64=2;
pub const PORT_READ:u64=3;
pub const REPORT:u64=4;
pub const EXIT:u64=5;
pub const TICKS:u64=6;
pub const DEVICE:u64=7;
pub const TRIAL_READY:u64=8;
pub const SELF_RECV:usize=0;
pub const SHELL:usize=1;
pub const COMPOSITOR:usize=2;
pub const STORAGE:usize=3;
pub const FILES:usize=4;
pub const SETTINGS:usize=5;
pub const TERMINAL:usize=6;
pub const INPUT:usize=7;
pub const FRAMEBUFFER:usize=8;
pub const HEALTH:usize=9;
pub const MANAGER:usize=10;
pub const DEVICE_CAP:usize=11;
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Boot {
    pub magic:u64,pub version:u64,pub bytes:u64,pub role:u64,pub phase:u64,
    pub generation:u64,pub entry:u64,pub kernel_probe:u64,pub peer_probe:u64,
    pub framebuffer:u64,pub width:u64,pub height:u64,pub pitch:u64,pub format:u64,
    pub health_token:u64,pub caps:[u64;12],pub peers:[u64;10],
    pub device_sectors:u64,pub device_serial:[u8;20],pub device_model:[u8;40],
    pub reserved:[u8;4],
}
impl Boot {
    pub const EMPTY:Self=Self {magic:0,version:0,bytes:0,role:0,phase:0,generation:0,
        entry:0,kernel_probe:0,peer_probe:0,framebuffer:0,width:0,height:0,pitch:0,
        format:0,health_token:0,caps:[0;12],peers:[0;10],device_sectors:0,
        device_serial:[0;20],device_model:[0;40],reserved:[0;4]};
}
#[repr(C)]
#[derive(Clone,Copy)]
pub struct Envelope {pub sender:u64,pub generation:u64,pub length:u64,pub bytes:[u8;128]}
impl Envelope {
    pub const EMPTY:Self=Self {sender:0,generation:0,length:0,bytes:[0;128]};
}
const _: [();BOOT_BYTES as usize]=[();core::mem::size_of::<Boot>()];
const _: [();ENVELOPE_BYTES as usize]=[();core::mem::size_of::<Envelope>()];
fn active_mask(role:u64)->Option<u16> {
    match role {0=>Some(0x75),1=>Some(0x851),2=>Some(0x82),3=>Some(0x101),
        4|6=>Some(0x0d),5=>Some(7),8=>Some(0x401),9=>Some(0x801),15=>Some(0),_=>None}
}
fn text(value:&[u8])->bool {
    value.iter().all(|b|(0x20..=0x7e).contains(b))&&value.iter().any(|b|*b!=b' ')
}
/// Redundant receiver-side shape checks, not authentication. The kernel alone
/// writes this read-only mapping and derives grants, identities and expectations.
pub fn valid_boot(b:&Boot)->bool {
    if b.magic!=MAGIC||b.version!=VERSION||b.bytes!=BOOT_BYTES||b.generation==0||
        !(0x400000..0x500000).contains(&b.entry)||b.reserved!=[0;4]||b.peers[7]!=0||
        b.kernel_probe!=0||b.peer_probe!=0 {return false;}
    let mask=match b.phase {
        ACTIVE=>{
            let Some(mask)=active_mask(b.role) else{return false;};
            if b.health_token!=0||(b.role!=15&&b.peers[b.role as usize]!=b.generation) {return false;}
            mask
        },
        TRIAL if b.role==5&&b.health_token!=0=>1<<HEALTH,
        _=>return false,
    };
    for (index,handle) in b.caps.iter().enumerate() {
        let expected=mask&(1<<index)!=0;
        if expected {
            if handle>>32==0||(*handle as u32)as usize!=index+1 {return false;}
        } else if *handle!=0 {return false;}
    }
    if b.phase==ACTIVE&&b.role==3 {
        if b.framebuffer!=0x800000||b.width!=640||b.height!=480||
            !(640..=4096).contains(&b.pitch)||b.format>1 {return false;}
    } else if [b.framebuffer,b.width,b.height,b.pitch,b.format]!=[0;5] {return false;}
    if b.phase==ACTIVE&&matches!(b.role,1|9) {
        if b.device_sectors==0||b.device_sectors>=1<<28||
            !text(&b.device_serial)||!text(&b.device_model) {return false;}
    } else if b.device_sectors!=0||b.device_serial!=[0;20]||b.device_model!=[0;40] {return false;}
    true
}
/// Receiver-side consistency only, never kernel activation authority.
/// The kernel publishes ACTIVE while the healthy candidate is unscheduled.
pub fn valid_trial_activation(trial:&Boot,active:&Boot)->bool {
    valid_boot(trial)&&trial.phase==TRIAL&&valid_boot(active)&&active.phase==ACTIVE&&
        active.role==5&&active.generation==trial.generation&&active.entry==trial.entry
}
/// DEVICE args: caller-local handle, operation, value, zero. No port or device ID.
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum DeviceOp {
    Status,Count(u8),LbaLow(u8),LbaMid(u8),LbaHigh(u8),Head(u8),
    Identify,Read,Write,Flush,ReadWord,WriteWord(u16),
}
pub fn device_op(operation:u64,value:u64,extra:u64)->Option<DeviceOp> {
    if extra!=0 {return None;}
    match operation {
        0 if value==0=>Some(DeviceOp::Status),
        1 if value<=1=>Some(DeviceOp::Count(value as u8)),
        2 if value<=255=>Some(DeviceOp::LbaLow(value as u8)),
        3 if value<=255=>Some(DeviceOp::LbaMid(value as u8)),
        4 if value<=255=>Some(DeviceOp::LbaHigh(value as u8)),
        5 if value==0xa0||(0xe0..=0xef).contains(&value)=>Some(DeviceOp::Head(value as u8)),
        6 if value==0=>Some(DeviceOp::Identify),
        7 if value==0=>Some(DeviceOp::Read),
        8 if value==0=>Some(DeviceOp::Write),
        9 if value==0=>Some(DeviceOp::Flush),
        10 if value==0=>Some(DeviceOp::ReadWord),
        11 if value<=65535=>Some(DeviceOp::WriteWord(value as u16)),
        _=>None,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn fixture(role:u64)->Boot {
        let mut b=Boot {magic:MAGIC,version:VERSION,bytes:BOOT_BYTES,role,
            generation:(1<<40)|3,entry:0x401000,..Boot::EMPTY};
        b.peers=[1;10];b.peers[7]=0;if role!=15 {b.peers[role as usize]=b.generation;}
        let slots:&[usize]=match role {0=>&[0,2,4,5,6],1=>&[0,4,6,11],2=>&[1,7],
            3=>&[0,8],4|6=>&[0,2,3],5=>&[0,1,2],8=>&[0,10],9=>&[0,11],15=>&[],_=>panic!()};
        for &slot in slots {b.caps[slot]=(1<<32)|(slot as u64+1);}
        if role==3 {b.framebuffer=0x800000;b.width=640;b.height=480;b.pitch=640;}
        if matches!(role,1|9) {b.device_sectors=14;b.device_serial=[b'S';20];b.device_model=[b'M';40];}
        b
    }
    #[test] fn exact_layout_without_implicit_padding() {
        assert_eq!(core::mem::size_of::<Boot>(),368);assert_eq!(core::mem::align_of::<Boot>(),8);
        assert_eq!(core::mem::offset_of!(Boot,caps),120);
        assert_eq!(core::mem::offset_of!(Boot,peers),216);
        assert_eq!(core::mem::offset_of!(Boot,device_sectors),296);
        assert_eq!(core::mem::offset_of!(Boot,reserved),364);
        assert_eq!(core::mem::size_of::<Envelope>(),152);
        assert_eq!(core::mem::offset_of!(Envelope,bytes),24);
    }
    #[test] fn active_role_capability_graph_and_full_incarnations() {
        for role in [0,1,2,3,4,5,6,8,9,15] {
            let b=fixture(role);assert!(valid_boot(&b));
            for slot in 0..12 {
                let mut bad=b;
                if bad.caps[slot]==0 {bad.caps[slot]=(1<<32)|(slot as u64+1);}
                else {bad.caps[slot]=0;}
                assert!(!valid_boot(&bad));
                if b.caps[slot]!=0 {
                    let mut bad=b;bad.caps[slot]&=0xffff_ffff;assert!(!valid_boot(&bad));
                    let mut bad=b;bad.caps[slot]^=1;assert!(!valid_boot(&bad));
                }
            }
            if role!=15 {let mut bad=b;bad.peers[role as usize]=3;assert!(!valid_boot(&bad));}
        }
    }
    #[test] fn bootstrap_refuses_version_shape_and_cross_role_resources() {
        let b=fixture(4);
        for field in 0..11 {
            let mut bad=b;
            match field {0=>bad.magic=0,1=>bad.version=2,2=>bad.bytes=176,3=>bad.generation=0,
                4=>bad.reserved[0]=1,5=>bad.phase=2,6=>bad.health_token=1,
                7=>bad.peers[7]=1,8=>bad.entry=0x500000,9=>bad.kernel_probe=0x2000000,
                _=>bad.peer_probe=0x600000}
            assert!(!valid_boot(&bad));
        }
        for role in [7,10,14,16,u64::MAX] {let mut bad=b;bad.role=role;assert!(!valid_boot(&bad));}
        let mut bad=b;bad.device_sectors=1;assert!(!valid_boot(&bad));
        let mut bad=b;bad.device_serial[0]=b'A';assert!(!valid_boot(&bad));
        let mut bad=b;bad.framebuffer=0x800000;assert!(!valid_boot(&bad));
        let mut bad=fixture(3);bad.format=2;assert!(!valid_boot(&bad));
        let mut bad=fixture(3);bad.pitch=4097;assert!(!valid_boot(&bad));
        let mut bad=fixture(1);bad.device_sectors=1<<28;assert!(!valid_boot(&bad));
        for byte in [0,31,127,255] {
            let mut bad=fixture(9);bad.device_serial[0]=byte;assert!(!valid_boot(&bad));
        }
        let mut bad=fixture(1);bad.device_model=[b' ';40];assert!(!valid_boot(&bad));
    }
    #[test] fn trial_only_has_one_shot_health_material() {
        let mut b=fixture(5);b.phase=TRIAL;b.health_token=7;b.caps=[0;12];
        b.caps[HEALTH]=(1<<32)|(HEALTH as u64+1);b.peers[5]=1;
        assert!(valid_boot(&b));
        for slot in 0..12 {if slot!=HEALTH {
            let mut bad=b;bad.caps[slot]=(1<<32)|(slot as u64+1);assert!(!valid_boot(&bad));
        }}
        let mut bad=b;bad.health_token=0;assert!(!valid_boot(&bad));
        let mut bad=b;bad.role=1;assert!(!valid_boot(&bad));
        let mut bad=b;bad.device_sectors=1;assert!(!valid_boot(&bad));
    }
    #[test] fn trial_activation_requires_fresh_active_grants_and_same_full_identity() {
        let active=fixture(5);
        let mut trial=active;trial.phase=TRIAL;trial.health_token=17;trial.caps=[0;12];
        trial.caps[HEALTH]=(1<<32)|(HEALTH as u64+1);trial.peers[5]=1;
        assert!(valid_trial_activation(&trial,&active));
        assert!(!valid_trial_activation(&trial,&trial));
        assert!(!valid_trial_activation(&active,&active));
        let mut bad=active;bad.generation+=1;bad.peers[5]=bad.generation;
        assert!(valid_boot(&bad));assert!(!valid_trial_activation(&trial,&bad));
        let mut bad=active;bad.entry+=1;
        assert!(valid_boot(&bad));assert!(!valid_trial_activation(&trial,&bad));
        let mut bad=active;bad.peers[5]=active.generation as u32 as u64;
        assert!(!valid_trial_activation(&trial,&bad));
        let mut bad=active;bad.caps[HEALTH]=trial.caps[HEALTH];
        assert!(!valid_trial_activation(&trial,&bad));
        let mut bad=active;bad.caps[COMPOSITOR]=0;
        assert!(!valid_trial_activation(&trial,&bad));
        assert!(!valid_trial_activation(&trial,&fixture(4)));
    }
    #[test] fn device_allowlist_rejects_truncation_slave_and_extra_arguments() {
        for operation in 0..12 {
            for value in [0,1,255,256,65535,65536,u64::MAX] {
                let allowed=match operation {0|6..=10=>value==0,1=>value<=1,
                    2..=4=>value<=255,5=>value==0xa0||(0xe0..=0xef).contains(&value),
                    11=>value<=65535,_=>false};
                assert_eq!(device_op(operation,value,0).is_some(),allowed);
                assert_eq!(device_op(operation,value,1),None);
            }
        }
        for head in 0..=255 {
            assert_eq!(device_op(5,head,0).is_some(),head==0xa0||(0xe0..=0xef).contains(&head));
        }
        for opcode in [12,255,65536,u64::MAX] {assert_eq!(device_op(opcode,0,0),None);}
        assert_eq!(device_op(11,65535,0),Some(DeviceOp::WriteWord(65535)));
    }
}
