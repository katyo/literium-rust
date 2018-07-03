use futures::{Future, Stream};
use http::{Request, Response};
use http::request::{Parts as RequestParts};
use http::response::{Parts as ResponseParts};
use bytes::{Bytes};
use hyper::{Error, Body};

pub trait UnwrapBody<H, B>
{
    fn unwrap_body(self) -> (H, B);
}

pub trait WrapBody<H, B>
{
    fn wrap_body(h: H, b: B) -> Self;
}

impl<T> UnwrapBody<RequestParts, T> for Request<T> {
    fn unwrap_body(self) -> (RequestParts, T) {
        self.into_parts()
    }
}

impl<T> WrapBody<RequestParts, T> for Request<T> {
    fn wrap_body(h: RequestParts, b: T) -> Self {
        Self::from_parts(h, b)
    }
}

impl<T> UnwrapBody<ResponseParts, T> for Response<T> {
    fn unwrap_body(self) -> (ResponseParts, T) {
        self.into_parts()
    }
}

impl<T> WrapBody<ResponseParts, T> for Response<T> {
    fn wrap_body(h: ResponseParts, b: T) -> Self {
        Self::from_parts(h, b)
    }
}

pub type BodyFuture<T> = Box<Future<Item = T, Error = Error>>;

pub trait ConcatBody<T, H>: UnwrapBody<H, Body> + Sized
    where T: WrapBody<H, Bytes>,
          H: 'static
{
    fn concat_body_limited(self, _maxlen: Option<usize>) -> BodyFuture<T>
    {
        let (parts, body) = self.unwrap_body();
        Box::new(body.concat2().map(move |body| {
            T::wrap_body(parts, body.into_bytes())
        }))
    }
    
    fn concat_body(self) -> BodyFuture<T> {
        self.concat_body_limited(None)
    }
    
    fn concat_body_with_limit(self, maxlen: usize) -> BodyFuture<T> {
        self.concat_body_limited(Some(maxlen))
    }
}

impl ConcatBody<Request<Bytes>, RequestParts> for Request<Body> {}
impl ConcatBody<Response<Bytes>, ResponseParts> for Response<Body> {}

pub trait RollupBody<T, H>: UnwrapBody<H, Bytes> + Sized
    where T: WrapBody<H, Body>
{
    fn rollup_body(self) -> T {
        let (parts, body) = self.unwrap_body();
        T::wrap_body(parts, body.into())
    }
}

impl RollupBody<Request<Body>, RequestParts> for Request<Bytes> {}
impl RollupBody<Response<Body>, ResponseParts> for Response<Bytes> {}

#[cfg(test)]
mod tests {
    use std::io::{Error};
    use futures::stream::{iter_ok};
    use http::{Request, Response};
    use super::*;
    
    #[test]
    fn test_concat_body() {
        let cs = vec!["hello", " ", "world"];
        let s = iter_ok::<_, Error>(cs);
        let b = Body::wrap_stream(s);
        
        let a = Request::builder()
            .body(b)
            .unwrap();
        let d = a.concat_body().wait().unwrap();
        
        assert_eq!(d.into_body(), "hello world");
    }

    #[test]
    fn test_rollup_body() {
        let a: Response<Bytes> = Response::builder()
            .body("hello world".into())
            .unwrap();
        let d = a.rollup_body();
        let b = d.into_body().concat2().wait().unwrap();
        
        assert_eq!(b.into_bytes(), "hello world");
    }
}
