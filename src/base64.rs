use http::{Request, Response};
use http::header::{HeaderValue};
use bytes::{Bytes};
use base64lib;
use super::{ReadHeader, WriteHeader, CodecError, CodecResult, UnwrapType, WrapType};

pub type Base64Result<T> = Result<T, base64lib::DecodeError>;

pub trait DecodeBase64: Sized {
    fn decode_base64_native(self) -> Base64Result<Self>;
    
    #[inline]
    fn decode_base64(self) -> CodecResult<Self> {
        self.decode_base64_native().map_err(|_| CodecError::InvalidData)
    }   
}

pub trait DecodeBase64Body: DecodeBase64 + ReadHeader + WriteHeader + UnwrapType
{
    #[inline]
    fn decode_base64_with_type<V>(self, mimetype: V) -> CodecResult<Self>
        where HeaderValue: PartialEq<V>
    {
        if self.is_header("Content-Type", mimetype) {
            self.decode_base64()
        } else {
            Err(CodecError::InvalidType)
        }
    }

    #[inline]
    fn decode_base64_auto_type(self) -> CodecResult<Self> {
        if let Some(mimetype) = self.unwrap_type("base64") {
            self.decode_base64().map(move |mut new_self| {
                new_self.set_header("Content-Type", mimetype);
                new_self
            })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl DecodeBase64 for Bytes {
    fn decode_base64_native(self) -> Base64Result<Bytes> {
        Ok(base64lib::decode(&self)?.into())
    }
}

impl DecodeBase64 for Request<Bytes>
{
    fn decode_base64_native(self) -> Base64Result<Request<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.decode_base64_native()?;
        Ok(Request::from_parts(parts, body))
    }
}

impl DecodeBase64Body for Request<Bytes> {}

impl DecodeBase64 for Response<Bytes>
{
    fn decode_base64_native(self) -> Base64Result<Response<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.decode_base64_native()?;
        Ok(Response::from_parts(parts, Bytes::from(body)))
    }
}

impl DecodeBase64Body for Response<Bytes> {}

pub trait EncodeBase64: Sized {
    fn encode_base64_native(self) -> Base64Result<Self>;

    #[inline]
    fn encode_base64(self) -> CodecResult<Self> {
        self.encode_base64_native().map_err(|_| CodecError::InvalidData)
    }
}

pub trait EncodeBase64Body: EncodeBase64 + ReadHeader + WriteHeader + WrapType
{
    #[inline]
    fn encode_base64_with_type(self, mimetype: &'static str) -> CodecResult<Self>
    {
        self.encode_base64()
            .map(|mut new_self| {
                new_self.set_header("Content-Type", HeaderValue::from_static(mimetype));
                new_self
            })
    }

    #[inline]
    fn encode_base64_auto_type(self) -> CodecResult<Self> {
        if let Some(mimetype) = self.wrap_type("base64") {
            self.encode_base64()
                .map(move |mut new_self| {
                    new_self.set_header("Content-Type", mimetype);
                    new_self
                })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl EncodeBase64 for Bytes {
    fn encode_base64_native(self) -> Base64Result<Bytes> {
        Ok(base64lib::encode(&self).into())
    }
}

impl EncodeBase64 for Request<Bytes>
{
    fn encode_base64_native(self) -> Base64Result<Request<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.encode_base64_native()?;
        Ok(Request::from_parts(parts, body))
    }
}

impl EncodeBase64Body for Request<Bytes> {}

impl EncodeBase64 for Response<Bytes>
{
    fn encode_base64_native(self) -> Base64Result<Response<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.encode_base64_native()?;
        Ok(Response::from_parts(parts, body))
    }
}

impl EncodeBase64Body for Response<Bytes> {}

#[cfg(test)]
mod tests {
    use http::{Request, Response};
    use super::*;

    #[test]
    fn test_decode_base64_ok() {
        let a: Request<Bytes> = Request::builder()
            .body("aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64().unwrap();
        
        assert_eq!(d.into_body(), "hello world");
    }

    #[test]
    fn test_decode_base64_err_data() {
        let a: Request<Bytes> = Request::builder()
            .body("+aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64();
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_base64_with_type_ok() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/base64")
            .body("aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_with_type("application/base64").unwrap();
        
        assert_eq!(d.into_body(), "hello world");
    }

    #[test]
    fn test_decode_base64_with_type_err_data() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/base64")
            .body("+aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_with_type("application/base64");
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_base64_with_type_err_type() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "text/html")
            .body("aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_with_type("application/base64");
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidType);
    }

    #[test]
    fn test_decode_base64_auto_type_ok() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+base64")
            .body("aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_auto_type().unwrap();
        
        assert!(d.is_header("Content-Type", "application/vnd.literium.v1"));
    }

    #[test]
    fn test_decode_base64_auto_type_err_data() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+base64")
            .body("+aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_auto_type();

        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_base64_auto_type_err_type() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+json")
            .body("aGVsbG8gd29ybGQ=".into())
            .unwrap();
        let d = a.decode_base64_auto_type();

        assert_eq!(d.unwrap_err(), CodecError::InvalidType);
    }

    #[test]
    fn test_encode_base64() {
        let a: Response<Bytes> = Response::builder()
            .body("hello world".into())
            .unwrap();
        let d = a.encode_base64().unwrap();
        
        assert_eq!(d.into_body(), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_encode_base64_with_type() {
        let a: Response<Bytes> = Response::builder()
            .body("hello world".into())
            .unwrap();
        let d = a.encode_base64_with_type("application/base64").unwrap();

        assert!(d.is_header("Content-Type", "application/base64"));
        assert_eq!(d.into_body(), "aGVsbG8gd29ybGQ=");
    }

    #[test]
    fn test_encode_base64_auto_type() {
        let a: Response<Bytes> = Response::builder()
            .header("Content-Type", "application/vnd.literium.v1")
            .body("hello world".into())
            .unwrap();
        let d = a.encode_base64_auto_type().unwrap();

        assert!(d.is_header("Content-Type", "application/vnd.literium.v1+base64"));
        assert_eq!(d.into_body(), "aGVsbG8gd29ybGQ=");
    }
}
