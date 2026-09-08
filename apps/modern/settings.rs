//! Settings view construction shared by constrained health and active UI.
//! Contract: docs/interfaces/modern-trial-entry.md.
use crate::services::{self,apps::View};
pub fn initial_view()->View {
    let mut view=View::EMPTY;view.line(0,b"APPEARANCE");view.line(1,b"DARK");
    view.line(2,b"SPACE TO CHANGE THEME");view.line(3,b"SESSION ONLY");view
}
/// Only local bounded data; no syscall, IPC, device, heap or surface access.
pub fn health()->bool {
    let view=initial_view();
    view.lines[0].as_bytes()==b"APPEARANCE"&&view.lines[1].as_bytes()==b"DARK"&&
        (0..6).all(|i|services::line(1,i,&view.lines[i]).is_some())
}
#[cfg(test)] mod tests {
    use super::*;
    #[test] fn exact_initial_view_and_trial_local_health() {
        let view=initial_view();
        let expected:[&[u8];6]=[
            b"APPEARANCE",b"DARK",b"SPACE TO CHANGE THEME",b"SESSION ONLY",b"",b""];
        for (i,text) in expected.iter().enumerate() {
            assert_eq!(view.lines[i].as_bytes(),*text);
            assert!(services::line(1,i,&view.lines[i]).is_some());
        }
        assert!(health());
    }
}
