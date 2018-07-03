use http::{Request, Response};
use http::header::{HeaderValue};
use bytes::{Bytes};
use sodiumoxide::crypto::sealedbox;
use super::{ReadHeader, WriteHeader, CodecError, CodecResult, UnwrapType, WrapType, PublicKey, SecretKey};

pub type SealedboxResult<T> = Result<T, ()>;

pub trait DecryptSealedbox: Sized {
    fn decrypt_sealedbox_native(self, public_key: &PublicKey, secret_key: &SecretKey) -> SealedboxResult<Self>;
    
    #[inline]
    fn decrypt_sealedbox(self, public_key: &PublicKey, secret_key: &SecretKey) -> CodecResult<Self> {
        self.decrypt_sealedbox_native(public_key, secret_key).map_err(|_| CodecError::InvalidData)
    }
}

pub trait DecryptSealedboxBody: DecryptSealedbox + ReadHeader + WriteHeader + UnwrapType
{
    #[inline]
    fn decrypt_sealedbox_with_type<V>(self, public_key: &PublicKey, secret_key: &SecretKey, mimetype: V) -> CodecResult<Self>
        where HeaderValue: PartialEq<V>
    {
        if self.is_header("Content-Type", mimetype) {
            self.decrypt_sealedbox(public_key, secret_key)
        } else {
            Err(CodecError::InvalidType)
        }
    }

    #[inline]
    fn decrypt_sealedbox_auto_type(self, public_key: &PublicKey, secret_key: &SecretKey) -> CodecResult<Self> {
        if let Some(mimetype) = self.unwrap_type("sealedbox") {
            self.decrypt_sealedbox(public_key, secret_key)
                .map(move |mut new_self| {
                    new_self.set_header("Content-Type", mimetype);
                    new_self
                })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl DecryptSealedbox for Bytes
{
    fn decrypt_sealedbox_native(self, public_key: &PublicKey, secret_key: &SecretKey) -> SealedboxResult<Bytes> {
        Ok(sealedbox::open(&self, public_key, secret_key)?.into())
    }
}

impl DecryptSealedbox for Request<Bytes>
{
    fn decrypt_sealedbox_native(self, public_key: &PublicKey, secret_key: &SecretKey) -> SealedboxResult<Self> {
        let (parts, body) = self.into_parts();
        let body = body.decrypt_sealedbox_native(public_key, secret_key)?;
        Ok(Request::from_parts(parts, body))
    }
}

impl DecryptSealedboxBody for Request<Bytes> {}

impl DecryptSealedbox for Response<Bytes>
{
    fn decrypt_sealedbox_native(self, public_key: &PublicKey, secret_key: &SecretKey) -> SealedboxResult<Self> {
        let (parts, body) = self.into_parts();
        let body = body.decrypt_sealedbox_native(public_key, secret_key)?;
        Ok(Response::from_parts(parts, body))
    }
}

impl DecryptSealedboxBody for Response<Bytes> {}

pub trait EncryptSealedbox: Sized {
    fn encrypt_sealedbox_native(self, public_key: &PublicKey) -> SealedboxResult<Self>;
    
    #[inline]
    fn encrypt_sealedbox(self, public_key: &PublicKey) -> CodecResult<Self> {
        self.encrypt_sealedbox_native(public_key).map_err(|_| CodecError::InvalidData)
    }
}

pub trait EncryptSealedboxBody: EncryptSealedbox + ReadHeader + WriteHeader + WrapType
{
    #[inline]
    fn encrypt_sealedbox_with_type(self, public_key: &PublicKey, mimetype: &'static str) -> CodecResult<Self>
    {
        self.encrypt_sealedbox(public_key)
            .map(|mut new_self| {
                new_self.set_header("Content-Type", HeaderValue::from_static(mimetype));
                new_self
            })
    }

    #[inline]
    fn encrypt_sealedbox_auto_type(self, public_key: &PublicKey) -> CodecResult<Self> {
        if let Some(mimetype) = self.wrap_type("sealedbox") {
            self.encrypt_sealedbox(public_key)
                .map(move |mut new_self| {
                    new_self.set_header("Content-Type", mimetype);
                    new_self
                })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl EncryptSealedbox for Bytes
{
    fn encrypt_sealedbox_native(self, public_key: &PublicKey) -> SealedboxResult<Self> {
        Ok(sealedbox::seal(&self, public_key).into())
    }
}

impl EncryptSealedbox for Request<Bytes>
{
    fn encrypt_sealedbox_native(self, public_key: &PublicKey) -> SealedboxResult<Self> {
        let (parts, body) = self.into_parts();
        let body = body.encrypt_sealedbox_native(public_key)?;
        Ok(Request::from_parts(parts, body))
    }
}

impl EncryptSealedboxBody for Request<Bytes> {}

impl EncryptSealedbox for Response<Bytes>
{
    fn encrypt_sealedbox_native(self, public_key: &PublicKey) -> SealedboxResult<Self> {
        let (parts, body) = self.into_parts();
        let body = body.encrypt_sealedbox_native(public_key)?;
        Ok(Response::from_parts(parts, body))
    }
}

impl EncryptSealedboxBody for Response<Bytes> {}

#[cfg(test)]
mod tests {
    use http::{Request};
    use super::super::gen_keypair;
    use super::*;

    #[test]
    fn test_sealedbox() {
        let (pk, sk) = gen_keypair();
        
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+plain")
            .body("hello world".into())
            .unwrap();
        
        let e = a.encrypt_sealedbox_auto_type(&pk).unwrap();

        assert!(e.is_header("Content-Type", "application/vnd.literium.v1+plain+sealedbox"));
        
        let d = e.decrypt_sealedbox_auto_type(&pk, &sk).unwrap();

        assert!(d.is_header("Content-Type", "application/vnd.literium.v1+plain"));
        
        assert_eq!(d.into_body(), "hello world");
    }
}
