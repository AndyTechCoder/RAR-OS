use std::fmt;
use std::io::Read;

pub fn sha256_hex(input: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input);
    hasher.finish_hex()
}

pub fn sha256_reader<R: Read>(reader: &mut R) -> std::io::Result<String> {
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 { break; }
        hasher.update(&buffer[..count]);
    }
    Ok(hasher.finish_hex())
}

struct Sha256 { state: [u32; 8], block: [u8; 64], block_len: usize, total_bytes: u64 }

impl Sha256 {
    const INITIAL: [u32; 8] = [0x6a09e667,0xbb67ae85,0x3c6ef372,0xa54ff53a,0x510e527f,0x9b05688c,0x1f83d9ab,0x5be0cd19];
    const K: [u32;64] = [
        0x428a2f98,0x71374491,0xb5c0fbcf,0xe9b5dba5,0x3956c25b,0x59f111f1,0x923f82a4,0xab1c5ed5,
        0xd807aa98,0x12835b01,0x243185be,0x550c7dc3,0x72be5d74,0x80deb1fe,0x9bdc06a7,0xc19bf174,
        0xe49b69c1,0xefbe4786,0x0fc19dc6,0x240ca1cc,0x2de92c6f,0x4a7484aa,0x5cb0a9dc,0x76f988da,
        0x983e5152,0xa831c66d,0xb00327c8,0xbf597fc7,0xc6e00bf3,0xd5a79147,0x06ca6351,0x14292967,
        0x27b70a85,0x2e1b2138,0x4d2c6dfc,0x53380d13,0x650a7354,0x766a0abb,0x81c2c92e,0x92722c85,
        0xa2bfe8a1,0xa81a664b,0xc24b8b70,0xc76c51a3,0xd192e819,0xd6990624,0xf40e3585,0x106aa070,
        0x19a4c116,0x1e376c08,0x2748774c,0x34b0bcb5,0x391c0cb3,0x4ed8aa4a,0x5b9cca4f,0x682e6ff3,
        0x748f82ee,0x78a5636f,0x84c87814,0x8cc70208,0x90befffa,0xa4506ceb,0xbef9a3f7,0xc67178f2];
    fn new() -> Self { Self { state: Self::INITIAL, block: [0;64], block_len: 0, total_bytes: 0 } }
    fn update(&mut self, mut input: &[u8]) {
        self.total_bytes = self.total_bytes.wrapping_add(input.len() as u64);
        if self.block_len != 0 {
            let count = (64-self.block_len).min(input.len());
            self.block[self.block_len..self.block_len+count].copy_from_slice(&input[..count]);
            self.block_len += count; input = &input[count..];
            if self.block_len == 64 { let block=self.block; self.compress(&block); self.block_len=0; }
        }
        while input.len() >= 64 { self.compress(&input[..64]); input=&input[64..]; }
        if !input.is_empty() { self.block[..input.len()].copy_from_slice(input); self.block_len=input.len(); }
    }
    fn compress(&mut self, block: &[u8]) {
        let mut w=[0u32;64]; for (i,b) in block.chunks_exact(4).enumerate(){w[i]=u32::from_be_bytes([b[0],b[1],b[2],b[3]]);}
        for i in 16..64 { let s0=w[i-15].rotate_right(7)^w[i-15].rotate_right(18)^(w[i-15]>>3); let s1=w[i-2].rotate_right(17)^w[i-2].rotate_right(19)^(w[i-2]>>10); w[i]=w[i-16].wrapping_add(s0).wrapping_add(w[i-7]).wrapping_add(s1); }
        let [mut a,mut b,mut c,mut d,mut e,mut f,mut g,mut h]=self.state;
        for i in 0..64 { let s1=e.rotate_right(6)^e.rotate_right(11)^e.rotate_right(25); let ch=(e&f)^((!e)&g); let t1=h.wrapping_add(s1).wrapping_add(ch).wrapping_add(Self::K[i]).wrapping_add(w[i]); let s0=a.rotate_right(2)^a.rotate_right(13)^a.rotate_right(22); let maj=(a&b)^(a&c)^(b&c); let t2=s0.wrapping_add(maj); h=g;g=f;f=e;e=d.wrapping_add(t1);d=c;c=b;b=a;a=t1.wrapping_add(t2); }
        for (slot,value) in self.state.iter_mut().zip([a,b,c,d,e,f,g,h]) { *slot=slot.wrapping_add(value); }
    }
    fn finish_hex(mut self) -> String {
        let bits=self.total_bytes.wrapping_mul(8); self.block[self.block_len]=0x80; self.block_len+=1;
        if self.block_len>56 { self.block[self.block_len..].fill(0); let block=self.block; self.compress(&block); self.block=[0;64]; self.block_len=0; }
        self.block[self.block_len..56].fill(0); self.block[56..64].copy_from_slice(&bits.to_be_bytes()); let block=self.block; self.compress(&block);
        let mut out=String::with_capacity(64); for word in self.state { use fmt::Write as _; write!(&mut out,"{word:08x}").expect("String write"); } out
    }
}
