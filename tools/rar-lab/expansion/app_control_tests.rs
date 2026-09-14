//! Pure trusted app control framing tests, no target syscall execution.
#[path="../../../core/expansion/app_control.rs"] mod control;
use control::*;
#[test]fn exact_record_roundtrip_and_full_width_identity(){
    for index in 0..2{
        for (state,incarnation) in [(0,0),(1,1u64<<40),(2,u64::MAX)]{
            let record=Record{index,incarnation,generation:1,rights:if index==0{3}else{1},state,
                application:if index==0{NOTES}else{COUNTER},owner:[7;32],digest:[9;32]};
            let raw=record.encode().unwrap();assert_eq!(Record::decode(&raw),Ok(record));
            for length in 0..128{assert!(Record::decode(&raw[..length]).is_err());}
            for at in (0..8).chain(12..16).chain(120..128){
                let mut bad=raw;bad[at]^=128;assert!(Record::decode(&bad).is_err());
            }
        }
    }
}
#[test]fn canonical_controls_and_inputs(){
    assert_eq!(SYSCALL,14);assert_eq!(SIZE,128);
    for op in 1..=6{
        for index in 0..2{
            let inc=if op==1{0}else{u64::MAX};
            let raw=control(op,index,inc).unwrap();assert_eq!(parse(&raw),Ok((op,index,inc)));
            for at in (0..8).chain(10..16).chain(24..128){
                let mut bad=raw;bad[at]^=1;assert!(parse(&bad).is_err());
            }
        }
    }
    for op in [0,7,255]{assert!(control(op,0,1).is_err());}
    assert!(control(1,0,1).is_err());assert!(control(2,0,0).is_err());assert!(control(1,2,0).is_err());
    for key in 0..=255{
        let allowed=matches!(key,8|13|27|32..=126);
        assert_eq!(input(0,1<<40,key).is_ok(),allowed);
        if allowed{assert_eq!(parse_input(&input(0,1<<40,key).unwrap()),Ok((0,1<<40,key)));}
    }
    for compact in [false,true]{
        let raw=profile(compact);assert_eq!(parse_profile(&raw),Ok(compact));
        for at in (0..8).chain(9..128){let mut bad=raw;bad[at]^=1;assert!(parse_profile(&bad).is_err());}
    }
}

#[test]fn direct_decoders_preserve_exact_canonical_roundtrips(){
    let r=Record{index:0,incarnation:1<<40,generation:1,rights:3,state:2,
        application:NOTES,owner:[7;32],digest:[9;32]};
    let fixtures=[r.encode().unwrap(),control(1,0,0).unwrap(),control(6,1,9).unwrap(),
        input(1,9,b'a').unwrap(),profile(true)];
    for original in fixtures{
        for at in 0..128{for value in [0u8,1,7,128,255]{
            let mut bytes=original;bytes[at]=value;
            if let Ok(record)=Record::decode(&bytes){assert_eq!(record.encode().unwrap(),bytes);}
            if let Ok((op,index,inc))=parse(&bytes){assert_eq!(control(op,index,inc).unwrap(),bytes);}
            if let Ok((index,inc,key))=parse_input(&bytes){assert_eq!(input(index,inc,key).unwrap(),bytes);}
            if let Ok(compact)=parse_profile(&bytes){assert_eq!(profile(compact),bytes);}
        }}
    }
}
