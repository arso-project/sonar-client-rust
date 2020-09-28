use ::bytes::{Buf, BufMut};
use hex::{FromHex, ToHex};
use serde_json::value::RawValue;

use prost::{
    encoding::{bool, bytes, skip_field, DecodeContext, WireType},
    DecodeError, Message,
};

use serde::de::{Deserialize, Deserializer, IntoDeserializer, Visitor};
use serde::ser::{Serialize, SerializeStruct, Serializer};

pub mod schema {
    include!(concat!(env!("OUT_DIR"), "/sonar.rs"));
}

/// Serializes `buffer` to a lowercase hex string.
pub fn as_hex<S>(buffer: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(buffer) = buffer {
        serializer.serialize_str(&hex::encode(&buffer))
    } else {
        serializer.serialize_unit()
    }
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
fn u32_from_integer<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: Deserializer<'de>,
{
    // use serde::de::Error;
    // TODO: Properly support option.
    let n: serde_json::Number = Deserialize::deserialize(deserializer)?;
    let n = n.as_u64();
    let n = n.and_then(|n| Some(n as u32));
    Ok(n)
    // if s < 0 {
    //     Err(Error::custom("Number has to be unsigne"))
    // } else {
    //     Ok(Some(s as u32))
    // }
}

/// Deserializes a lowercase hex string to a `Vec<u8>`.
pub fn from_hex<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    // TODO: Support empty strings.
    String::deserialize(deserializer).and_then(|string| {
        let buf = Vec::from_hex(&string).map_err(|err| Error::custom(err.to_string()))?;
        Ok(Some(buf))
    })
}

// fn err(msg: impl ToString) {
//     serde::de::Error::custom(msg.to_string())
// }
// fn map_err(err: impl std::error::Error) -> serde::de::Error {
//     serde::de::Error::custom(err.to_string())
// }

impl<'de> Deserialize<'de> for schema::Link {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        let mut parts = string.split("@");
        // if parts.len() != 2 {
        //     return Err(serde::de::Error::custom("Invalid link"));
        // }
        let key = parts
            .next()
            .ok_or(serde::de::Error::custom("Missing key in link"))?;
        let seq = parts
            .next()
            .ok_or(serde::de::Error::custom("Missing seq in link"))?;
        let key = hex::decode(key).map_err(serde::de::Error::custom)?;
        let seq = seq.parse::<u64>().map_err(serde::de::Error::custom)?;
        Ok(Self { key, seq })
        // let raw_value: Box<RawValue> = Deserialize::deserialize(deserializer)?;
        // // let raw_value = Box<RawValue>::deserialize(deserializer)?;
        // // let raw_value = RawValue::deserialize(deserializer)?;
        // let this = Json {
        //     raw_value,
        //     // raw_value: Box::new(raw_value),
        //     ..Default::default()
        // };
        // Ok(this)
    }
}

#[derive(Debug, Clone)]
pub struct Json {
    buf: Vec<u8>,
    raw_value: Box<RawValue>,
}

impl Json {
    pub fn get(&self) -> &str {
        self.raw_value.get()
    }
}

impl std::cmp::PartialEq<Json> for Json {
    fn eq(&self, other: &Json) -> bool {
        self.buf == other.buf
    }
}

impl Default for Json {
    fn default() -> Self {
        Self {
            buf: Vec::new(),
            raw_value: RawValue::from_string("null".into()).unwrap(),
        }
    }
}

impl Serialize for Json {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw_value.serialize(serializer)
    }
}

// fn deserialize<'de, D,T>(deserializer: D) -> Result<T,D::Error>
// where D: Deserializer<'de>, T: Deserialize) {
//     T::deserialize(deserializer)
// }

impl<'de> Deserialize<'de> for Json {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_value: Box<RawValue> = Deserialize::deserialize(deserializer)?;
        // let raw_value = Box<RawValue>::deserialize(deserializer)?;
        // let raw_value = RawValue::deserialize(deserializer)?;
        let this = Json {
            raw_value,
            // raw_value: Box::new(raw_value),
            ..Default::default()
        };
        Ok(this)
    }
}

// impl<'de: 'a, 'a> Deserialize<'de> for &'a Json {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let raw_value = RawValue::deserialize(deserializer)?;
//         let this = Json {
//             raw_value: Box::new(raw_value),
//             ..Default::default()
//         };
//         Ok(&this)
//     }
// }

// impl Default for Json {
//     fn default() -> Self {
//         Self {
//             buf: Vec::new(),
//             raw_value: Box::new(Raw
//         }
//     }
// }

impl Message for Json {
    fn encode_raw<B>(&self, buf: &mut B)
    where
        B: BufMut,
    {
        if !self.buf.is_empty() {
            bytes::encode(1, &self.buf, buf)
        }
    }
    fn merge_field<B>(
        &mut self,
        tag: u32,
        wire_type: WireType,
        buf: &mut B,
        ctx: DecodeContext,
    ) -> Result<(), DecodeError>
    where
        B: Buf,
    {
        if tag == 1 {
            bytes::merge(wire_type, &mut self.buf, buf, ctx)
        } else {
            skip_field(wire_type, tag, buf, ctx)
        }
    }
    fn encoded_len(&self) -> usize {
        if !self.buf.is_empty() {
            bytes::encoded_len(1, &self.buf)
        } else {
            0
        }
    }
    fn clear(&mut self) {
        self.buf.clear();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
