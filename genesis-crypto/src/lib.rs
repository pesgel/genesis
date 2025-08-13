#![allow(dead_code)]

use std::io::Read;

mod argon;
mod base64;
mod blake3;
mod ed25519;

pub trait Crypto: SingerCrypto + VerifyCrypto {}

pub trait SingerCrypto {
    fn sign(&self, data: &mut dyn Read) -> anyhow::Result<String>;
}

pub trait VerifyCrypto {
    fn verify(&self, sig: &[u8], data: &mut dyn Read) -> anyhow::Result<bool>;
}
