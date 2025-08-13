use crate::{SingerCrypto, VerifyCrypto};
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use std::io::Read;

#[derive(Default)]
pub struct Argon;

impl SingerCrypto for Argon {
    fn sign(&self, data: &mut dyn Read) -> anyhow::Result<String> {
        let mut password = Vec::new();
        data.read_to_end(&mut password)?;
        let salt = SaltString::generate(&mut OsRng);
        // Argon2 with default params (Argon2id v19)
        let argon2 = Argon2::default();
        // Hash password to PHC string ($argon2id$v=19$...)
        let password_hash = argon2
            .hash_password(password.as_slice(), &salt)
            .map_err(|e| anyhow::anyhow!(e.to_string()))?
            .to_string();
        Ok(password_hash)
    }
}

impl VerifyCrypto for Argon {
    fn verify(&self, sig: &[u8], data: &mut dyn Read) -> anyhow::Result<bool> {
        let mut password = Vec::new();
        data.read_to_end(&mut password)?;
        let stored_hash = std::str::from_utf8(sig)?;
        let parsed_hash = PasswordHash::new(stored_hash)
            .map_err(|e| anyhow::anyhow!("Invalid hash format: {}", e))?;
        Ok(Argon2::default()
            .verify_password(&password, &parsed_hash)
            .is_ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_argon() {
        let argon: Argon = Default::default();
        let mut sign_str = "world".as_bytes();
        let encrypt_data = argon.sign(&mut sign_str).unwrap();
        let mut sign_str = "world".as_bytes();
        assert!(
            argon
                .verify(encrypt_data.as_bytes(), &mut sign_str)
                .unwrap()
        );
    }
}
