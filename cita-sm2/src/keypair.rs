// Copyright Rivtower Technologies LLC.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Address, Error, PrivKey, PubKey};
use crate::types::H160;
use cita_crypto_trait::CreateKey;
use hashable::Hashable;
use rustc_serialize::hex::ToHex;
use std::fmt;
use ring::signature::{ECDSA_SM2P256_SM3_ASN1_SIGNING, EcdsaKeyPair};

pub fn pubkey_to_address(pubkey: &PubKey) -> Address {
    H160::from(pubkey.crypt_hash())
}

pub struct KeyPair {
    pub inner: EcdsaKeyPair,
    privkey: PrivKey,
    pubkey: PubKey,
}

impl fmt::Display for KeyPair {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        writeln!(f, "privkey:  {}", self.privkey().0.to_hex())?;
        writeln!(f, "pubkey:  {}", self.pubkey().0.to_hex())?;
        write!(f, "address:  {}", self.address().0.to_hex())
    }
}

impl CreateKey for KeyPair {
    type PrivKey = PrivKey;
    type PubKey = PubKey;
    type Error = Error;

    fn from_privkey(privkey: Self::PrivKey) -> Result<Self, Self::Error> {
        let inner = EcdsaKeyPair::from_privatekey_bytes(
            &ECDSA_SM2P256_SM3_ASN1_SIGNING,
            untrusted::Input::from(privkey.as_ref())
        ).map_err(|_| Error::RecoverError)?;
        let out = &inner.private_key()[16..];
        let privkey = PrivKey::from(out);
        let pubkey = PubKey::from(&inner.public_key()[1..]);
        Ok(KeyPair { inner, privkey, pubkey })
    }

    fn gen_keypair() -> Self {
        let inner = EcdsaKeyPair::generate_keypair(&ECDSA_SM2P256_SM3_ASN1_SIGNING).unwrap();
        let out = &inner.private_key()[16..];
        let privkey = PrivKey::from(out);
        let pubkey = PubKey::from(&inner.public_key()[1..]);
        KeyPair { inner, privkey, pubkey }
    }

    fn privkey(&self) -> &Self::PrivKey {
        &self.privkey
    }

    fn pubkey(&self) -> &Self::PubKey {
        &self.pubkey
    }

    fn address(&self) -> Address {
        pubkey_to_address(self.pubkey())
    }
}

impl Default for KeyPair {
    fn default() -> Self {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::KeyPair;
    use cita_crypto_trait::CreateKey;

    #[test]
    fn test_gen_keypair() {
        let keypair = KeyPair::gen_keypair();
        let privkey = keypair.privkey().clone();
        let new_keypair = KeyPair::from_privkey(privkey).unwrap();
        assert_eq!(keypair.pubkey(), new_keypair.pubkey());
    }
}
