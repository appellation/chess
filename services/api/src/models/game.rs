use super::{db, user::{User, UserWithAccounts}};
use async_std::prelude::*;
use chess::{ChessMove, Color, GameResult};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::convert::TryFrom;

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

impl TryFrom<db::Game> for Game {
	type Error = chess::Error;

	fn try_from(game: db::Game) -> Result<Self, Self::Error> {
		let board: chess::Game = game.board.parse()?;
		Ok(Self {
			id: game.id,
			white_id: game.white_id,
			black_id: game.black_id,
			side_to_move: board.side_to_move(),
			board,
			moves: game
				.moves
				.into_iter()
				.map(|m| m.parse())
				.collect::<Result<_, _>>()?,
			result: game.result.and_then(|res| res.parse().ok()),
		})
	}
}

impl Game {
	pub fn color_of(&self, user: &User) -> Option<Color> {
		if user.id == self.black_id {
			Some(Color::Black)
		} else if user.id == self.white_id {
			Some(Color::White)
		} else {
			None
		}
	}

	pub fn reload(&mut self) -> &Self {
		self.side_to_move = self.board.side_to_move();
		self.result = self.board.result();
		self
	}

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
			side_to_move: self.side_to_move,
			moves: self.moves,
			result: self.result,
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
	pub side_to_move: Color,
	pub moves: Vec<ChessMove>,
	pub result: Option<GameResult>,
}
