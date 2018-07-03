use http::{Request};
use serde::{de};
use serde_qs as qs;

pub trait ReadQuery {
    fn get_query_str(&self) -> Option<&str>;
    fn get_query<T>(&self) -> Result<Option<T>, qs::Error>
        where for<'de> T: de::Deserialize<'de>
    {
        if let Some(q) = self.get_query_str() {
            qs::from_str(q)
        } else {
            Ok(None)
        }
    }
}

impl<T> ReadQuery for Request<T> {
    fn get_query_str(&self) -> Option<&str> {
        self.uri().query()
    }
}
