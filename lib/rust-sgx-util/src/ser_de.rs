#![cfg(feature = "with_serde")]

use serde::{Deserialize, self, Deserializer, Serializer};

pub(crate) fn serialize<S>(blob: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&base64::encode(blob))
}

pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{

    let string = String::deserialize(deserializer)?;
    base64::decode(&string).map_err(serde::de::Error::custom)
}
