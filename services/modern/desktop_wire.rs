//! Private byte compatibility constants; no volatile storage implementation.
pub(crate) const CREATE:u8=1;
pub(crate) const WRITE:u8=2;
pub(crate) const READ:u8=3;
pub(crate) const LIST:u8=4;
pub(crate) const OK:u8=0;
pub(crate) const INVALID:u8=1;
pub(crate) const NOT_FOUND:u8=2;
pub(crate) const EXISTS:u8=3;
pub(crate) const QUOTA:u8=4;
#[cfg(test)]
#[path="../platform/model.rs"]
mod historical;
#[cfg(test)]
pub(crate) fn request(op:u8,name:&[u8],data:&[u8])->Option<[u8;128]> {
    historical::request(op,name,data)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn constants_match_unchanged_historical_protocol() {
        assert_eq!([CREATE,WRITE,READ,LIST,OK,INVALID,NOT_FOUND,EXISTS,QUOTA],
            [historical::CREATE,historical::WRITE,historical::READ,historical::LIST,
             historical::OK,historical::INVALID,historical::NOT_FOUND,historical::EXISTS,historical::QUOTA]);
    }
}
