//! Desktop role authority extends the released Platform mechanisms, not policy.
#[path="../platform/model.rs"] mod platform;
pub use platform::*;
pub const ACTIVE:usize=8;
pub fn receives(role:usize)->bool{matches!(role,0|1|3|4|5|6)}
pub fn send_target(role:usize,slot:usize)->Option<usize>{
    let allowed=match role {
        0=>matches!(slot,2|4|5|6),
        1=>matches!(slot,4|6),
        2=>slot==1,
        4|6=>matches!(slot,2|3),
        5=>matches!(slot,1|2),
        _=>false,
    };
    if !allowed{return None;}
    match slot{1=>Some(0),2=>Some(3),3=>Some(1),4=>Some(4),5=>Some(5),6=>Some(6),_=>None}
}
#[cfg(test)] mod desktop_tests {
    use super::*;
    #[test] fn exact_least_authority_matrix(){
        let expected=[[(2,3),(4,4),(5,5),(6,6)].as_slice(),
            &[(4,4),(6,6)],&[(1,0)],&[],&[(2,3),(3,1)],&[(1,0),(2,3)],&[(2,3),(3,1)],&[]];
        for role in 0..16 {for slot in 0..16 {
            let want=expected.get(role).and_then(|pairs|pairs.iter().find(|&&(s,_)|s==slot).map(|&(_,t)|t));
            assert_eq!(send_target(role,slot),want);
        }}
        for role in 0..16 {assert_eq!(receives(role),[0,1,3,4,5,6].contains(&role));}
    }
}

#[cfg(test)] #[path="../../core/desktop/abi.rs"] mod desktop_abi;
#[cfg(test)] mod bootstrap_tests {
    use super::desktop_abi::*;
    #[test] fn fixed_boot_layout_and_identity() {
        assert_eq!(core::mem::size_of::<Boot>(),176);
        assert_eq!(core::mem::size_of::<Envelope>(),144);
        assert_eq!(BOOT_ADDRESS%core::mem::align_of::<Boot>(),0);
        let mut b=Boot{magic:MAGIC,role:0,generation:1,entry:0,kernel_probe:0,peer_probe:0,
            framebuffer:0,width:0,height:0,pitch:0,format:0,caps:[0;11]};
        for role in 0..8{b.role=role;assert!(valid_boot(&b));}
        b.role=8;assert!(!valid_boot(&b));b.role=0;b.generation=2;assert!(!valid_boot(&b));
        b.generation=1;b.magic=0;assert!(!valid_boot(&b));
    }
}
