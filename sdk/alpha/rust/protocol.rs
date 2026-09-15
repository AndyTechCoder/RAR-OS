//! Experimental app operation payloads. No caller identity is inferred here.
#![forbid(unsafe_code)]
use super::wire::{self,Error,Message};
pub const OK:u8=0;
pub const INVALID:u8=1;
pub const DENIED:u8=2;
pub const UNAVAILABLE:u8=3;
pub const READ_ONLY:u8=4;
pub const INDETERMINATE:u8=5;
pub const BUSY:u8=6;
pub const EXHAUSTED:u8=7;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Paint<'a>{Begin(u8),Line{row:u8,text:&'a[u8]},Commit}
pub fn paint(message:&Message)->Result<Paint<'_>,Error>{
    if message.operation()!=wire::PAINT||message.status()!=OK{return Err(Error::Invalid);}
    match message.payload(){
        [0,count] if (1..=6).contains(count)=>Ok(Paint::Begin(*count)),
        [1,row,text @ ..] if *row<6&&text.len()<=48&&text.iter().all(|b|(32..=126).contains(b))
            =>Ok(Paint::Line{row:*row,text}),
        [2]=>Ok(Paint::Commit),
        _=>Err(Error::Invalid),
    }
}
pub fn begin(sequence:u32,count:u8)->Result<Message,Error>{
    let m=Message::new(wire::PAINT,OK,sequence,&[0,count])?;paint(&m)?;Ok(m)
}
pub fn line(sequence:u32,row:u8,text:&[u8])->Result<Message,Error>{
    if text.len()>48{return Err(Error::Invalid);}
    let mut bytes=[0;50];bytes[0]=1;bytes[1]=row;bytes[2..2+text.len()].copy_from_slice(text);
    let m=Message::new(wire::PAINT,OK,sequence,&bytes[..2+text.len()])?;paint(&m)?;Ok(m)
}
pub fn commit(sequence:u32)->Result<Message,Error>{Message::new(wire::PAINT,OK,sequence,&[2])}
pub fn input(sequence:u32,key:u8)->Result<Message,Error>{
    let m=Message::new(wire::INPUT,OK,sequence,&[key])?;key_of(&m)?;Ok(m)
}
pub fn key_of(m:&Message)->Result<u8,Error>{
    if m.operation()!=wire::INPUT||m.status()!=OK{return Err(Error::Invalid);}
    match m.payload(){[key] if matches!(key,8|13|27|32..=126)=>Ok(*key),_=>Err(Error::Invalid)}
}
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Document<'a>{Read,Write(&'a[u8])}
pub fn document(m:&Message)->Result<Document<'_>,Error>{
    if m.status()!=OK{return Err(Error::Invalid);}
    match m.operation(){
        wire::READ_DOCUMENT if m.payload().is_empty()=>Ok(Document::Read),
        wire::WRITE_DOCUMENT if m.payload().len()<=64=>Ok(Document::Write(m.payload())),
        _=>Err(Error::Invalid),
    }
}
pub fn document_reply(m:&Message)->Result<(),Error>{
    if !matches!(m.operation(),wire::READ_DOCUMENT|wire::WRITE_DOCUMENT){return Err(Error::Invalid);}
    if m.status()!=OK{return if m.payload().is_empty(){Ok(())}else{Err(Error::Invalid)};}
    if m.operation()==wire::READ_DOCUMENT&&m.payload().len()<=64||
        m.operation()==wire::WRITE_DOCUMENT&&m.payload().is_empty(){Ok(())}
    else{Err(Error::Invalid)}
}
#[cfg(test)]mod tests{
    use super::*;
    #[test]fn operation_payloads_are_directional_bounded_and_canonical(){
        for count in 0..=255{assert_eq!(begin(1,count).is_ok(),(1..=6).contains(&count));}
        for row in 0..=255{assert_eq!(line(1,row,b"hello").is_ok(),row<6);}
        assert!(line(1,0,&[32;49]).is_err());assert!(line(1,0,b"\n").is_err());
        for key in 0..=255{assert_eq!(input(1,key).is_ok(),matches!(key,8|13|27|32..=126));}
        for n in 0..=112{
            let data=[b'a';112];
            let read=Message::new(wire::READ_DOCUMENT,0,1,&data[..n]).unwrap();
            assert_eq!(document(&read).is_ok(),n==0);
            assert_eq!(document_reply(&read).is_ok(),n<=64);
            let write=Message::new(wire::WRITE_DOCUMENT,0,1,&data[..n]).unwrap();
            assert_eq!(document(&write).is_ok(),n<=64);
            assert_eq!(document_reply(&write).is_ok(),n==0);
            let denied=Message::new(wire::WRITE_DOCUMENT,DENIED,1,&data[..n]).unwrap();
            assert!(document(&denied).is_err());assert_eq!(document_reply(&denied).is_ok(),n==0);
        }
        for op in [wire::TOOL,wire::DATAGRAM,wire::INPUT,wire::PAINT]{
            assert!(document(&Message::new(op,0,1,b"").unwrap()).is_err());
        }
    }
}
