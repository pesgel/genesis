use crate::{Crypto, SingerCrypto, VerifyCrypto};
use base64::Engine;
use base64::prelude::BASE64_URL_SAFE_NO_PAD;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use std::io::Read;
pub struct Ed25519;

impl Ed25519 {
    pub fn generate() -> anyhow::Result<(Vec<u8>, Vec<u8>)> {
        let mut csp_rng = OsRng;
        let sk = SigningKey::generate(&mut csp_rng);
        let pk: VerifyingKey = (&sk).into();
        Ok((sk.to_bytes().to_vec(), pk.to_bytes().to_vec()))
    }
}

impl Crypto for Ed25519Singer {}
pub struct Ed25519Singer {
    sk: SigningKey,
}

impl Ed25519Singer {
    pub fn try_new(sk: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let sk_bytes: [u8; 32] = (&sk.as_ref()[..32]).try_into()?;
        let sk = SigningKey::from_bytes(&sk_bytes);
        Ok(Self { sk })
    }
}

impl SingerCrypto for Ed25519Singer {
    fn sign(&self, data: &mut dyn Read) -> anyhow::Result<String> {
        let mut input = Vec::new();
        data.read_to_end(&mut input)?;
        let sig = self.sk.sign(&input);
        Ok(BASE64_URL_SAFE_NO_PAD.encode(sig.to_bytes()))
    }
}

impl VerifyCrypto for Ed25519Singer {
    fn verify(&self, sig: &[u8], reader: &mut dyn Read) -> anyhow::Result<bool> {
        // let verifying_key: VerifyingKey = self.sk.verifying_key();
        let sig_bytes = BASE64_URL_SAFE_NO_PAD.decode(sig)?;
        let sig: Signature = Signature::from_bytes(sig_bytes[..64].try_into()?);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(self.sk.verify(&buf, &sig).is_ok())
    }
}

pub struct Ed25519Verifier {
    vk: VerifyingKey,
}

impl Ed25519Verifier {
    pub fn try_new(vk: impl AsRef<[u8]>) -> anyhow::Result<Self> {
        let vk_bytes: [u8; 32] = (&vk.as_ref()[..32]).try_into()?;
        let vk = VerifyingKey::from_bytes(&vk_bytes)?;
        Ok(Self { vk })
    }
}

impl VerifyCrypto for Ed25519Verifier {
    fn verify(&self, sig: &[u8], reader: &mut dyn Read) -> anyhow::Result<bool> {
        let sig_bytes = BASE64_URL_SAFE_NO_PAD.decode(sig)?;
        let sig: Signature = Signature::from_bytes(sig_bytes[..64].try_into()?);
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf)?;
        Ok(self.vk.verify(&buf, &sig).is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ed25519_sign_and_verify() {
        let km = Ed25519::generate().unwrap();

        let signer = Ed25519Singer::try_new(km.0).unwrap();
        let mut reader = "hello world".as_bytes();
        let sig = signer.sign(&mut reader).unwrap();
        println!("sig: {}, len: {}", sig, sig.len());
        let mut reader = "hello world".as_bytes();
        let check_1 = signer.verify(sig.as_bytes(), &mut reader).unwrap();
        assert!(check_1);
        let verifier = Ed25519Verifier::try_new(km.1).unwrap();
        let mut reader = "hello world".as_bytes();
        let result = verifier.verify(sig.as_bytes(), &mut reader).unwrap();
        assert!(result);
    }
}
