use k256::{
    elliptic_curve::rand_core,
    schnorr::{signature::hazmat::PrehashSigner, SigningKey},
};
use rand_core::OsRng;

use super::{Error, Id, Signature};

pub struct PrivateKey(SigningKey);

impl PrivateKey {
    #[allow(dead_code)]
    pub fn gen() -> PrivateKey {
        let signing_key = SigningKey::random(&mut OsRng);
        PrivateKey(signing_key)
    }

    pub fn sign_id(&self, id: Id) -> Result<Signature, Error> {
        let sig = self.0.sign_prehash(&id.0)?;
        Ok(Signature(sig))
    }
    #[allow(dead_code)]
    pub fn get_public_key_string(&self) -> String {
        let pubkey = self.0.verifying_key();
        hex::encode(pubkey.to_bytes())
    }

    // 警告：这将导致你的密码从内存中被暴露出来！！
    #[allow(dead_code)]
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0.to_bytes())
    }
    #[allow(dead_code)]
    pub fn try_from_hex_string(v: &str) -> Result<PrivateKey, Error> {
        let vec = hex::decode(v)?;
        Ok(PrivateKey(SigningKey::from_bytes(&vec)?))
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    pub fn test_gen_key() {
        let key = PrivateKey::gen();
        println!("{}", key.as_hex_string());
        println!("{}", key.get_public_key_string());
    }
}
