use anyhow::{Result, anyhow};
use orion::{
    aead::{SecretKey, open, seal},
    hazardous::kdf::pbkdf2,
    util::secure_rand_bytes,
};

pub struct Salt(pub [u8; 16]);
pub struct Vault {
    secret: SecretKey,
    pub salt: [u8; 16],
}

impl Salt {
    pub fn new() -> Result<Self> {
        let mut salt = [0u8; 16];
        secure_rand_bytes(&mut salt)?;
        Ok(Self(salt))
    }
    pub fn from(vec: Vec<u8>) -> Result<Self> {
        if vec.len() != 16 {
            Err(anyhow!("Vec for Salt was too short"))
        } else {
            Ok(Salt(vec.iter().enumerate().fold(
                [0u8; 16],
                |mut acc, (i, b)| {
                    acc[i] = *b;
                    acc
                },
            )))
        }
    }
}

impl Vault {
    pub fn new(salt: Salt, password: String) -> Result<Self> {
        let secret = pbkdf2::sha512::Password::from_slice(password.as_bytes())
            .and_then(|password| {
                let mut buf = [0u8; 32];
                pbkdf2::sha512::derive_key(&password, &salt.0, 65536, &mut buf)?;
                Ok(buf)
            })
            .and_then(|derived_key| SecretKey::from_slice(&derived_key))?;
        Ok(Vault {
            secret,
            salt: salt.0,
        })
    }
    pub fn decrypt(&self, bytes: Vec<u8>) -> Result<String> {
        let decrypted_bytes = open(&self.secret, &bytes)?;
        Ok(String::from_utf8(decrypted_bytes)?)
    }
    pub fn encrypt(&self, string: String) -> Result<Vec<u8>> {
        Ok(seal(&self.secret, string.as_bytes())?)
    }
}
