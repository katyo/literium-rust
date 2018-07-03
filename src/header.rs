use http::{Request, Response};
use http::header::{HeaderValue, HeaderMap, AsHeaderName, IntoHeaderName};
use bytes::{Bytes};

pub trait ReadHeader {
    fn get_headers(&self) -> &HeaderMap;
    
    fn is_header<K, V>(&self, key: K, value: V) -> bool
        where K: AsHeaderName,
              HeaderValue: PartialEq<V>
    {
        self.get_headers()
            .get(key)
            .map(|val| *val == value)
            .unwrap_or(false)
    }

    fn get_header<K>(&self, key: K) -> Option<&HeaderValue>
        where K: AsHeaderName
    {
        self.get_headers()
            .get(key)
    }

    fn get_header_str<K>(&self, key: K) -> Option<&str>
        where K: AsHeaderName
    {
        self.get_headers()
            .get(key)
            .and_then(|val| val.to_str().ok())
    }

    fn get_header_bin<K>(&self, key: K) -> Option<Bytes>
        where K: AsHeaderName
    {
        self.get_headers()
            .get(key)
            .map(|val| val.as_bytes().into())
    }
}

impl<T> ReadHeader for Request<T> {
    fn get_headers(&self) -> &HeaderMap {
        self.headers()
    }
}

impl<T> ReadHeader for Response<T> {
    fn get_headers(&self) -> &HeaderMap {
        self.headers()
    }
}

pub trait WriteHeader {
    fn set_header<K, V>(&mut self, key: K, value: V)
        where K: IntoHeaderName,
              HeaderValue: From<V>;
}

impl<T> WriteHeader for Request<T> {
    fn set_header<K, V>(&mut self, key: K, value: V)
        where K: IntoHeaderName,
              HeaderValue: From<V>
    {
        self.headers_mut().insert(key, value.into());
    }
}

impl<T> WriteHeader for Response<T> {
    fn set_header<K, V>(&mut self, key: K, value: V)
        where K: IntoHeaderName,
              HeaderValue: From<V>
    {
        self.headers_mut().insert(key, value.into());
    }
}

/*
pub trait WithHeader {
    fn with_header<K, V>(&mut self, key: K, value: V) -> &mut Self
        where HeaderName: HttpTryFrom<K>,
              HeaderValue: HttpTryFrom<V>;
}

impl WithHeader for RequestBuilder {
    fn with_header<K, V>(&mut self, key: K, value: V) -> &mut Self
        where HeaderName: HttpTryFrom<K>,
              HeaderValue: HttpTryFrom<V>
    {
        self.header(key, value)
    }
}

impl WithHeader for ResponseBuilder {
    fn with_header<K, V>(&mut self, key: K, value: V) -> &mut Self
        where HeaderName: HttpTryFrom<K>,
              HeaderValue: HttpTryFrom<V>
    {
        self.header(key, value)
    }
}
*/

#[cfg(test)]
mod tests {
    use http::{Request};
    use super::*;
    
    #[test]
    fn test_is_header() {
        assert_eq!(Request::builder()
                   .body(())
                   .unwrap()
                   .is_header("Accept", "application/json"),
                   false);

        assert_eq!(Request::builder()
                   .header("Accept", "text/html")
                   .body(())
                   .unwrap()
                   .is_header("Accept", "application/json"),
                   false);

        assert_eq!(Request::builder()
                   .header("Accept", "application/json")
                   .body(())
                   .unwrap()
                   .is_header("Accept", "application/json"),
                   true);
    }

    #[test]
    fn test_get_header_str() {
        assert_eq!(Request::builder()
                   .body(())
                   .unwrap()
                   .get_header_str("Accept"),
                   None);

        assert_eq!(Request::builder()
                   .header("Accept", "text/html")
                   .body(())
                   .unwrap()
                   .get_header_str("Accept"),
                   Some("text/html"));
    }
}
