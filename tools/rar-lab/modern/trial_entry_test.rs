//! Pure host tests only: no Modern entry, trap, device or executable invocation.
#![forbid(unsafe_code)]
#[path="../../../core/modern/abi.rs"] mod abi;
#[path="../../../services/modern/gui.rs"] mod services;
#[path="../../../apps/modern/settings.rs"] mod settings;
