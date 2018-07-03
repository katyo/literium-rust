use http::{Request, Response};
use http::header::{HeaderValue};
use bytes::{Bytes};
use serde::{ser, de};
use serde_json as json;
use super::{ReadHeader, WriteHeader, CodecError, CodecResult, UnwrapType, WrapType};

pub trait DecodeJson<T>: Sized {
    fn decode_json_native(self) -> json::Result<T>;
    
    #[inline]
    fn decode_json(self) -> CodecResult<T> {
        self.decode_json_native().map_err(|_| CodecError::InvalidData)
    }
}

pub trait DecodeJsonBody<T>: DecodeJson<T> + ReadHeader + UnwrapType
    where T: WriteHeader
{
    #[inline]
    fn decode_json_with_type<V>(self, mimetype: V) -> CodecResult<T>
        where HeaderValue: PartialEq<V>
    {
        if self.is_header("Content-Type", mimetype) {
            self.decode_json()
        } else {
            Err(CodecError::InvalidType)
        }
    }

    #[inline]
    fn decode_json_auto_type(self) -> CodecResult<T> {
        if let Some(mimetype) = self.unwrap_type("json") {
            self.decode_json().map(move |mut new_self| {
                new_self.set_header("Content-Type", mimetype);
                new_self
            })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl<T> DecodeJson<T> for Bytes
    where for<'de> T: de::Deserialize<'de>
{
    fn decode_json_native(self) -> json::Result<T> {
        json::from_slice(&self)
    }
}

impl<T> DecodeJson<Request<T>> for Request<Bytes>
    where for<'de> T: de::Deserialize<'de>
{
    fn decode_json_native(self) -> json::Result<Request<T>> {
        let (parts, body) = self.into_parts();
        let body = body.decode_json_native()?;
        Ok(Request::from_parts(parts, body))
    }
}

impl<T> DecodeJsonBody<Request<T>> for Request<Bytes>
    where for<'de> T: de::Deserialize<'de> {}

impl<T> DecodeJson<Response<T>> for Response<Bytes>
    where for<'de> T: de::Deserialize<'de>
{
    fn decode_json_native(self) -> json::Result<Response<T>> {
        let (parts, body) = self.into_parts();
        let body = body.decode_json_native()?;
        Ok(Response::from_parts(parts, body))
    }
}

impl<T> DecodeJsonBody<Response<T>> for Response<Bytes>
    where for<'de> T: de::Deserialize<'de> {}

pub trait EncodeJson<T>: Sized {
    fn encode_json_native(self) -> json::Result<T>;

    #[inline]
    fn encode_json(self) -> CodecResult<T> {
        self.encode_json_native().map_err(|_| CodecError::InvalidData)
    }
}

pub trait EncodeJsonBody<T>: EncodeJson<T> + ReadHeader + WrapType
    where T: WriteHeader
{
    #[inline]
    fn encode_json_with_type(self, mimetype: &'static str) -> CodecResult<T>
    {
        self.encode_json()
            .map(|mut new_self| {
                new_self.set_header("Content-Type", HeaderValue::from_static(mimetype));
                new_self
            })
    }

    #[inline]
    fn encode_json_auto_type(self) -> CodecResult<T> {
        if let Some(mimetype) = self.wrap_type("json") {
            self.encode_json()
                .map(move |mut new_self| {
                    new_self.set_header("Content-Type", mimetype);
                    new_self
                })
        } else {
            Err(CodecError::InvalidType)
        }
    }
}

impl<T> EncodeJson<Bytes> for T
    where T: ser::Serialize
{
    fn encode_json_native(self) -> json::Result<Bytes> {
        Ok(json::to_vec(&self)?.into())
    }
}

impl<T> EncodeJson<Request<Bytes>> for Request<T>
    where T: ser::Serialize
{
    fn encode_json_native(self) -> json::Result<Request<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.encode_json_native()?;
        Ok(Request::from_parts(parts, body))
    }
}

impl<T> EncodeJsonBody<Request<Bytes>> for Request<T>
    where T: ser::Serialize {}

impl<T> EncodeJson<Response<Bytes>> for Response<T>
    where T: ser::Serialize
{
    fn encode_json_native(self) -> json::Result<Response<Bytes>> {
        let (parts, body) = self.into_parts();
        let body = body.encode_json_native()?;
        Ok(Response::from_parts(parts, body))
    }
}

impl<T> EncodeJsonBody<Response<Bytes>> for Response<T>
    where T: ser::Serialize {}

#[cfg(test)]
mod tests {
    use http::{Request, Response};
    use super::*;

    #[test]
    fn test_decode_json_ok() {
        let a: Request<Bytes> = Request::builder()
            .body("[13,1,0]".into())
            .unwrap();
        let d: Request<Vec<u8>> = a.decode_json().unwrap();
        
        assert_eq!(d.into_body(), vec![13u8, 1, 0]);
    }

    #[test]
    fn test_decode_json_err_data() {
        let a: Request<Bytes> = Request::builder()
            .body("{13,1,0}".into())
            .unwrap();
        let d: Result<Request<Vec<u8>>, _> = a.decode_json();
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_json_with_type_ok() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/json")
            .body("[13,1,0]".into())
            .unwrap();
        let d: Request<Vec<u8>> = a.decode_json_with_type("application/json").unwrap();
        
        assert_eq!(d.into_body(), vec![13u8, 1, 0]);
    }

    #[test]
    fn test_decode_json_with_type_err_data() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/json")
            .body("{13,1,0}".into())
            .unwrap();
        let d: Result<Request<Vec<u8>>, _> = a.decode_json_with_type("application/json");
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_json_with_type_err_type() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "text/html")
            .body("[13,1,0]".into())
            .unwrap();
        let d: Result<Request<Vec<u8>>, _> = a.decode_json_with_type("application/json");
        
        assert_eq!(d.unwrap_err(), CodecError::InvalidType);
    }

    #[test]
    fn test_decode_json_auto_type_ok() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+json")
            .body("[13,1,0]".into())
            .unwrap();
        let d: Request<Vec<u8>> = a.decode_json_auto_type().unwrap();
        
        assert!(d.is_header("Content-Type", "application/vnd.literium.v1"));
    }

    #[test]
    fn test_decode_json_auto_type_err_data() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+json")
            .body("[13;1;0]".into())
            .unwrap();
        let d: Result<Request<Vec<u8>>, _> = a.decode_json_auto_type();

        assert_eq!(d.unwrap_err(), CodecError::InvalidData);
    }

    #[test]
    fn test_decode_json_auto_type_err_type() {
        let a: Request<Bytes> = Request::builder()
            .header("Content-Type", "application/vnd.literium.v1+base64")
            .body("[13,1,0]".into())
            .unwrap();
        let d: Result<Request<Vec<u8>>, _> = a.decode_json_auto_type();

        assert_eq!(d.unwrap_err(), CodecError::InvalidType);
    }

    #[test]
    fn test_encode_json() {
        let a = Response::builder()
            .body(vec![13u8, 1, 0])
            .unwrap();
        let d: Response<Bytes> = a.encode_json().unwrap();
        
        assert_eq!(d.into_body(), "[13,1,0]");
    }

    #[test]
    fn test_encode_json_with_type() {
        let a = Response::builder()
            .body(vec![13u8, 1, 0])
            .unwrap();
        let d: Response<Bytes> = a.encode_json_with_type("application/json").unwrap();

        assert!(d.is_header("Content-Type", "application/json"));
        assert_eq!(d.into_body(), "[13,1,0]");
    }

    #[test]
    fn test_encode_json_auto_type() {
        let a = Response::builder()
            .header("Content-Type", "application/vnd.literium.v1")
            .body(vec![13u8, 1, 0])
            .unwrap();
        let d: Response<Bytes> = a.encode_json_auto_type().unwrap();

        assert!(d.is_header("Content-Type", "application/vnd.literium.v1+json"));
        assert_eq!(d.into_body(), "[13,1,0]");
    }
}
