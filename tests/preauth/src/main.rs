#![deny(unsafe_code)]
#[path="../../../tools/rar-lab/preauth/src/lib.rs"]mod preauth;
mod base_oci;
mod transaction;
use preauth::{InputLockV4,TransactionGraphV1,TRANSACTION_GRAPH_FIELDS,sha256_hex};

fn hash(byte:char)->String{std::iter::repeat_n(byte,64).collect()}

#[test]
fn input_lock_v4_accepts_only_current_input_contract(){
 let current=include_str!("../../../spec/lab/preauth/locks/r0-x86_64-preauth-input-v4.lock");
 InputLockV4::parse(current).expect("current input lock");
 let old=include_str!("../fixtures/legacy-rejection/records/r0-x86_64-preauth-v3.lock");
 assert!(InputLockV4::parse(old).is_err());
}

#[test]
fn transaction_graph_is_typed_complete_and_self_hashed(){
 let mut payload=String::new();
 for name in &TRANSACTION_GRAPH_FIELDS[..TRANSACTION_GRAPH_FIELDS.len()-1]{let value=match *name{"schema"=>"rar-preauth-transaction-graph-v1".into(),"source_revision"=>"a".repeat(40),"raw_to_canonical_index_relation"=>"strict-json-parse+canonical-serialize-v1".into(),_=>hash('b')};payload.push_str(name);payload.push('=');payload.push_str(&value);payload.push('\n');}
 let record=format!("{payload}record_sha256={}\n",sha256_hex(payload.as_bytes()));
 let parsed=TransactionGraphV1::parse(&record).expect("transaction graph");
 assert_eq!(parsed.source_revision,"a".repeat(40));
 assert_eq!(parsed.nodes.len(),39);
}
