use http::{Request, Response, HttpTryFrom};
use http::header::{HeaderValue};
use super::{ReadHeader, ContentType};

pub trait UnwrapType: ReadHeader {
    fn unwrap_type(&self, subtype: &str) -> Option<HeaderValue> {
        if let Some(mimetype) = self.get_header_str("Content-Type") {
            let mut ct = ContentType::new(mimetype);
            if ct.last_subtype() == Some(subtype) {
                ct.pop_subtype();
                if let Ok(mimetype) = HeaderValue::try_from(ct.as_ref()) {
                    return Some(mimetype);
                }
            }
        }
        None
    }
}

pub trait WrapType: ReadHeader {
    fn wrap_type(&self, subtype: &str) -> Option<HeaderValue> {
        if let Some(mimetype) = self.get_header_str("Content-Type") {
            let mut ct = ContentType::new(mimetype);
            ct.push_subtype(subtype);
            if let Ok(mimetype) = HeaderValue::try_from(ct.as_ref()) {
                return Some(mimetype);
            }
        }
        None
    }
}

impl<T> UnwrapType for Request<T> {}
impl<T> UnwrapType for Response<T> {}
impl<T> WrapType for Request<T> {}
impl<T> WrapType for Response<T> {}
