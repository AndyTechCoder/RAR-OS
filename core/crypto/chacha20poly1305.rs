//! Experimental RFC8439 AEAD, not activated in an OS image.
//! Caller MUST ensure nonce uniqueness for each key across crashes and rollback.
//! Fixed secret-dependent arithmetic shape; no compiled constant-time claim yet.
#![forbid(unsafe_code)]

pub const MAX_DATA:usize=4096;
pub const MAX_AAD:usize=256;
#[derive(Clone,Copy,Debug,PartialEq,Eq)]
pub enum Error { Bounds, Authentication }
fn word(b:&[u8],i:usize)->u32 {
    u32::from_le_bytes([b[i],b[i+1],b[i+2],b[i+3]])
}
fn quarter(x:&mut[u32;16],a:usize,b:usize,c:usize,d:usize) {
    x[a]=x[a].wrapping_add(x[b]);x[d]=(x[d]^x[a]).rotate_left(16);
    x[c]=x[c].wrapping_add(x[d]);x[b]=(x[b]^x[c]).rotate_left(12);
    x[a]=x[a].wrapping_add(x[b]);x[d]=(x[d]^x[a]).rotate_left(8);
    x[c]=x[c].wrapping_add(x[d]);x[b]=(x[b]^x[c]).rotate_left(7);
}
fn block(key:&[u8;32],nonce:&[u8;12],counter:u32)->[u8;64] {
    let mut initial=[0u32;16];
    initial[..4].copy_from_slice(&[0x61707865,0x3320646e,0x79622d32,0x6b206574]);
    for i in 0..8 {initial[4+i]=word(key,4*i);}
    initial[12]=counter;
    for i in 0..3 {initial[13+i]=word(nonce,4*i);}
    let mut x=initial;
    for _ in 0..10 {
        quarter(&mut x,0,4,8,12);quarter(&mut x,1,5,9,13);
        quarter(&mut x,2,6,10,14);quarter(&mut x,3,7,11,15);
        quarter(&mut x,0,5,10,15);quarter(&mut x,1,6,11,12);
        quarter(&mut x,2,7,8,13);quarter(&mut x,3,4,9,14);
    }
    let mut out=[0;64];
    for i in 0..16 {out[4*i..4*i+4].copy_from_slice(&x[i].wrapping_add(initial[i]).to_le_bytes());}
    out
}
fn xor(key:&[u8;32],nonce:&[u8;12],data:&mut[u8]) {
    // Public API has already bounded data to64 blocks, so counter cannot wrap.
    for (i,chunk) in data.chunks_mut(64).enumerate() {
        let stream=block(key,nonce,1+i as u32);
        for (b,k) in chunk.iter_mut().zip(stream) {*b^=k;}
    }
}
const MASK:u64=(1<<26)-1;
fn limbs(bytes:&[u8;17])->[u64;5] {
    [word(bytes,0)as u64&MASK,(word(bytes,3)as u64>>2)&MASK,
     (word(bytes,6)as u64>>4)&MASK,(word(bytes,9)as u64>>6)&MASK,
     (word(bytes,12)as u64>>8)|((bytes[16]as u64)<<24)]
}
struct Poly {h:[u64;5],r:[u64;5],s:u128}
impl Poly {
    fn new(key:&[u8;32])->Self {
        let mut r=[0u8;17];r[..16].copy_from_slice(&key[..16]);
        for i in [3,7,11,15] {r[i]&=15;}
        for i in [4,8,12] {r[i]&=252;}
        let mut s=[0;16];s.copy_from_slice(&key[16..]);
        Self {h:[0;5],r:limbs(&r),s:u128::from_le_bytes(s)}
    }
    fn absorb(&mut self,b:&[u8;17]) {
        let n=limbs(b);let mut a=[0u64;5];
        for i in 0..5 {a[i]=self.h[i]+n[i];}
        // Canonical inputs: a<2^27, r<2^26. At most five terms,
        // each multiplied by at most5; every accumulator stays below2^58.
        let mut product=[0u64;5];
        for i in 0..5 {for j in 0..5 {
            let k=i+j;
            product[k%5]+=a[i]*self.r[j]*if k>=5 {5} else {1};
        }}
        // Three fixed carry sweeps suffice: first reduces the wide product,
        // second handles the wrapped carry, third normalizes any carry chain.
        for _ in 0..3 {
            for i in 0..4 {let carry=product[i]>>26;product[i]&=MASK;product[i+1]+=carry;}
            let carry=product[4]>>26;product[4]&=MASK;product[0]+=carry*5;
        }
        // Conditional subtraction of p=2^130-5, without a secret branch.
        let mut reduced=[0u64;5];let mut carry=5u64;
        for i in 0..5 {let sum=product[i]+carry;reduced[i]=sum&MASK;carry=sum>>26;}
        let select=0u64.wrapping_sub(carry);
        for i in 0..5 {self.h[i]=(reduced[i]&select)|(product[i]&!select);}
    }
    fn padded(&mut self,bytes:&[u8]) {
        for chunk in bytes.chunks(16) {
            let mut b=[0u8;17];b[..chunk.len()].copy_from_slice(chunk);
            // AEAD padding is part of the authenticated full16-byte block.
            b[16]=1;self.absorb(&b);
        }
    }
    fn finish(self)->[u8;16] {
        let low=(self.h[0]as u128)|((self.h[1]as u128)<<26)|
            ((self.h[2]as u128)<<52)|((self.h[3]as u128)<<78)|
            (((self.h[4]&0x00ff_ffff)as u128)<<104);
        low.wrapping_add(self.s).to_le_bytes()
    }
}
fn authenticate(key:&[u8;32],nonce:&[u8;12],aad:&[u8],cipher:&[u8])->[u8;16] {
    let first=block(key,nonce,0);let mut one_time=[0;32];one_time.copy_from_slice(&first[..32]);
    let mut p=Poly::new(&one_time);p.padded(aad);p.padded(cipher);
    let mut lengths=[0u8;17];
    lengths[..8].copy_from_slice(&(aad.len()as u64).to_le_bytes());
    lengths[8..16].copy_from_slice(&(cipher.len()as u64).to_le_bytes());
    lengths[16]=1;p.absorb(&lengths);p.finish()
}
fn bounds(aad:&[u8],data:&[u8])->Result<(),Error> {
    if aad.len()>MAX_AAD||data.len()>MAX_DATA {Err(Error::Bounds)}else{Ok(())}
}
/// Encrypt in place. Bounds failure leaves data unchanged.
/// Nonces must never repeat under the same key, including abandoned writes.
pub fn seal(key:&[u8;32],nonce:&[u8;12],aad:&[u8],data:&mut[u8])->Result<[u8;16],Error> {
    bounds(aad,data)?;xor(key,nonce,data);Ok(authenticate(key,nonce,aad,data))
}
/// Authenticate before releasing any plaintext. Every error preserves ciphertext.
/// No truncated tags, unauthenticated decryption or fallback mode is exposed.
pub fn open(key:&[u8;32],nonce:&[u8;12],aad:&[u8],data:&mut[u8],tag:&[u8;16])->Result<(),Error> {
    bounds(aad,data)?;
    let expected=authenticate(key,nonce,aad,data);let mut different=0u8;
    for i in 0..16 {different|=tag[i]^expected[i];}
    if different!=0 {return Err(Error::Authentication);}
    xor(key,nonce,data);Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    fn hex(s:&str)->Vec<u8> {
        assert_eq!(s.len()%2,0);
        s.as_bytes().chunks(2).map(|p|u8::from_str_radix(core::str::from_utf8(p).unwrap(),16).unwrap()).collect()
    }
    fn array<const N:usize>(s:&str)->[u8;N] {hex(s).try_into().unwrap()}
    fn mac(key:&[u8;32],message:&[u8])->[u8;16] {
        let mut p=Poly::new(key);
        for chunk in message.chunks(16) {
            let mut b=[0;17];b[..chunk.len()].copy_from_slice(chunk);b[chunk.len()]=1;p.absorb(&b);
        }
        p.finish()
    }
    #[test] fn rfc_quarter_and_block() {
        let mut x=[0;16];x[..4].copy_from_slice(&[0x11111111,0x01020304,0x9b8d6f43,0x01234567]);
        quarter(&mut x,0,1,2,3);
        assert_eq!(&x[..4],&[0xea2a92f4,0xcb1cf8ce,0x4581472e,0x5881c4bb]);
        let key=core::array::from_fn(|i|i as u8);
        assert_eq!(block(&key,&array("000000090000004a00000000"),1),array(
            "10f1e7e4d13b5915500fdd1fa32071c4c7d1f4c733c068030422aa9ac3d46c4ed2826446079faa0914c2d705d98b02a2b5129cd1de164eb9cbd083e8a2503c4e"));
    }
    #[test] fn rfc_poly_key_and_mac() {
        let key=core::array::from_fn(|i|0x80+i as u8);
        assert_eq!(&block(&key,&array("000000000001020304050607"),0)[..32],
            &hex("8ad5a08b905f81cc815040274ab29471a833b637e3fd0da508dbb8e2fdd1a646"));
        assert_eq!(mac(&array("85d6be7857556d337f4452fe42d506a80103808afb0db2fd4abff6af4149f51b"),
            b"Cryptographic Forum Research Group"),array("a8061dc1305136c6c22b8baf0c0127a9"));
    }
    #[test] fn rfc_aead_and_all_single_byte_tampering() {
        let key=core::array::from_fn(|i|0x80+i as u8);
        let nonce=array("070000004041424344454647");let aad=hex("50515253c0c1c2c3c4c5c6c7");
        let plaintext=b"Ladies and Gentlemen of the class of '99: If I could offer you only one tip for the future, sunscreen would be it.";
        let cipher=hex("d31a8d34648e60db7b86afbc53ef7ec2a4aded51296e08fea9e2b5a736ee62d63dbea45e8ca9671282fafb69da92728b1a71de0a9e060b2905d6a5b67ecd3b3692ddbd7f2d778b8c9803aee328091b58fab324e4fad675945585808b4831d7bc3ff4def08e4b7a9de576d26586cec64b6116");
        let expected=array("1ae10b594f09e26a7e902ecbd0600691");let mut data=plaintext.to_vec();
        assert_eq!(seal(&key,&nonce,&aad,&mut data),Ok(expected));assert_eq!(data,cipher);
        open(&key,&nonce,&aad,&mut data,&expected).unwrap();assert_eq!(data,plaintext);
        for i in 0..cipher.len() {let mut bad=cipher.clone();bad[i]^=1;let original=bad.clone();
            assert_eq!(open(&key,&nonce,&aad,&mut bad,&expected),Err(Error::Authentication));assert_eq!(bad,original);}
        for i in 0..16 {let mut tag=expected;tag[i]^=1;let mut bad=cipher.clone();
            assert_eq!(open(&key,&nonce,&aad,&mut bad,&tag),Err(Error::Authentication));assert_eq!(bad,cipher);}
        for i in 0..aad.len() {let mut bad_aad=aad.clone();bad_aad[i]^=1;let mut bad=cipher.clone();
            assert_eq!(open(&key,&nonce,&bad_aad,&mut bad,&expected),Err(Error::Authentication));assert_eq!(bad,cipher);}
        for i in 0..12 {let mut bad_nonce=nonce;bad_nonce[i]^=1;let mut bad=cipher.clone();
            assert_eq!(open(&key,&bad_nonce,&aad,&mut bad,&expected),Err(Error::Authentication));assert_eq!(bad,cipher);}
        for i in 0..32 {let mut bad_key=key;bad_key[i]^=1;let mut bad=cipher.clone();
            assert_eq!(open(&bad_key,&nonce,&aad,&mut bad,&expected),Err(Error::Authentication));assert_eq!(bad,cipher);}
    }
    #[test] fn boundaries_roundtrip_and_errors_preserve_input() {
        let key=[17;32];let mut nonce=[0;12];let mut sequence=0u64;
        for size in [0,1,15,16,17,63,64,65,4095,4096] {for aad_len in [0,1,15,16,17,255,256] {
            sequence+=1;nonce[..8].copy_from_slice(&sequence.to_le_bytes());
            let aad=vec![23;aad_len];let mut data=vec![42;size];let before=data.clone();
            let tag=seal(&key,&nonce,&aad,&mut data).unwrap();open(&key,&nonce,&aad,&mut data,&tag).unwrap();
            assert_eq!(data,before);
        }}
        for (a,n) in [(257,1),(0,4097)] {
            let aad=vec![0;a];let mut data=vec![7;n];let before=data.clone();
            assert_eq!(seal(&key,&nonce,&aad,&mut data),Err(Error::Bounds));assert_eq!(data,before);
            assert_eq!(open(&key,&nonce,&aad,&mut data,&[0;16]),Err(Error::Bounds));assert_eq!(data,before);
        }
    }
    #[test] fn rfc_appendix_poly_1() {
        assert_eq!(mac(&array("0000000000000000000000000000000000000000000000000000000000000000"),&hex("00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")),array("00000000000000000000000000000000"));
    }
    #[test] fn rfc_appendix_poly_2() {
        assert_eq!(mac(&array("0000000000000000000000000000000036e5f6b5c5e06070f0efca96227a863e"),&hex("416e79207375626d697373696f6e20746f20746865204945544620696e74656e6465642062792074686520436f6e7472696275746f7220666f72207075626c69636174696f6e20617320616c6c206f722070617274206f6620616e204945544620496e7465726e65742d4472616674206f722052464320616e6420616e792073746174656d656e74206d6164652077697468696e2074686520636f6e74657874206f6620616e204945544620616374697669747920697320636f6e7369646572656420616e20224945544620436f6e747269627574696f6e222e20537563682073746174656d656e747320696e636c756465206f72616c2073746174656d656e747320696e20494554462073657373696f6e732c2061732077656c6c206173207772697474656e20616e6420656c656374726f6e696320636f6d6d756e69636174696f6e73206d61646520617420616e792074696d65206f7220706c6163652c207768696368206172652061646472657373656420746f")),array("36e5f6b5c5e06070f0efca96227a863e"));
    }
    #[test] fn rfc_appendix_poly_3() {
        assert_eq!(mac(&array("36e5f6b5c5e06070f0efca96227a863e00000000000000000000000000000000"),&hex("416e79207375626d697373696f6e20746f20746865204945544620696e74656e6465642062792074686520436f6e7472696275746f7220666f72207075626c69636174696f6e20617320616c6c206f722070617274206f6620616e204945544620496e7465726e65742d4472616674206f722052464320616e6420616e792073746174656d656e74206d6164652077697468696e2074686520636f6e74657874206f6620616e204945544620616374697669747920697320636f6e7369646572656420616e20224945544620436f6e747269627574696f6e222e20537563682073746174656d656e747320696e636c756465206f72616c2073746174656d656e747320696e20494554462073657373696f6e732c2061732077656c6c206173207772697474656e20616e6420656c656374726f6e696320636f6d6d756e69636174696f6e73206d61646520617420616e792074696d65206f7220706c6163652c207768696368206172652061646472657373656420746f")),array("f3477e7cd95417af89a6b8794c310cf0"));
    }
    #[test] fn rfc_appendix_poly_4() {
        assert_eq!(mac(&array("1c9240a5eb55d38af333888604f6b5f0473917c1402b80099dca5cbc207075c0"),&hex("2754776173206272696c6c69672c20616e642074686520736c6974687920746f7665730a446964206779726520616e642067696d626c6520696e2074686520776162653a0a416c6c206d696d737920776572652074686520626f726f676f7665732c0a416e6420746865206d6f6d65207261746873206f757467726162652e")),array("4541669a7eaaee61e708dc7cbcc5eb62"));
    }
    #[test] fn rfc_appendix_poly_5() {
        assert_eq!(mac(&array("0200000000000000000000000000000000000000000000000000000000000000"),&hex("ffffffffffffffffffffffffffffffff")),array("03000000000000000000000000000000"));
    }
    #[test] fn rfc_appendix_poly_6() {
        assert_eq!(mac(&array("02000000000000000000000000000000ffffffffffffffffffffffffffffffff"),&hex("02000000000000000000000000000000")),array("03000000000000000000000000000000"));
    }
    #[test] fn rfc_appendix_poly_7() {
        assert_eq!(mac(&array("0100000000000000000000000000000000000000000000000000000000000000"),&hex("fffffffffffffffffffffffffffffffff0ffffffffffffffffffffffffffffff11000000000000000000000000000000")),array("05000000000000000000000000000000"));
    }
    #[test] fn rfc_appendix_poly_8() {
        assert_eq!(mac(&array("0100000000000000000000000000000000000000000000000000000000000000"),&hex("fffffffffffffffffffffffffffffffffbfefefefefefefefefefefefefefefe01010101010101010101010101010101")),array("00000000000000000000000000000000"));
    }
    #[test] fn rfc_appendix_poly_9() {
        assert_eq!(mac(&array("0200000000000000000000000000000000000000000000000000000000000000"),&hex("fdffffffffffffffffffffffffffffff")),array("faffffffffffffffffffffffffffffff"));
    }
    #[test] fn rfc_appendix_poly_10() {
        assert_eq!(mac(&array("0100000000000000040000000000000000000000000000000000000000000000"),&hex("e33594d7505e43b900000000000000003394d7505e4379cd01000000000000000000000000000000000000000000000001000000000000000000000000000000")),array("14000000000000005500000000000000"));
    }
    #[test] fn rfc_appendix_poly_11() {
        assert_eq!(mac(&array("0100000000000000040000000000000000000000000000000000000000000000"),&hex("e33594d7505e43b900000000000000003394d7505e4379cd010000000000000000000000000000000000000000000000")),array("13000000000000000000000000000000"));
    }
}
