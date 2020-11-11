use chess::Board;
use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub white_id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub black_id: Uuid,
	pub board: chess::Game,
	pub moves: Vec<String>,
	pub result: Option<chess::GameResult>,
}

impl<'a> FromRow<'a, sqlx::postgres::PgRow<'a>> for Game {
	fn from_row(row: &sqlx::postgres::PgRow<'a>) -> sqlx::Result<Self> {
		let board: &str = row.try_get("board")?;
		// TODO: make error better
		let board: Board = board.parse().map_err(|_| sqlx::Error::PoolClosed)?;
		let game = chess::Game::new_with_board(board);

		Ok(Self {
			id: row.try_get("id")?,
			white_id: row.try_get("white_id")?,
			black_id: row.try_get("black_id")?,
			board: game,
			moves: row.try_get("moves")?,
			result: None,
		})
	}
}
