//! Page-isolated immutable public-laboratory packages embedded in the boot PE.
//! No secret key, arbitrary path input or mutable package source exists here.
#[cfg(all(rar_signed_updates,not(rar_modern_compile_only)))]
mod embedded{
    #[repr(align(4096))]
    struct Aligned<const N:usize>([u8;N]);
    const fn pad<const N:usize>(bytes:&[u8])->[u8;N]{
        let mut out=[0u8;N];let mut i=0;
        while i<bytes.len(){out[i]=bytes[i];i+=1;}out
    }
    macro_rules! bank{
        ($raw:ident,$name:ident,$path:literal)=>{
            const $raw:&[u8]=include_bytes!($path);
            static $name:Aligned<{($raw.len()+4095)/4096*4096}>=Aligned(pad($raw));
        };
    }
    bank!(UPDATE_RAW,UPDATE,"/tmp/modern-settings-update.layer");
    bank!(HEALTH_RAW,HEALTH,"/tmp/modern-settings-bad-health.layer");
    bank!(SIGNATURE_RAW,SIGNATURE,"/tmp/modern-settings-bad-signature.layer");
    bank!(ABI_RAW,ABI,"/tmp/modern-settings-bad-abi.layer");
    bank!(FACTORY_RAW,FACTORY,"/tmp/modern-settings-factory.layer");
    pub fn all()->[(&'static[u8],usize);5]{
        [(&UPDATE.0,UPDATE_RAW.len()),(&HEALTH.0,HEALTH_RAW.len()),
            (&SIGNATURE.0,SIGNATURE_RAW.len()),(&ABI.0,ABI_RAW.len()),
            (&FACTORY.0,FACTORY_RAW.len())]
    }
}
pub fn all()->[Option<(&'static[u8],usize)>;5]{
    #[cfg(all(rar_signed_updates,not(rar_modern_compile_only)))]
    {embedded::all().map(Some)}
    #[cfg(any(not(rar_signed_updates),rar_modern_compile_only))]
    {[None;5]}
}
