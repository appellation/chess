use super::user::UserWithAccounts;
use async_std::prelude::*;
use chess::{Board, Color, ChessMove, GameResult};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgRow, types::Uuid, FromRow, Row};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub white_id: Uuid,
	#[serde(with = "crate::serde::uuid")]
	pub black_id: Uuid,
	pub board: chess::Game,
	pub side_to_move: Color,
	pub moves: Vec<ChessMove>,
	pub result: Option<GameResult>,
}

impl Game {
	pub async fn with_users<'exec, E>(self, conn: E) -> Result<GameWithUsers, sqlx::Error>
	where
		E: sqlx::Executor<'exec, Database = sqlx::Postgres> + Copy,
	{
		let white_fut = UserWithAccounts::fetch(&self.white_id, conn);
		let black_fut = UserWithAccounts::fetch(&self.black_id, conn);
		let (white, black) = white_fut.try_join(black_fut).await?;

		Ok(GameWithUsers {
			id: self.id,
			white,
			black,
			board: self.board,
			moves: self.moves,
			result: self.result,
		})
	}
}

impl<'a> FromRow<'a, PgRow> for Game {
	fn from_row(row: &PgRow) -> sqlx::Result<Self> {
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
			side_to_move: game.side_to_move(),
			board: game,
			moves,
			result: row
				.try_get::<Option<String>, _>("result")?
				.and_then(|res| GameResult::from_str(&res).ok()),
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameWithUsers {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
	pub white: UserWithAccounts,
	pub black: UserWithAccounts,
	pub board: chess::Game,
	pub moves: Vec<ChessMove>,
	pub result: Option<GameResult>,
}
