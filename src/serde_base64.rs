use serde::{Serializer, de, Deserialize, Deserializer};
use base64lib;
use super::{IntoBinary, FromBinary};

pub fn serialize<S, T>(bytes: &T, serializer: S) -> Result<S::Ok, S::Error>
where S: Serializer, T: IntoBinary
{
    let b = base64lib::encode(bytes.into_binary());
    serializer.serialize_str(&b)
}

pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where D: Deserializer<'de>, T: FromBinary + Sized
{
    let s = <&str>::deserialize(deserializer)?;
    base64lib::decode(s).map_err(de::Error::custom)
        .and_then(|b| T::from_binary(b.as_ref())
                  .ok_or_else(|| de::Error::custom("Invalid value size")))
}
