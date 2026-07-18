use std::collections::BTreeSet;
use super::{PreauthError, Result};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageRow {
    pub name: String, pub version: String, pub architecture: String, pub filename: String,
    pub size: u64, pub sha256: String, pub license_sha256: String,
    pub source_name: String, pub source_version: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackageManifest { pub rows: Vec<PackageRow> }

fn token(value: &str) -> bool {
    !value.is_empty() && value.len() <= 256 && value.bytes().all(|byte|
        byte.is_ascii_alphanumeric() || matches!(byte, b'.'|b'_'|b'-'|b'+'|b':'|b'~'|b'%'))
}
fn digest(value: &str) -> bool {
    value.len()==64 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

impl PackageManifest {
    pub fn parse(input: &str) -> Result<Self> {
        if input.len()>64*1024 || !input.ends_with('\n') || input.contains('\r')
            || !input.starts_with("schema=rar-preauth-package-manifest-v2\n") {
            return Err(PreauthError::new("invalid-package-manifest"));
        }
        let mut rows=Vec::new(); let mut names=BTreeSet::new(); let mut files=BTreeSet::new(); let mut previous=None::<String>;
        for line in input.lines().skip(1) {
            let f:Vec<_>=line.split('|').collect();
            if f.len()!=10 || f[0]!="package" || !token(f[1]) || !token(f[2]) || !matches!(f[3],"amd64"|"all")
                || !token(f[4]) || !f[4].ends_with(".deb") || !digest(f[6]) || !digest(f[7]) || !token(f[8]) || !token(f[9]) {
                return Err(PreauthError::new("invalid-package-row"));
            }
            let size=f[5].parse::<u64>().map_err(|_|PreauthError::new("invalid-package-size"))?;
            if size==0 || size>128*1024*1024 { return Err(PreauthError::new("invalid-package-size")); }
            if previous.as_deref().is_some_and(|name|name>=f[1]) || !names.insert(f[1].to_owned()) || !files.insert(f[4].to_owned()) {
                return Err(PreauthError::new("noncanonical-package-order"));
            }
            previous=Some(f[1].to_owned()); rows.push(PackageRow{name:f[1].into(),version:f[2].into(),architecture:f[3].into(),filename:f[4].into(),size,sha256:f[6].into(),license_sha256:f[7].into(),source_name:f[8].into(),source_version:f[9].into()});
        }
        if rows.len()!=36 { return Err(PreauthError::new("package-count-mismatch")); }
        Ok(Self{rows})
    }
}
