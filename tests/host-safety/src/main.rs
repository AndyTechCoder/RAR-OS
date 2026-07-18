#![deny(unsafe_code)]
#[cfg_attr(rar_flat_bootstrap,path="safety.rs")]
#[cfg_attr(not(rar_flat_bootstrap),path="../../../tools/rar-lab/safety/src/lib.rs")]
mod safety;

#[test]
fn removed_host_records_are_unconditionally_refused(){
 for bytes in [b"".as_slice(),b"synthetic-record".as_slice(),b"schema=unknown\n".as_slice()]{
  assert_eq!(safety::refuse_removed_host_record(bytes).unwrap_err().code,"legacy-preauth-version-refused");
 }
}

#[test]
fn refusal_api_has_no_authority_or_process_surface(){
 let source=include_str!("../../../tools/rar-lab/safety/src/lib.rs");
 for forbidden in ["std::process","Command::new","spawn(","consume_once","authorize_then"]{assert!(!source.contains(forbidden),"forbidden host surface: {forbidden}");}
}
