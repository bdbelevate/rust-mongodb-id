use mongodb::bson::{oid::ObjectId, Bson};
use serde::{
    de, de::MapAccess, de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize,
    Serializer,
};
use std::fmt;

/// An ID as defined by the GraphQL specification
///
/// Represented as a string, but can be converted _to_ from an integer as well.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ID {
    ObjectId(ObjectId),
    String(String),
    Int64(i64),
}

impl Serialize for ID {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ID::ObjectId(o) => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("$oid", &o.to_string())?;
                map.end()
            }
            ID::String(s) => serializer.serialize_str(s),
            ID::Int64(i) => serializer.serialize_i64(i.clone()),
        }
    }
}

struct IDVisitor;
impl<'de> Visitor<'de> for IDVisitor {
    type Value = ID;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("unable to parse ID - was not Bson or Json string")
    }

    fn visit_map<M>(self, access: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        // send this back into the Bson deserializer
        Ok(ID::with_bson(&Bson::deserialize(
            de::value::MapAccessDeserializer::new(access),
        )?))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ID::from_string(v))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ID::from_string(v))
    }

    fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ID::Int64(v))
    }

    fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Ok(ID::Int64(v as i64))
    }
}

impl<'de> Deserialize<'de> for ID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(IDVisitor)
    }
}

impl fmt::Display for ID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl From<String> for ID {
    fn from(s: String) -> ID {
        ID::from_string(s)
    }
}

impl From<ID> for String {
    fn from(id: ID) -> String {
        match id {
            ID::ObjectId(o) => format!("$oid:{}", o.to_hex()),
            ID::String(s) => s,
            ID::Int64(i) => i.to_string(),
        }
    }
}

impl From<i64> for ID {
    fn from(i: i64) -> ID {
        ID::Int64(i)
    }
}

impl From<ObjectId> for ID {
    fn from(o: ObjectId) -> ID {
        ID::ObjectId(o)
    }
}

impl ID {
    pub fn from_string<S: Into<String>>(value: S) -> Self {
        let s: String = value.into();
        if s.starts_with("$oid:") {
            match ObjectId::with_string(&s[5..]) {
                Ok(oid) => ID::ObjectId(oid),
                Err(_) => ID::String(s),
            }
        } else {
            ID::String(s.into())
        }
    }

    /// Construct a new ID from anything implementing `Into<String>`
    pub fn with_string<S: Into<String>>(value: S) -> Self {
        ID::String(value.into())
    }

    pub fn with_i64<I: Into<i64>>(value: I) -> Self {
        ID::Int64(value.into())
    }

    pub fn with_oid(value: ObjectId) -> Self {
        ID::ObjectId(value)
    }

    pub fn with_bson(value: &Bson) -> Self {
        match value.into() {
            Bson::String(s) => ID::String(s),
            Bson::ObjectId(o) => ID::ObjectId(o),
            Bson::Int64(i) => ID::Int64(i),
            _ => panic!("Invalid id type used {:?}", value),
        }
    }

    pub fn to_bson(&self) -> Bson {
        match self {
            ID::ObjectId(o) => Bson::ObjectId(o.clone()),
            ID::String(s) => Bson::String(s.to_string()),
            ID::Int64(i) => Bson::Int64(i.clone()),
        }
    }

    pub fn to_string(&self) -> String {
        self.clone().into()
    }
}

impl From<ID> for ObjectId {
    fn from(id: ID) -> ObjectId {
        match id {
            ID::ObjectId(o) => o,
            ID::String(s) => ObjectId::with_string(&s).unwrap(),
            ID::Int64(i) => ObjectId::with_string(&i.to_string()).unwrap(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_id_from_normal_string() {
        let test_string = String::from("Something");
        assert_eq!(
            ID::from_string(test_string),
            ID::String(String::from("Something"))
        );
        let test_str = "SomethingElse";
        assert_eq!(
            ID::from_string(test_str),
            ID::String(String::from("SomethingElse"))
        );
    }

    #[test]
    fn test_convert_id_from_oid() {
        let test_string = String::from("$oid:5eaefffa00c9fdf000c46fdc");
        assert_eq!(
            ID::from_string(test_string),
            ID::ObjectId(ObjectId::with_string("5eaefffa00c9fdf000c46fdc").unwrap())
        );
    }

    #[test]
    fn test_convert_invalid_id() {
        assert_eq!(
            ID::from_string("$oid:not_valid"),
            ID::String("$oid:not_valid".to_string())
        );
    }

    #[test]
    fn test_convert_id_from_number() {
        assert_eq!(ID::from(64 as i64), ID::Int64(64));
        assert_eq!(ID::with_i64(32 as u32), ID::Int64(32));
        assert_eq!(ID::with_i64(8 as u8), ID::Int64(8));
    }
}
