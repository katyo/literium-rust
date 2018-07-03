use bytes::{Bytes};
use super::{PublicKey, SecretKey, Key};

pub trait IntoBinary {
    fn into_binary(&self) -> &[u8];
}

pub trait FromBinary: Sized {
    fn from_binary(b: &[u8]) -> Option<Self>;
}

impl IntoBinary for Vec<u8> {
    fn into_binary(&self) -> &[u8] {
        &self[..]
    }
}

impl FromBinary for Vec<u8> {
    fn from_binary(b: &[u8]) -> Option<Self> {
        Some(b.into())
    }
}

impl IntoBinary for Bytes {
    fn into_binary(&self) -> &[u8] {
        &self[..]
    }
}

impl FromBinary for Bytes {
    fn from_binary(b: &[u8]) -> Option<Self> {
        Some(b.into())
    }
}

impl IntoBinary for PublicKey {
    fn into_binary(&self) -> &[u8] {
        &self.0[..]
    }
}

impl FromBinary for PublicKey {
    fn from_binary(b: &[u8]) -> Option<Self> {
        PublicKey::from_slice(b)
    }
}

impl IntoBinary for SecretKey {
    fn into_binary(&self) -> &[u8] {
        &self.0[..]
    }
}

impl FromBinary for SecretKey {
    fn from_binary(b: &[u8]) -> Option<Self> {
        SecretKey::from_slice(b)
    }
}

impl IntoBinary for Key {
    fn into_binary(&self) -> &[u8] {
        &self.0[..]
    }
}

impl FromBinary for Key {
    fn from_binary(b: &[u8]) -> Option<Self> {
        Key::from_slice(b.as_ref())
    }
}
