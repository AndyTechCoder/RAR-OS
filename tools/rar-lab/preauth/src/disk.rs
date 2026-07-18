#![deny(unsafe_code)]

use std::env;
use std::fs::{self, OpenOptions};
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

const CLUSTER: u64 = 65_536;

fn put_u32(buffer: &mut [u8], offset: usize, value: u32) {
    buffer[offset..offset + 4].copy_from_slice(&value.to_be_bytes());
}

fn put_u64(buffer: &mut [u8], offset: usize, value: u64) {
    buffer[offset..offset + 8].copy_from_slice(&value.to_be_bytes());
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("RAR_PREAUTH_BUILD_CONTAINER").as_deref() != Ok("rar-preauth-closure-v2") {
        return Err("refusing outside the pinned Prompt 7A build container".into());
    }
    let path = env::args().nth(1).ok_or("one output path is required")?;
    let output = Path::new(&path);
    if output.is_symlink() || !path.starts_with("/workspace/out/r0/vm/x86_64/") || !path.ends_with(".qcow2") {
        return Err("unsafe disposable disk path".into());
    }
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create_new(true).write(true).open(output)?;
    let mut header = [0_u8; 104];
    put_u32(&mut header, 0, 0x5146_49fb);
    put_u32(&mut header, 4, 3);
    put_u32(&mut header, 20, 16);
    put_u64(&mut header, 24, 64 * 1024 * 1024);
    put_u32(&mut header, 36, 1);
    put_u64(&mut header, 40, 3 * CLUSTER);
    put_u64(&mut header, 48, CLUSTER);
    put_u32(&mut header, 56, 1);
    put_u32(&mut header, 96, 4);
    put_u32(&mut header, 100, 104);
    file.write_all(&header)?;
    file.seek(SeekFrom::Start(CLUSTER))?;
    file.write_all(&(2 * CLUSTER).to_be_bytes())?;
    file.seek(SeekFrom::Start(2 * CLUSTER))?;
    for _ in 0..4 {
        file.write_all(&1_u16.to_be_bytes())?;
    }
    file.set_len(4 * CLUSTER)?;
    file.sync_all()?;
    Ok(())
}
