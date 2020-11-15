use sqlx::{types::Uuid, FromRow};

#[derive(Debug, Clone, FromRow)]
pub struct Game {
	pub id: Uuid,
	pub white_id: Uuid,
	pub black_id: Uuid,
	pub board: String,
	pub moves: Vec<String>,
	pub result: Option<String>,
}
