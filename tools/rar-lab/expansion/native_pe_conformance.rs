//! Cloud host byte inspection only. Never executes the input native image.
use std::io::{self, Read};
#[derive(Debug)] pub enum Error { Invalid, Denied }
#[path="../../../nucleus/platform/pe.rs"] mod pe;
fn main() {
    let mut bytes=Vec::new();
    io::stdin().take((pe::LIMIT+1) as u64).read_to_end(&mut bytes).unwrap();
    assert!((512..=pe::LIMIT).contains(&bytes.len()));
    let layout=pe::parse(&bytes).expect("actual kernel PE parser");
    assert!(layout.entry>=pe::BASE+4096 && layout.entry<pe::BASE+pe::LIMIT as u64);
    assert!(layout.image_size<=pe::LIMIT && layout.count<=3);
    assert!(layout.header_size>=512);
    assert!(layout.sections[..layout.count].iter().any(|s|s.executable));
    for section in &layout.sections[..layout.count] {
        assert!(!(section.writable && section.executable));
        assert!(section.file_offset+section.file_size<=bytes.len());
        assert!(section.virtual_offset+section.memory_size<=layout.image_size);
    }
    println!("Native C PE bytes accepted by the unchanged kernel parser: {} bytes, {} sections",bytes.len(),layout.count);
}
