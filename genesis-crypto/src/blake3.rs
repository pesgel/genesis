use crate::{Crypto, SingerCrypto, VerifyCrypto};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use std::io::Read;

pub struct Blake3 {
    key: [u8; 32],
}

impl Blake3 {
    pub fn new(key: [u8; 32]) -> Self {
        Blake3 { key }
    }
}
impl Crypto for Blake3 {}
impl SingerCrypto for Blake3 {
    fn sign(&self, data: &mut dyn Read) -> anyhow::Result<String> {
        let mut input = Vec::new();
        data.read_to_end(&mut input)?;
        let hash = blake3::keyed_hash(&self.key, &input);
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(hash.as_bytes());
        Ok(encoded)
    }
}
impl VerifyCrypto for Blake3 {
    fn verify(&self, sig: &[u8], data: &mut dyn Read) -> anyhow::Result<bool> {
        let mut input = Vec::new();
        data.read_to_end(&mut input)?;
        let hash = blake3::keyed_hash(&self.key, &input);
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(hash.as_bytes());
        anyhow::Ok(encoded.as_bytes().eq(sig))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_blake3() {
        let b3 = Blake3::new(b"1qw387yrhdj1823hdbcnem,sj23nv!,s".to_owned());
        let mut reader = "hello world".as_bytes();
        let _res = b3.sign(&mut reader).unwrap();
        let mut reader1 = "hello world".as_bytes();
        let input = "XgyB9wK7phmUyesdw7tgFDd3FdZJ4UFdinqeq063B9Q".as_bytes();
        let res = b3.verify(input, &mut reader1).unwrap();
        assert!(res);
    }
}
