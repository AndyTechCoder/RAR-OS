//! Pure Modern kernel mechanisms and private ABI; runtime activation is separate.
#![cfg_attr(not(test), no_std)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
pub mod model;
// The only native unsafe leaf. Model and ABI retain their own forbid guards.
#[allow(unsafe_code)] pub mod native_pio;
#[path="../../core/modern/abi.rs"] pub mod abi;
#[cfg(test)]
mod bootstrap_integration {
    use super::{abi,model};
    #[test] fn kernel_grants_match_private_active_bootstrap_layout() {
        let runtime=model::Runtime::new();
        for role in [0usize,1,2,3,4,5,6,8,9] {
            let mut boot=abi::Boot {magic:abi::MAGIC,version:abi::VERSION,bytes:abi::BOOT_BYTES,
                role:role as u64,generation:runtime.binding(role).unwrap().unwrap().incarnation,
                entry:0x401000,..abi::Boot::EMPTY};
            for index in 0..12 {boot.caps[index]=runtime.handle(role,index).unwrap_or(0);}
            for index in 0..10 {boot.peers[index]=runtime.binding(index).unwrap().map_or(0,|e|e.incarnation);}
            if role==3 {boot.framebuffer=0x800000;boot.width=640;boot.height=480;boot.pitch=640;}
            if matches!(role,1|9) {boot.device_sectors=14;boot.device_serial=[b'S';20];boot.device_model=[b'M';40];}
            assert!(abi::valid_boot(&boot),"role {role}");
        }
    }
}
