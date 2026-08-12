#![deny(unsafe_code)]
#[path="../../../tools/rar-lab/preauth/src/lib.rs"]mod preauth;
mod base_oci;
mod json_bounds;
mod transaction;
use preauth::{InputLockV4,TransactionGraphV1,TRANSACTION_GRAPH_FIELDS,sha256_hex};

fn hash(byte:char)->String{std::iter::repeat_n(byte,64).collect()}
pub(crate) const ARCHIVE_MEMBER_BOUND:usize=4096;

fn collect_repository_effects(
 path:&std::path::Path,root:&std::path::Path,effects:&mut Vec<(String,String)>
){
 let metadata=std::fs::symlink_metadata(path).expect("repository effect metadata");
 let relative=path.strip_prefix(root).expect("repository-relative effect path")
  .to_string_lossy().into_owned();
 if metadata.is_dir(){
  let mut entries:Vec<_>=std::fs::read_dir(path).expect("repository effect directory")
   .map(|entry|entry.expect("repository effect entry").path()).collect();
  entries.sort();
  for entry in entries{collect_repository_effects(&entry,root,effects);}
 }else if metadata.file_type().is_symlink(){
  let target=std::fs::read_link(path).expect("repository effect symlink");
  effects.push((relative,format!("symlink:{}",target.to_string_lossy())));
 }else{
  let bytes=std::fs::read(path).expect("repository effect file");
  effects.push((relative,sha256_hex(&bytes)));
 }
}

fn repository_effect_snapshot()->Vec<(String,String)>{
 let root=std::env::current_dir().expect("repository root");
 let mut effects=Vec::new();
 for relative in ["tools/rar-lab/preauth","tests/preauth","spec/lab/preauth","out/r0/preauth"]{
  let path=root.join(relative);
  if path.exists(){collect_repository_effects(&path,&root,&mut effects);}
  else{effects.push((relative.into(),"missing".into()));}
 }
 effects.sort();
 effects
}

pub(crate) fn assert_side_effect_free_rejection<T:std::fmt::Debug>(
 expected:&str,operation:impl FnOnce()->preauth::Result<T>
){
 let before=repository_effect_snapshot();
 let error=operation().expect_err("one-over-limit input accepted");
 assert_eq!(error.code,expected);
 assert_eq!(repository_effect_snapshot(),before,
  "rejection changed publication or repository state");
}

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
