use mongodb::bson::oid::ObjectId;
use serde::{
	de::{Deserializer, Error, Unexpected, Visitor},
	ser::Serializer,
	Deserialize, Serialize,
};
use std::{fmt, str::FromStr};

struct GameVisitor;

impl<'de> Visitor<'de> for GameVisitor {
	type Value = chess::Game;

	fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
	where
		E: Error,
	{
		Self::Value::from_str(&v).map_err(|_| Error::invalid_value(Unexpected::Str(&v), &self))
	}

	fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		write!(formatter, "FEN string")
	}
}

fn serialize_game<S>(game: &chess::Game, s: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	s.serialize_str(&game.current_position().to_string())
}

fn deserialize_game<'de, D>(d: D) -> Result<chess::Game, D::Error>
where
	D: Deserializer<'de>,
{
	d.deserialize_string(GameVisitor {})
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
	pub _id: ObjectId,
	pub white_id: ObjectId,
	pub black_id: ObjectId,
	#[serde(
		serialize_with = "serialize_game",
		deserialize_with = "deserialize_game"
	)]
	pub board: chess::Game,
}
