use std::net::{IpAddr};
use std::str::{FromStr};
use http::{Request};
use super::{ReadHeader};

pub trait ReadClientInfo {
    fn get_x_real_ip(&self) -> Option<IpAddr>;
    fn get_x_forwarded_for(&self) -> Option<Vec<IpAddr>>;
}

impl<T> ReadClientInfo for Request<T> {
    fn get_x_real_ip(&self) -> Option<IpAddr>
    {
        self.get_header_str("X-Real-IP")
            .and_then(|val| IpAddr::from_str(val).ok())
    }

    fn get_x_forwarded_for(&self) -> Option<Vec<IpAddr>>
    {
        self.get_header_str("X-Forwarded-For")
            .and_then(|val| val.split(',')
                      .map(|s| IpAddr::from_str(s.trim()))
                      .collect::<Result<Vec<_>, _>>().ok())
    }
}

#[cfg(test)]
mod tests {
    use http::{Request};
    use super::*;
    
    #[test]
    fn test_get_x_real_ip() {
        assert_eq!(Request::builder()
                   .body(())
                   .unwrap()
                   .get_x_real_ip(),
                   None);
        
        assert_eq!(Request::builder()
                   .header("x-real-ip", "192.168.101.21")
                   .body(())
                   .unwrap()
                   .get_x_real_ip(),
                   Some("192.168.101.21".parse().unwrap()));
    }

    #[test]
    fn test_get_x_forwarded_for() {
        assert_eq!(Request::builder()
                   .body(())
                   .unwrap()
                   .get_x_forwarded_for(),
                   None);
        
        assert_eq!(Request::builder()
                   .header("x-forwarded-for", "192.168.101.21")
                   .body(())
                   .unwrap()
                   .get_x_forwarded_for(),
                   Some(vec!["192.168.101.21".parse().unwrap()]));

        assert_eq!(Request::builder()
                   .header("x-forwarded-for", "192.168.101.21, 10.0.0.13")
                   .body(())
                   .unwrap()
                   .get_x_forwarded_for(),
                   Some(vec!["192.168.101.21".parse().unwrap(),
                             "10.0.0.13".parse().unwrap()]));
    }
}
