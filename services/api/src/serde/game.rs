use serde::{
	de::{Deserializer, Error, Unexpected, Visitor},
	ser::Serializer,
};
use std::{fmt, str::FromStr};

struct GameVisitor;

impl<'de> Visitor<'de> for GameVisitor {
	type Value = chess::Game;

	fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
	where
		E: Error,
	{
		Self::Value::from_str(v).map_err(|_| Error::invalid_value(Unexpected::Str(&v), &self))
	}

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "FEN string")
	}
}

pub fn serialize<S>(game: &chess::Game, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&game.current_position().to_string())
}

pub fn deserialize<'de, D>(d: D) -> Result<chess::Game, D::Error>
where
	D: Deserializer<'de>,
{
	d.deserialize_string(GameVisitor {})
}
