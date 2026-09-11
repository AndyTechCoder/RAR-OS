//! Settings code variants for actual signed laboratory update packages.
//! Health is local bounded data only: no syscall, device, heap or IPC.
use crate::services::{self,apps::View};
#[derive(Clone,Copy)]
pub struct Preferences{light:bool,compact:bool}
impl Preferences{
    pub const fn new()->Self{Self{light:false,compact:false}}
    fn view_variant(&self,updated:bool)->View{
        let mut view=View::EMPTY;view.line(0,b"APPEARANCE");
        view.line(1,if self.light{b"LIGHT"}else{b"DARK"});
        view.line(2,b"SPACE TO CHANGE THEME");
        if updated{
            view.line(3,b"D SPACING / X LAB FAULT");
            view.line(4,if self.compact{b"COMPACT"}else{b"COMFORTABLE"});
            view.line(5,b"UPDATED SETTINGS");
        }else{view.line(3,b"SESSION ONLY");}
        view
    }
    pub fn view(&self)->View{self.view_variant(cfg!(rar_settings_v2))}
    fn key_variant(&mut self,key:u8,updated:bool)->Option<Option<bool>>{
        if key==b' '{self.light=!self.light;Some(Some(self.light))}
        else if updated&&key==b'd'{self.compact=!self.compact;Some(None)}
        else{None}
    }
    /// Outer None: ignored. Some(None): local view change. Some(Some(theme)):
    /// view plus the existing shell theme notification, no new authority.
    pub fn key(&mut self,key:u8)->Option<Option<bool>>{self.key_variant(key,cfg!(rar_settings_v2))}
}
fn fault_key(key:u8,updated:bool)->bool{updated&&key==b'x'}
/// Deliberate laboratory active-process fault; never called by trial health.
pub fn laboratory_fault_key(key:u8)->bool{fault_key(key,cfg!(rar_settings_v2))}
pub fn initial_view()->View{Preferences::new().view()}
pub fn health()->bool{
    if cfg!(rar_settings_fail_health){return false;}
    let view=initial_view();
    view.lines[0].as_bytes()==b"APPEARANCE"&&view.lines[1].as_bytes()==b"DARK"&&
        (0..6).all(|i|services::line(1,i,&view.lines[i]).is_some())
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn active_fault_key_is_updated_variant_only(){
        for key in 0..=255{
            assert!(!fault_key(key,false));
            assert_eq!(fault_key(key,true),key==b'x');
        }
        assert_eq!(laboratory_fault_key(b'x'),cfg!(rar_settings_v2));
    }
    #[test]fn exact_initial_view_and_trial_local_health(){
        let view=Preferences::new().view_variant(false);
        let expected:[&[u8];6]=[b"APPEARANCE",b"DARK",b"SPACE TO CHANGE THEME",b"SESSION ONLY",b"",b""];
        for(i,text)in expected.iter().enumerate(){assert_eq!(view.lines[i].as_bytes(),*text);}
        assert_eq!(health(),!cfg!(rar_settings_fail_health));
    }
    #[test]fn updated_code_adds_interaction_without_changing_old_variant(){
        let mut state=Preferences::new();
        assert_eq!(state.key_variant(b'd',false),None);
        assert_eq!(state.view_variant(true).lines[4].as_bytes(),b"COMFORTABLE");
        assert_eq!(state.key_variant(b'd',true),Some(None));
        let view=state.view_variant(true);
        assert_eq!(view.lines[4].as_bytes(),b"COMPACT");
        assert_eq!(view.lines[5].as_bytes(),b"UPDATED SETTINGS");
        assert_eq!(state.key_variant(b' ',true),Some(Some(true)));
        assert_eq!(state.view_variant(true).lines[1].as_bytes(),b"LIGHT");
        assert_eq!(state.key_variant(b' ',true),Some(Some(false)));
        for i in 0..6{assert!(services::line(1,i,&view.lines[i]).is_some());}
    }
}
