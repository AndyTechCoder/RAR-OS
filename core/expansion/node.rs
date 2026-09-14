//! Small portable headless script engine. No allocation, OS calls or I/O.
#![forbid(unsafe_code)]
pub const PROGRAM_BYTES:usize=64;
pub const STEP_BUDGET:u32=64;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error{Program,Bounds,Overflow,Budget,Output}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub struct Machine{pub registers:[u32;8],pub output:[u32;4],pub count:usize,pub steps:u32}
impl Machine {
    pub const fn new()->Self{Self{registers:[0;8],output:[0;4],count:0,steps:0}}
    pub fn run(program:&[u8],inputs:[u32;4])->Result<Self,Error>{
        if program.is_empty()||program.len()>PROGRAM_BYTES{return Err(Error::Program);}
        let mut m=Self::new();let mut pc=0usize;
        loop{
            if m.steps==STEP_BUDGET{return Err(Error::Budget);}
            m.steps+=1;
            let op=*program.get(pc).ok_or(Error::Bounds)?;pc+=1;
            match op {
                0=>{if pc!=program.len(){return Err(Error::Program);}return Ok(m);},
                1|3=>{
                    let operands=program.get(pc..pc+2).ok_or(Error::Bounds)?;pc+=2;
                    let (a,b)=(operands[0]as usize,operands[1]as usize);
                    if a>=8{return Err(Error::Bounds);}
                    let value=if op==1{*inputs.get(b).ok_or(Error::Bounds)?}
                        else{m.registers[a].checked_add(*m.registers.get(b).ok_or(Error::Bounds)?).ok_or(Error::Overflow)?};
                    m.registers[a]=value;
                },
                2=>{
                    let operands=program.get(pc..pc+5).ok_or(Error::Bounds)?;pc+=5;
                    let r=m.registers.get_mut(operands[0]as usize).ok_or(Error::Bounds)?;
                    *r=u32::from_le_bytes(operands[1..].try_into().unwrap());
                },
                4=>{
                    let r=*program.get(pc).ok_or(Error::Bounds)? as usize;pc+=1;
                    if m.count==4{return Err(Error::Output);}
                    m.output[m.count]=*m.registers.get(r).ok_or(Error::Bounds)?;m.count+=1;
                },
                5=>{pc=*program.get(pc).ok_or(Error::Bounds)? as usize;if pc>=program.len(){return Err(Error::Bounds);}},
                _=>return Err(Error::Program),
            }
        }
    }
}
pub const DEMO:[u8;15]=[1,0,0,2,1,7,0,0,0,3,0,1,4,0,0];

#[cfg(test)]mod tests{
    use super::*;
    #[test]fn bounded_portable_sample(){
        for sample in [0,3,42,1000,u32::MAX-7]{
            let m=Machine::run(&DEMO,[sample,0,0,0]).unwrap();
            assert_eq!(m.output[0],sample+7);assert_eq!(m.count,1);assert_eq!(m.steps,5);
            assert!(core::mem::size_of::<Machine>()<=128);
        }
        assert_eq!(Machine::run(&DEMO,[u32::MAX,0,0,0]),Err(Error::Overflow));
    }
    #[test]fn malformed_and_unbounded_programs_fail(){
        for length in 0..DEMO.len(){assert!(Machine::run(&DEMO[..length],[0;4]).is_err());}
        for p in [&[5,0][..],&[1,8,0,0],&[1,0,4,0],&[2,9,0,0,0,0,0],
                  &[4,8,0],&[3,0,8,0],&[99,0],&[0,0],&[5,255]]{
            assert!(Machine::run(p,[0;4]).is_err());
        }
        assert_eq!(Machine::run(&[5,0],[0;4]),Err(Error::Budget));
        assert_eq!(Machine::run(&[4,0,4,0,4,0,4,0,4,0,0],[0;4]),Err(Error::Output));
        assert!(Machine::run(&[0;65],[0;4]).is_err());
    }
}
