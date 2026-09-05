//! Experimental public-input-only Ed25519 verification from RFC8032.
//! RAR-owned arithmetic; no signing/key generation or secret-scalar API.
//! Variable-time branches are permitted only because every input is public.
//! Not a production-audited verifier. Strict point policy is documented in README.
use crate::sha512::Sha512;
const MASK:u64=(1u64<<51)-1;
const ORDER:[u8;32]=[0xed,0xd3,0xf5,0x5c,0x1a,0x63,0x12,0x58,0xd6,0x9c,0xf7,0xa2,0xde,0xf9,0xde,0x14,
0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0x10];
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
struct Field([u64;5]);
impl Field{
    const ZERO:Self=Self([0;5]);
    const ONE:Self=Self([1,0,0,0,0]);
    const D:Self=Self([0x34dca135978a3,0x1a8283b156ebd,0x5e7a26001c029,0x739c663a03cbb,0x52036cee2b6ff]);
    const SQRT_M1:Self=Self([0x61b274a0ea0b0,0xd5a5fc8f189d,0x7ef5e9cbd0c60,0x78595a6804c9e,0x2b8324804fc1d]);
    /// All callers supply sums/products of canonical 51-bit limbs.
    /// Products fit below 2^110 in u128. Three carry sweeps normalize modulo p.
    /// The only normalized integers >=p have limbs1..4=MASK and limb0>=MASK-18.
    fn reduce(mut v:[u128;5])->Self{
        for _ in 0..3{
            for i in 0..4{let carry=v[i]>>51;v[i]&=MASK as u128;v[i+1]+=carry;}
            let carry=v[4]>>51;v[4]&=MASK as u128;v[0]+=19*carry;
        }
        let mut out=v.map(|x|x as u64);
        if out[1..].iter().all(|&x|x==MASK)&&out[0]>=MASK-18{
            out[0]-=MASK-18;out[1..].fill(0);
        }
        debug_assert!(out.iter().all(|&x|x<=MASK));
        Self(out)
    }
    fn add(self,b:Self)->Self{Self::reduce(core::array::from_fn(|i|self.0[i]as u128+b.0[i]as u128))}
    fn sub(self,b:Self)->Self{
        // Add 2p limbwise before subtraction, so every coefficient is nonnegative.
        Self::reduce(core::array::from_fn(|i|self.0[i]as u128+2*(if i==0{MASK-18}else{MASK})as u128-b.0[i]as u128))
    }
    fn mul(self,b:Self)->Self{
        let mut c=[0u128;5];
        for i in 0..5{for j in 0..5{
            let degree=i+j;let scale=if degree>=5{19}else{1};
            c[degree%5]+=self.0[i]as u128*b.0[j]as u128*scale;
        }}
        Self::reduce(c)
    }
    fn square(self)->Self{self.mul(self)}
    fn power(self,exponent:[u8;32])->Self{
        let mut value=Self::ONE;
        for bit in (0..256).rev(){
            value=value.square();
            if (exponent[bit/8]>>(bit%8))&1!=0{value=value.mul(self);}
        }
        value
    }
    fn inverse(self)->Option<Self>{
        if self==Self::ZERO{return None;}
        let mut e=[255;32];e[0]=0xeb;e[31]=0x7f;Some(self.power(e))
    }
    fn decode(bytes:[u8;32])->Option<Self>{
        if bytes[31]&0x80!=0{return None;}
        let mut words=[0u64;5];
        for bit in 0..255{words[bit/51]|=(((bytes[bit/8]>>(bit%8))&1)as u64)<<(bit%51);}
        let out=Self(words);
        if Self::reduce(words.map(|x|x as u128))!=out{return None;}
        Some(out)
    }
    fn encode(self)->[u8;32]{
        let mut out=[0u8;32];
        for bit in 0..255{out[bit/8]|=(((self.0[bit/51]>>(bit%51))&1)as u8)<<(bit%8);}
        out
    }
}
#[derive(Clone,Copy)]
struct Point{x:Field,y:Field,z:Field,t:Field}
impl Point{
    const IDENTITY:Self=Self{x:Field::ZERO,y:Field::ONE,z:Field::ONE,t:Field::ZERO};
    fn decode(mut bytes:[u8;32])->Option<Self>{
        let sign=bytes[31]>>7;bytes[31]&=127;
        let y=Field::decode(bytes)?;
        let y2=y.square();
        let a=y2.sub(Field::ONE).mul(Field::D.mul(y2).add(Field::ONE).inverse()?);
        let mut exponent=[255;32];exponent[0]=0xfe;exponent[31]=0x0f;
        let mut x=a.power(exponent); // (p+3)/8 = 2^252 - 2
        if x.square()!=a{x=x.mul(Field::SQRT_M1);}
        if x.square()!=a || (x==Field::ZERO&&sign!=0){return None;}
        if x.encode()[0]&1!=sign{x=Field::ZERO.sub(x);}
        Some(Self{x,y,z:Field::ONE,t:x.mul(y)})
    }
    fn add(self,b:Self)->Self{
        let a=self.y.sub(self.x).mul(b.y.sub(b.x));
        let bb=self.y.add(self.x).mul(b.y.add(b.x));
        let c=self.t.mul(Field::D.add(Field::D)).mul(b.t);
        let d=self.z.add(self.z).mul(b.z);
        let e=bb.sub(a);let f=d.sub(c);let g=d.add(c);let h=bb.add(a);
        Self{x:e.mul(f),y:g.mul(h),z:f.mul(g),t:e.mul(h)}
    }
    fn times(self,scalar:&[u8])->Self{
        let mut out=Self::IDENTITY;
        for bit in (0..scalar.len()*8).rev(){
            out=out.add(out);
            if (scalar[bit/8]>>(bit%8))&1!=0{out=out.add(self);}
        }
        out
    }
    fn same(self,b:Self)->bool{
        self.z!=Field::ZERO&&b.z!=Field::ZERO&&
            self.x.mul(b.z)==b.x.mul(self.z)&&self.y.mul(b.z)==b.y.mul(self.z)
    }
    fn prime_order(self)->bool{self.times(&ORDER).same(Self::IDENTITY)}
}
fn scalar_canonical(s:&[u8;32])->bool{
    for i in (0..32).rev(){if s[i]!=ORDER[i]{return s[i]<ORDER[i];}}
    false
}
/// Pure Ed25519 with canonical points/scalar, prime-subgroup R/A, nonidentity A.
/// This strict Alpha acceptance profile is not Ed25519ph/ctx and not batch verify.
/// Input is public and bounded to 4096 message bytes; false has no side effects.
/// A later manifest verifier must additionally bind the approved publisher key.
pub fn verify(public_key:&[u8;32],message:&[u8],signature:&[u8;64])->bool{
    if message.len()>4096{return false;}
    let r_bytes:[u8;32]=signature[..32].try_into().unwrap();
    let s:[u8;32]=signature[32..].try_into().unwrap();
    if !scalar_canonical(&s){return false;}
    let Some(a)=Point::decode(*public_key)else{return false;};
    let Some(r)=Point::decode(r_bytes)else{return false;};
    if a.same(Point::IDENTITY)||!a.prime_order()||!r.prime_order(){return false;}
    let mut base=[0x66;32];base[0]=0x58;
    let Some(b)=Point::decode(base)else{return false;};
    let mut hash=Sha512::new();
    if hash.update(&r_bytes).is_err()||hash.update(public_key).is_err()||hash.update(message).is_err(){return false;}
    b.times(&s).same(r.add(a.times(&hash.finish())))
}
#[cfg(test)] mod tests{
    use super::*;
    fn bytes<const N:usize>(s:&str)->[u8;N]{
        assert_eq!(s.len(),2*N);
        core::array::from_fn(|i|u8::from_str_radix(&s[2*i..2*i+2],16).unwrap())
    }
    const KEY:&str="d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
    const SIG:&str="e5564300c360ac729086e2cc806e828a84877f1eb8e5d974d873e065224901555fb8821590a33bacc61e39701cf9b46bd25bf5f0595bbe24655141438e7a100b";
    #[test] fn rfc8032_short_vectors(){
        assert!(verify(&bytes(KEY),b"",&bytes(SIG)));
        assert!(verify(&bytes("3d4017c3e843895a92b70aa74d1b7ebc9c982ccf2ec4968cc0cd55f12af4660c"),
            &[0x72],&bytes("92a009a9f0d4cab8720e820b5f642540a2b27b5416503f8fb3762223ebdb69da085ac1e43e15996e458f3613d0f11d8c387b2eaeb4302aeeb00d291612bb0c00")));
        assert!(verify(&bytes("fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025"),
            &[0xaf,0x82],&bytes("6291d657deec24024827e69c3abe01a30ce548a284743a445e3680d7db5ac3ac18ff9b538d16f290aeebd9c97254c71b36c5459425fcf4b2c15eb67a833b5c1d00")));
    }
    #[test] fn altered_signature_key_message_and_bounds(){
        let key=bytes(KEY);let sig=bytes(SIG);
        for i in [0,15,31,32,47,63]{let mut wrong=sig;wrong[i]^=1;assert!(!verify(&key,b"",&wrong));}
        assert!(!verify(&key,b"x",&sig));
        let mut wrong=key;wrong[0]^=1;assert!(!verify(&wrong,b"",&sig));
        assert!(!verify(&key,&[0;4097],&sig));
    }
    #[test] fn rejects_noncanonical_scalar_points_and_small_order_key(){
        let mut sig=bytes(SIG);sig[32..].copy_from_slice(&ORDER);
        assert!(!verify(&bytes(KEY),b"",&sig));
        let mut p=[255;32];p[0]=0xed;p[31]=127;
        assert!(Field::decode(p).is_none());assert!(Point::decode(p).is_none());
        let mut id=[0;32];id[0]=1;
        assert!(!verify(&id,b"",&bytes(SIG)));
        id[31]=128;assert!(Point::decode(id).is_none()); // x=0 with sign=1
        assert!(!verify(&[0;32],b"",&bytes(SIG))); // order4
    }
    #[test] fn field_carry_and_inverse_boundaries(){
        let mut pm1=[255;32];pm1[0]=0xec;pm1[31]=127;
        let m=Field::decode(pm1).unwrap();
        assert_eq!(m.add(Field::ONE),Field::ZERO);
        assert_eq!(m.square(),Field::ONE);
        assert_eq!(Field::ZERO.sub(Field::ONE),m);
        assert_eq!(m.inverse(),Some(m));assert_eq!(Field::ZERO.inverse(),None);
        for n in 1..50{
            let mut value=[0;32];value[0]=n;
            let x=Field::decode(value).unwrap();
            assert_eq!(x.mul(x.inverse().unwrap()),Field::ONE);
            assert_eq!(x.encode(),value);
        }
    }
}
