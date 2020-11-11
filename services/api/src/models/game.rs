use chess::{Board, ChessMove, GameResult};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Uuid, FromRow, Row};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub white_id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub black_id: Uuid,
	pub board: chess::Game,
	pub moves: Vec<ChessMove>,
	pub result: Option<GameResult>,
}

impl<'a> FromRow<'a, PgRow<'a>> for Game {
	fn from_row(row: &PgRow<'a>) -> sqlx::Result<Self> {
		let board = row
			.try_get::<&str, _>("board")?
			.parse::<Board>()
			.map_err(|e| sqlx::Error::Decode(Box::new(e)))?;
		let game = chess::Game::new_with_board(board);

		let moves = row
			.try_get::<Vec<String>, _>("moves")?
			.iter()
			.map(|move_text| ChessMove::from_san(&board, move_text))
			.collect::<Result<_, _>>()
			.map_err(|e| sqlx::Error::Decode(Box::new(e)))?;

		Ok(Self {
			id: row.try_get("id")?,
			white_id: row.try_get("white_id")?,
			black_id: row.try_get("black_id")?,
			board: game,
			moves,
			result: None,
		})
	}
}
