#![deny(unsafe_code)]
#[derive(Clone,Debug,Eq,PartialEq)]pub struct LegacyRefusal{pub code:&'static str}
pub fn refuse_removed_host_record(_bytes:&[u8])->Result<(),LegacyRefusal>{Err(LegacyRefusal{code:"legacy-preauth-version-refused"})}
