//! Cloud-only cross-language codec conformance; no PE or VM execution.
#![forbid(unsafe_code)]
#![allow(dead_code)]
#[path="../../../core/crypto/sha256.rs"] mod sha256;
#[path="../../../core/crypto/sha512.rs"] mod sha512;
#[path="../../../core/crypto/ed25519.rs"] mod ed25519;
#[path="../../../core/modern/manifest.rs"] mod manifest;
#[path="../../../core/modern/journal.rs"] mod journal;
#[path="../../../core/modern/system_volume.rs"] mod system_volume;
#[path="../../../nucleus/platform/pe.rs"] mod pe;
#[derive(Clone,Copy,Debug,PartialEq,Eq)] pub enum Error {Invalid,Denied}
use std::io::Read;
const PACKAGE:usize=384+1024;
const IMAGE:usize=16384*512;
const TOTAL:usize=5*PACKAGE+IMAGE;
struct ReadOnly<'a>(&'a [u8]);
impl system_volume::Io for ReadOnly<'_>{
    fn read(&mut self,sector:u32)->Result<[u8;512],()>{
        let start=(sector as usize).checked_mul(512).ok_or(())?;
        self.0.get(start..start.checked_add(512).ok_or(())?)
            .ok_or(())?.try_into().map_err(|_|())
    }
    fn write(&mut self,_:u32,_:&[u8;512])->Result<(),()>{panic!("unexpected media write")}
    fn flush(&mut self)->Result<(),()>{panic!("unexpected media flush")}
}
fn main(){
    let mut input=Vec::new();
    std::io::stdin().lock().take((TOTAL+1)as u64).read_to_end(&mut input).unwrap();
    assert_eq!(input.len(),TOTAL,"exact bounded synthetic fixture stream");
    let package=|i:usize|&input[i*PACKAGE..(i+1)*PACKAGE];
    let verify=|i:usize,floor:u64|{
        let p=package(i);manifest::verify(&p[..384],&p[384..],floor)
    };
    let factory=verify(0,1).unwrap();
    let update=verify(1,2).unwrap();
    assert_eq!(factory.manifest().generation(),1);
    assert_eq!(update.manifest().generation(),2);
    // Manifest validity is distinct from executing the failed-health variant.
    assert_eq!(verify(2,3).unwrap().manifest().generation(),3);
    assert!(matches!(verify(3,1),Err(manifest::Reject::Signature)));
    assert!(matches!(verify(4,1),Err(manifest::Reject::Compatibility)));
    assert!(matches!(verify(0,2),Err(manifest::Reject::Rollback)));
    assert!(matches!(verify(1,3),Err(manifest::Reject::Rollback)));
    let image=&input[5*PACKAGE..];
    let first:&[u8;512]=image[..512].try_into().unwrap();
    let second:&[u8;512]=image[512..1024].try_into().unwrap();
    let expected=journal::Record::factory(&factory);
    assert_eq!(journal::Record::decode(first).unwrap(),expected);
    assert_eq!(journal::select([first,second]).unwrap().record(),expected);
    assert_eq!(first,&expected.encode());
    assert_eq!(second,&[0;512]);
    assert_eq!(&image[1024..1024+PACKAGE],package(0));
    assert!(image[1024+PACKAGE..].iter().all(|&b|b==0));
    let mut volume=system_volume::Volume::mount(ReadOnly(image),system_volume::SECTORS).unwrap();
    assert_eq!(volume.record(),expected);
    let prepared=volume.prepare_boot().unwrap();
    assert_eq!(prepared.identity().package_hash,sha256::sha256(package(0)).unwrap());
    assert_eq!(prepared.identity().length,PACKAGE);
    let mut copied=Vec::new();
    volume.copy_prepared(&prepared,|total,offset,part|{
        assert_eq!(total,PACKAGE);assert_eq!(offset,copied.len());
        copied.extend_from_slice(part);Ok(())
    }).unwrap();
    assert_eq!(copied,package(0));
    volume.complete_boot(&prepared).unwrap();
    assert_eq!(volume.record(),expected);
    assert_eq!(expected.install(&update).unwrap().highest_committed_generation(),2);
    // Corrupt selectors must not be interpreted as virgin-media authorization.
    let mut corrupt=image.to_vec();corrupt[..1024].fill(0);
    assert!(system_volume::Volume::mount(ReadOnly(&corrupt),system_volume::SECTORS).is_err());
    corrupt[..512].copy_from_slice(first);corrupt[480]^=1;
    assert!(system_volume::Volume::mount(ReadOnly(&corrupt),system_volume::SECTORS).is_err());
    println!("Modern package conformance: Python bytes pass RAR signature/ABI/rollback, selector, mounted boot readback; no target execution");
}
