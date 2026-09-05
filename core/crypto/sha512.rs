//! RAR-owned SHA-512, FIPS 180-4. No allocation, unsafe code or external crate.
//! Internal Alpha primitive; not an audited production cryptographic module.
const IV:[u64;8]=[
0x6a09e667f3bcc908,0xbb67ae8584caa73b,0x3c6ef372fe94f82b,0xa54ff53a5f1d36f1,
0x510e527fade682d1,0x9b05688c2b3e6c1f,0x1f83d9abfb41bd6b,0x5be0cd19137e2179];
const K:[u64;80]=[
0x428a2f98d728ae22,0x7137449123ef65cd,0xb5c0fbcfec4d3b2f,0xe9b5dba58189dbbc,
0x3956c25bf348b538,0x59f111f1b605d019,0x923f82a4af194f9b,0xab1c5ed5da6d8118,
0xd807aa98a3030242,0x12835b0145706fbe,0x243185be4ee4b28c,0x550c7dc3d5ffb4e2,
0x72be5d74f27b896f,0x80deb1fe3b1696b1,0x9bdc06a725c71235,0xc19bf174cf692694,
0xe49b69c19ef14ad2,0xefbe4786384f25e3,0x0fc19dc68b8cd5b5,0x240ca1cc77ac9c65,
0x2de92c6f592b0275,0x4a7484aa6ea6e483,0x5cb0a9dcbd41fbd4,0x76f988da831153b5,
0x983e5152ee66dfab,0xa831c66d2db43210,0xb00327c898fb213f,0xbf597fc7beef0ee4,
0xc6e00bf33da88fc2,0xd5a79147930aa725,0x06ca6351e003826f,0x142929670a0e6e70,
0x27b70a8546d22ffc,0x2e1b21385c26c926,0x4d2c6dfc5ac42aed,0x53380d139d95b3df,
0x650a73548baf63de,0x766a0abb3c77b2a8,0x81c2c92e47edaee6,0x92722c851482353b,
0xa2bfe8a14cf10364,0xa81a664bbc423001,0xc24b8b70d0f89791,0xc76c51a30654be30,
0xd192e819d6ef5218,0xd69906245565a910,0xf40e35855771202a,0x106aa07032bbd1b8,
0x19a4c116b8d2d0c8,0x1e376c085141ab53,0x2748774cdf8eeb99,0x34b0bcb5e19b48a8,
0x391c0cb3c5c95a63,0x4ed8aa4ae3418acb,0x5b9cca4f7763e373,0x682e6ff3d6b2b8a3,
0x748f82ee5defb2fc,0x78a5636f43172f60,0x84c87814a1f0ab72,0x8cc702081a6439ec,
0x90befffa23631e28,0xa4506cebde82bde9,0xbef9a3f7b2c67915,0xc67178f2e372532b,
0xca273eceea26619c,0xd186b8c721c0c207,0xeada7dd6cde0eb1e,0xf57d4f7fee6ed178,
0x06f067aa72176fba,0x0a637dc5a2c898a6,0x113f9804bef90dae,0x1b710b35131c471b,
0x28db77f523047d84,0x32caab7b40c72493,0x3c9ebe0a15c9bebc,0x431d67c49c100d4c,
0x4cc5d4becb3e42b6,0x597f299cfc657e2a,0x5fcb6fab3ad6faec,0x6c44198c4a475817];
#[derive(Clone)]
pub struct Sha512{state:[u64;8],block:[u8;128],used:usize,bytes:u128}
impl Sha512{
    pub const fn new()->Self{Self{state:IV,block:[0;128],used:0,bytes:0}}
    /// Rejects lengths whose bit count cannot be encoded; state unchanged on error.
    pub fn update(&mut self,mut input:&[u8])->Result<(),()>{
        let total=self.bytes.checked_add(input.len()as u128).filter(|&n|n<=u128::MAX/8).ok_or(())?;
        self.bytes=total;
        if self.used!=0{
            let take=(128-self.used).min(input.len());
            self.block[self.used..self.used+take].copy_from_slice(&input[..take]);
            self.used+=take;input=&input[take..];
            if self.used<128{return Ok(());}
            compress(&mut self.state,&self.block);self.used=0;
        }
        while input.len()>=128{
            compress(&mut self.state,input[..128].try_into().unwrap());input=&input[128..];
        }
        self.block[..input.len()].copy_from_slice(input);self.used=input.len();Ok(())
    }
    pub fn finish(mut self)->[u8;64]{
        self.block[self.used]=0x80;self.used+=1;
        self.block[self.used..].fill(0);
        if self.used>112{compress(&mut self.state,&self.block);self.block.fill(0);}
        self.block[112..].copy_from_slice(&(self.bytes*8).to_be_bytes());
        compress(&mut self.state,&self.block);
        let mut out=[0;64];
        for (chunk,word) in out.chunks_exact_mut(8).zip(self.state){chunk.copy_from_slice(&word.to_be_bytes());}
        out
    }
}
pub fn digest(bytes:&[u8])->[u8;64]{
    // A single addressable Rust slice cannot overflow the u128 bit counter.
    let mut h=Sha512::new();h.update(bytes).unwrap();h.finish()
}
fn compress(state:&mut[u64;8],block:&[u8;128]){
    let mut w=[0u64;80];
    for (word,bytes) in w[..16].iter_mut().zip(block.chunks_exact(8)){*word=u64::from_be_bytes(bytes.try_into().unwrap());}
    for i in 16..80{
        let x=w[i-15];let y=w[i-2];
        w[i]=w[i-16].wrapping_add(x.rotate_right(1)^x.rotate_right(8)^(x>>7))
            .wrapping_add(w[i-7]).wrapping_add(y.rotate_right(19)^y.rotate_right(61)^(y>>6));
    }
    let [mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h]=*state;
    for i in 0..80{
        let t=h.wrapping_add(e.rotate_right(14)^e.rotate_right(18)^e.rotate_right(41))
            .wrapping_add((e&f)^(!e&g)).wrapping_add(K[i]).wrapping_add(w[i]);
        let u=(a.rotate_right(28)^a.rotate_right(34)^a.rotate_right(39)).wrapping_add((a&b)^(a&c)^(b&c));
        h=g;g=f;f=e;e=d.wrapping_add(t);d=c;c=b;b=a;a=t.wrapping_add(u);
    }
    for (x,y) in state.iter_mut().zip([a,b,c,d,e,f,g,h]){*x=x.wrapping_add(y);}
}
#[cfg(test)] mod tests{
    use super::*;
    fn hex(x:[u8;64])->String{x.iter().map(|v|format!("{v:02x}")).collect()}
    #[test] fn official_empty_and_abc(){
        assert_eq!(hex(digest(b"")),"cf83e1357eefb8bdf1542850d66d8007d620e4050b5715dc83f4a921d36ce9ce47d0d13c5d85f2b0ff8318d2877eec2f63b931bd47417a81a538327af927da3e");
        assert_eq!(hex(digest(b"abc")),"ddaf35a193617abacc417349ae20413112e6fa4e89a97ea20a9eeee64b55d39a2192992a274fc1a836ba3c23a3feebbd454d4423643ce80e2a9ac94fa54ca49f");
    }
    #[test] fn official_long_vector(){
        assert_eq!(hex(digest(b"abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu")),"8e959b75dae313da8cf4f72814fc143f8f7779c6eb9f7fa17299aeadb6889018501d289e4900f7e4331b99dec4b5433ac7d329eeb6dd26545e96e55b874be909");
    }
    #[test] fn incremental_boundaries_and_empty_updates(){
        let bytes:Vec<u8>=(0..4097).map(|x|(x*29)as u8).collect();
        for length in [0,1,111,112,113,127,128,129,255,256,4097]{
            for width in [1,7,111,112,127,128,129]{
                let mut h=Sha512::new();
                for chunk in bytes[..length].chunks(width){h.update(chunk).unwrap();h.update(&[]).unwrap();}
                assert_eq!(h.finish(),digest(&bytes[..length]));
            }
        }
    }
    #[test] fn length_overflow_fails_without_mutation(){
        let mut h=Sha512::new();h.bytes=u128::MAX/8;
        let before=h.clone();assert_eq!(h.update(&[1]),Err(()));
        assert_eq!(h.bytes,before.bytes);assert_eq!(h.state,before.state);assert_eq!(h.block,before.block);
    }
}
