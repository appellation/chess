use serde::{
	de::{Deserializer, Error, Unexpected, Visitor},
	ser::Serializer,
};
use sqlx::types::Uuid;
use std::fmt;

struct UuidVisitor;

impl<'de> Visitor<'de> for UuidVisitor {
	type Value = Uuid;

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: Error,
	{
		Self::Value::parse_str(v).map_err(|_| Error::invalid_value(Unexpected::Str(&v), &self))
	}

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "UUID string")
	}
}

pub fn serialize<S>(uuid: &Uuid, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&uuid.to_string())
}

pub fn deserialize<'de, D>(d: D) -> Result<Uuid, D::Error>
where
	D: Deserializer<'de>,
{
	d.deserialize_string(UuidVisitor {})
}
