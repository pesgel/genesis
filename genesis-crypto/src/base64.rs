use crate::{Crypto, SingerCrypto, VerifyCrypto};
use base64::Engine;
use base64::prelude::{BASE64_STANDARD, BASE64_URL_SAFE_NO_PAD};
use std::io::Read;

#[derive(Clone, Copy)]
pub enum Base64 {
    Standard,
    URLEncoded,
}

impl Crypto for Base64 {}

impl SingerCrypto for Base64 {
    fn sign(&self, data: &mut dyn Read) -> anyhow::Result<String> {
        let mut sd = Vec::new();
        data.read_to_end(&mut sd)?;
        let encoded = match self {
            Base64::Standard => BASE64_STANDARD.encode(&sd),
            Base64::URLEncoded => BASE64_URL_SAFE_NO_PAD.encode(&sd),
        };
        Ok(encoded)
    }
}
impl VerifyCrypto for Base64 {
    fn verify(&self, sig: &[u8], data: &mut dyn Read) -> anyhow::Result<bool> {
        let mut sd = Vec::new();
        data.read_to_end(&mut sd)?;
        let decoded = match self {
            Base64::Standard => BASE64_STANDARD.decode(sig)?,
            Base64::URLEncoded => BASE64_URL_SAFE_NO_PAD.decode(sig)?,
        };
        Ok(decoded == sd)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_b64_standard() {
        let b64 = Base64::Standard;
        let mut r = "hello world".as_bytes();
        let sig = b64.sign(&mut r).unwrap();
        let mut r2 = "hello world".as_bytes();
        assert!(b64.verify(sig.as_bytes(), &mut r2).unwrap());

        let b64 = Base64::URLEncoded;
        let mut r = "hello world".as_bytes();
        let sig = b64.sign(&mut r).unwrap();
        let mut r2 = "hello world".as_bytes();
        assert!(b64.verify(sig.as_bytes(), &mut r2).unwrap());
    }
}
