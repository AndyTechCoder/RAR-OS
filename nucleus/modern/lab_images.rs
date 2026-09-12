//! Page-isolated immutable public-laboratory packages embedded in the boot PE.
//! No secret key, arbitrary path input or mutable package source exists here.
#[cfg(all(rar_signed_updates,not(rar_modern_compile_only),not(test)))]
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
    #[cfg(all(rar_signed_updates,not(rar_modern_compile_only),not(test)))]
    {embedded::all().map(Some)}
    #[cfg(any(not(rar_signed_updates),rar_modern_compile_only,test))]
    {[None;5]}
}

#[path="../../core/crypto/sha256.rs"] mod sha256;
/// Derive component identity from the authenticated containing boot image.
/// This is public laboratory provenance, NOT hardware-backed secure boot.
fn component_hash(bank:&[u8],length:usize)->Result<[u8;32],()>{
    if !(896..=2_097_536).contains(&length)||length>bank.len(){return Err(());}
    let hash=sha256::sha256(&bank[..length]).map_err(|_|())?;
    if hash==[0;32]{return Err(());}Ok(hash)
}
pub(crate) fn factory_hash()->Result<[u8;32],()>{
    let (bank,length)=all()[4].ok_or(())?;
    component_hash(bank,length)
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn factory_component_hash_excludes_page_padding_and_checks_exact_bounds(){
        let mut bank=vec![7u8;4096];
        let expected=sha256::sha256(&bank[..896]).unwrap();
        assert_eq!(component_hash(&bank,896),Ok(expected));
        bank[896..].fill(0xa5);
        assert_eq!(component_hash(&bank,896),Ok(expected));
        bank[895]^=1;assert_ne!(component_hash(&bank,896),Ok(expected));
        for length in [0,895,4097,2_097_537,usize::MAX]{
            assert_eq!(component_hash(&bank,length),Err(()));
        }
        assert_eq!(component_hash(&[0;512],896),Err(()));
    }
    #[test]fn absent_factory_cannot_fall_back_to_a_zero_identity(){
        #[cfg(any(not(rar_signed_updates),rar_modern_compile_only,test))]
        assert_eq!(factory_hash(),Err(()));
    }
}
