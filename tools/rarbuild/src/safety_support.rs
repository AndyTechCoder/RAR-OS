#![deny(unsafe_code)]
#[cfg_attr(rar_flat_bootstrap,path="unix_fs.rs")]
#[cfg_attr(not(rar_flat_bootstrap),path="../../rar-lab/safety/src/unix_fs.rs")]
mod unix_fs;
#[cfg_attr(rar_flat_bootstrap,path="hash.rs")]
#[cfg_attr(not(rar_flat_bootstrap),path="../../rar-lab/preauth/src/hash.rs")]
mod hash;
use std::{fmt,fs};
use std::io::{Read,Seek,SeekFrom,Write};
use std::path::{Component,Path,PathBuf};
use std::sync::atomic::{AtomicU64,Ordering};
pub use hash::sha256_hex;

#[derive(Clone,Debug,Eq,PartialEq)]
pub struct SafetyError{pub code:&'static str,pub detail:String}
impl SafetyError{pub(crate) fn new(code:&'static str,detail:impl Into<String>)->Self{Self{code,detail:detail.into()}}}
impl fmt::Display for SafetyError{fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{write!(f,"{}: {}",self.code,self.detail)}}
impl std::error::Error for SafetyError{}
pub type SafetyResult<T>=Result<T,SafetyError>;
pub const REPOSITORY_MARKER_MAX_BYTES:usize=1024*1024;

pub fn validate_repository_root(root:&Path)->SafetyResult<PathBuf>{
 if !root.is_absolute(){return Err(SafetyError::new("unsafe-root","repository root must be absolute"));}
 let canonical=fs::canonicalize(root).map_err(|e|SafetyError::new("repository-root-unavailable",e.to_string()))?;
 if canonical!=root{return Err(SafetyError::new("repository-root-alias","repository root must be canonical"));}
 let metadata=fs::symlink_metadata(root).map_err(|e|SafetyError::new("repository-root-unavailable",e.to_string()))?;
 if metadata.file_type().is_symlink()||!metadata.is_dir(){return Err(SafetyError::new("unsafe-root","repository root must be a real directory"));}
 for marker in ["Cargo.toml","AGENTS.md","docs/approval-record.md","docs/host-safety.md","docs/tasks/release-0.md"]{let item=fs::symlink_metadata(root.join(marker)).map_err(|_|SafetyError::new("repository-marker-absent",marker))?;if item.file_type().is_symlink()||!item.is_file(){return Err(SafetyError::new("unsafe-repository-marker",marker));}}
 let git=fs::symlink_metadata(root.join(".git")).map_err(|_|SafetyError::new("repository-marker-absent",".git"))?;if git.file_type().is_symlink()||!(git.is_dir()||git.is_file()){return Err(SafetyError::new("unsafe-repository-marker",".git"));}
 for(document,marker)in[("docs/approval-record.md","Status: Approved"),("docs/approval-record.md","Approval: approved"),("docs/tasks/release-0.md","Status: Ready — Gate 0 owner approval recorded"),("docs/host-safety.md","Status: Mandatory and effective immediately")]{let text=read_bounded_utf8_file(&root.join(document),REPOSITORY_MARKER_MAX_BYTES)?;if !text.lines().any(|line|line.starts_with(marker)){return Err(SafetyError::new("repository-approval-marker-mismatch",document));}}
 Ok(canonical)
}
pub fn validate_workspace_path(root:&Path,relative:&str,must_exist:bool)->SafetyResult<PathBuf>{
 let root=validate_repository_root(root)?;let path=Path::new(relative);if path.is_absolute()||path.components().any(|c|!matches!(c,Component::Normal(_))){return Err(SafetyError::new("unsafe-path","path is not canonical repository-relative"));}
 let joined=root.join(path);let mut current=root.clone();for component in path.components(){let Component::Normal(part)=component else{unreachable!()};current.push(part);match fs::symlink_metadata(&current){Ok(m)if m.file_type().is_symlink()=>return Err(SafetyError::new("symlink-path-forbidden","path contains a symlink")),Ok(_)=>{},Err(e)if e.kind()==std::io::ErrorKind::NotFound=>break,Err(e)=>return Err(SafetyError::new("path-inspection-failed",e.to_string()))}}
 if must_exist{let m=fs::symlink_metadata(&joined).map_err(|_|SafetyError::new("required-file-absent","required file is absent"))?;if m.file_type().is_symlink()||!m.is_file(){return Err(SafetyError::new("required-file-not-regular","required file is not regular"));}let canonical=fs::canonicalize(&joined).map_err(|e|SafetyError::new("path-canonicalization-failed",e.to_string()))?;if canonical!=joined||!canonical.starts_with(&root){return Err(SafetyError::new("path-alias-forbidden","path is aliased"));}}
 Ok(joined)
}
static TEMP_SEQUENCE:AtomicU64=AtomicU64::new(0);
pub fn atomic_write_workspace_file(root:&Path,relative:&Path,bytes:&[u8])->SafetyResult<()>{atomic_write_inner(root,relative,bytes,&mut||Ok(()))}
pub fn atomic_write_workspace_file_with_precommit<F>(root:&Path,relative:&Path,bytes:&[u8],mut before:F)->SafetyResult<()>where F:FnMut()->SafetyResult<()>{atomic_write_inner(root,relative,bytes,&mut before)}
#[cfg(test)]pub fn atomic_write_workspace_file_with_hook<F>(root:&Path,relative:&Path,bytes:&[u8],mut before:F)->SafetyResult<()>where F:FnMut()->SafetyResult<()>{atomic_write_inner(root,relative,bytes,&mut before)}
fn atomic_write_inner(root:&Path,relative:&Path,bytes:&[u8],before:&mut dyn FnMut()->SafetyResult<()>)->SafetyResult<()>{
 if relative.is_absolute()||relative.components().any(|c|!matches!(c,Component::Normal(_))){return Err(SafetyError::new("unsafe-output-path","output path is not canonical relative"));}
 let parent=relative.parent().ok_or_else(||SafetyError::new("unsafe-output-path","output has no parent"))?;let destination=relative.file_name().and_then(|n|n.to_str()).ok_or_else(||SafetyError::new("unsafe-output-path","filename is not UTF-8"))?;let directory=unix_fs::open_or_create_relative_directory(root,parent)?;
 let(name,mut temporary)=(0..128).find_map(|_|{let sequence=TEMP_SEQUENCE.fetch_add(1,Ordering::Relaxed);let name=format!(".rarbuild-{}-{sequence:016x}.tmp",std::process::id());match unix_fs::create_new_file_at(&directory,&name){Ok(file)=>Some(Ok((name,file))),Err(e)if e.code=="descriptor-file-exists"=>None,Err(e)=>Some(Err(e))}}).transpose()?.ok_or_else(||SafetyError::new("temporary-output-exhausted","no exclusive temporary name"))?;
 let staged=(||{temporary.write_all(bytes).map_err(|e|SafetyError::new("output-write-failed",e.to_string()))?;temporary.sync_all().map_err(|e|SafetyError::new("output-sync-failed",e.to_string()))?;temporary.seek(SeekFrom::Start(0)).map_err(|e|SafetyError::new("output-seek-failed",e.to_string()))?;let actual=hash::sha256_reader(&mut temporary).map_err(|e|SafetyError::new("hash-read-failed",e.to_string()))?;if actual!=sha256_hex(bytes){return Err(SafetyError::new("output-verification-failed","staged bytes differ"));}before()?;unix_fs::verify_open_directory_binding(&directory,&root.join(parent))?;unix_fs::rename_at(&directory,&name,destination)?;directory.sync_all().map_err(|e|SafetyError::new("output-directory-sync-failed",e.to_string()))?;Ok(())})();
 if let Err(primary)=staged{let _=unix_fs::unlink_at(&directory,&name);let _=directory.sync_all();return Err(primary);}Ok(())
}
pub fn read_bounded_utf8_file(path:&Path,maximum:usize)->SafetyResult<String>{let mut file=unix_fs::open_absolute_regular_nofollow(path)?;let metadata=file.metadata().map_err(|e|SafetyError::new("bounded-read-metadata-failed",e.to_string()))?;if metadata.len()>maximum as u64{return Err(SafetyError::new("bounded-read-too-large","file exceeds bound"));}let mut bytes=Vec::with_capacity(metadata.len()as usize);Read::by_ref(&mut file).take(maximum as u64+1).read_to_end(&mut bytes).map_err(|e|SafetyError::new("bounded-read-failed",e.to_string()))?;if bytes.len()>maximum{return Err(SafetyError::new("bounded-read-too-large","file exceeds bound"));}String::from_utf8(bytes).map_err(|_|SafetyError::new("bounded-read-not-utf8","file is not UTF-8"))}
pub fn sha256_file(path:&Path)->SafetyResult<String>{let mut file=unix_fs::open_absolute_regular_nofollow(path)?;hash::sha256_reader(&mut file).map_err(|e|SafetyError::new("hash-read-failed",e.to_string()))}
