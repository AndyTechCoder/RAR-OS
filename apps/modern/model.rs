//! Pure status rules used by Modern Files and Terminal.
#![forbid(unsafe_code)]
use crate::{transport::Outcome,desktop_wire as fs};
pub struct State {uncertain:bool,readonly:bool,unavailable:bool}
impl State{
    pub const fn new()->Self{Self{uncertain:false,readonly:false,unavailable:false}}
    pub fn note(&mut self,result:Outcome)->Outcome{
        match result{
            Outcome::ReadOnly=>self.readonly=true,
            Outcome::Unavailable=>self.unavailable=true,
            Outcome::SaveUncertain=>self.uncertain=true,
            Outcome::Reply(_)=>{},
        }
        result
    }
    pub fn locked_outcome(&self,closed:bool)->Outcome{
        if self.uncertain{Outcome::SaveUncertain}
        else if self.unavailable||closed{Outcome::Unavailable}
        else{Outcome::ReadOnly}
    }
    /// Independent heading warning: a long value must not hide sticky state.
    pub fn warning(&self,closed:bool,locked:bool,input_lost:bool)->Option<&'static [u8]>{
        if input_lost{
            Some(if self.uncertain{b"UNCERTAIN SAVE / INPUT LOST"}
                else if self.unavailable||closed{b"UNAVAILABLE / INPUT LOST"}
                else if self.readonly||locked{b"READ ONLY / INPUT LOST"}
                else{b"INPUT LOST - CHECK COMMAND"})
        }else if self.uncertain||self.unavailable||closed||self.readonly||locked{
            Some(self.label(closed,locked,false))
        }else{None}
    }
    pub fn label(&self,closed:bool,locked:bool,input_lost:bool)->&'static [u8]{
        if self.uncertain{b"SAVE UNCERTAIN - WRITES LOCKED"}
        else if self.unavailable||closed{b"STORAGE UNAVAILABLE"}
        else if self.readonly||locked{b"DATA VAULT - READ ONLY"}
        else if input_lost{b"INPUT LOST - CHECK COMMAND"}
        else{b"PUBLIC LAB DATA - NOT PRIVATE"}
    }
}
pub fn can_write_after_create(result:Outcome)->bool{
    matches!(result,Outcome::Reply(r) if matches!(r[0],fs::OK|fs::EXISTS)&&r[1..].iter().all(|b|*b==0))
}
/// Only the exact canonical successful durable WRITE response permits SAVED.
pub fn saved(result:Outcome)->bool{matches!(result,Outcome::Reply(r) if r==[0;128])}
pub fn problem(result:Outcome)->&'static [u8]{
    match result{
        Outcome::SaveUncertain=>b"SAVE UNCERTAIN - DO NOT RETRY",
        Outcome::ReadOnly=>b"DATA VAULT IS READ ONLY",
        Outcome::Unavailable=>b"STORAGE UNAVAILABLE",
        Outcome::Reply(r)=>match r[0]{
            fs::NOT_FOUND=>b"FILE NOT FOUND",fs::EXISTS=>b"FILE ALREADY EXISTS",
            fs::QUOTA=>b"FILE OR DATA QUOTA REACHED",_=>b"INVALID FILE REQUEST",
        },
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn only_complete_canonical_ack_can_show_saved(){
        assert!(saved(Outcome::Reply([0;128])));
        for outcome in [Outcome::ReadOnly,Outcome::Unavailable,Outcome::SaveUncertain]{
            assert!(!saved(outcome));assert!(!can_write_after_create(outcome));
        }
        for i in 0..128{
            let mut r=[0;128];r[i]=1;assert!(!saved(Outcome::Reply(r)));
        }
        let mut exists=[0;128];exists[0]=fs::EXISTS;
        assert!(can_write_after_create(Outcome::Reply(exists)));assert!(!saved(Outcome::Reply(exists)));
        for i in 1..128{let mut r=exists;r[i]=1;assert!(!can_write_after_create(Outcome::Reply(r)));}
    }
    #[test]fn read_success_never_clears_uncertainty_or_readonly(){
        let mut s=State::new();s.note(Outcome::SaveUncertain);s.note(Outcome::Reply([0;128]));
        assert_eq!(s.locked_outcome(false),Outcome::SaveUncertain);
        assert_eq!(s.label(false,true,false),b"SAVE UNCERTAIN - WRITES LOCKED");
        s.note(Outcome::Unavailable);assert_eq!(s.locked_outcome(true),Outcome::SaveUncertain);
        let mut s=State::new();s.note(Outcome::ReadOnly);s.note(Outcome::Reply([0;128]));
        assert_eq!(s.label(false,true,false),b"DATA VAULT - READ ONLY");
        assert_eq!(s.locked_outcome(false),Outcome::ReadOnly);
        let mut s=State::new();s.note(Outcome::Unavailable);s.note(Outcome::Reply([0;128]));
        assert_eq!(s.label(false,false,false),b"STORAGE UNAVAILABLE");
        assert_eq!(State::new().label(false,false,true),b"INPUT LOST - CHECK COMMAND");
        assert_ne!(problem(Outcome::SaveUncertain),problem(Outcome::Unavailable));
        let mut s=State::new();assert_eq!(s.warning(false,false,false),None);
        assert_eq!(s.warning(false,false,true),Some(b"INPUT LOST - CHECK COMMAND".as_slice()));
        s.note(Outcome::SaveUncertain);s.note(Outcome::Reply([0;128]));
        assert_eq!(s.warning(false,true,true),Some(b"UNCERTAIN SAVE / INPUT LOST".as_slice()));
        assert_eq!(s.warning(false,true,false),Some(b"SAVE UNCERTAIN - WRITES LOCKED".as_slice()));
    }
}
