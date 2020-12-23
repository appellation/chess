use super::{
	db,
	r#move::EndOfGameState,
	user::{User, UserWithAccounts},
};
use async_std::prelude::*;
use itertools::Itertools;
use chess::{Color, GameResult};
use serde::{Deserialize, Serialize};
use sqlx::types::Uuid;
use std::{
	convert::TryFrom,
	fmt::{self, Display, Formatter},
};

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
	pub moves: Vec<String>,
	pub result: Option<GameResult>,
	pub pgn: Pgn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pgn(String);

impl Display for Pgn {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		f.write_str(&self.0)
	}
}

impl<'game, 'moves> From<(&'game chess::Game, &'moves [String])> for Pgn {
	fn from(value: (&'game chess::Game, &'moves [String])) -> Self {
		let mut pgn = value
			.1
			.chunks(2)
			.enumerate()
			.map(|(i, moves)| {
				format!(
					"{}. {}",
					i + 1,
					moves.join(" ")
				)
			})
			.intersperse(" ".to_owned())
			.collect::<String>();
		pgn += &format!(" {}", EndOfGameState::from(value.0.result()));
		Pgn(pgn)
	}
}

impl TryFrom<db::Game> for Game {
	type Error = chess::Error;

	fn try_from(game: db::Game) -> Result<Self, Self::Error> {
		let board: chess::Game = game.board.parse()?;
		let pgn: Pgn = (&board, game.moves.as_slice()).into();

		Ok(Self {
			id: game.id,
			white_id: game.white_id,
			black_id: game.black_id,
			side_to_move: board.side_to_move(),
			board,
			moves: game.moves,
			pgn,
			result: game.result.and_then(|res| res.parse().ok()),
		})
	}
}

impl Game {
	pub fn color_of(&self, user: &User) -> Option<UserColor> {
		if user.id == self.black_id && user.id == self.white_id {
			Some(UserColor::Both)
		} else if user.id == self.black_id {
			Some(UserColor::Black)
		} else if user.id == self.white_id {
			Some(UserColor::White)
		} else {
			None
		}
	}

	pub fn reload(&mut self) -> &Self {
		self.side_to_move = self.board.side_to_move();
		self.result = self.board.result();
		self.pgn = (&self.board, self.moves.as_slice()).into();
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
			pgn: self.pgn,
		})
	}
}

#[derive(Debug, Clone)]
pub enum UserColor {
	White,
	Black,
	Both,
}

impl PartialEq<Color> for UserColor {
	fn eq(&self, other: &Color) -> bool {
		match self {
			Self::White => other == &Color::White,
			Self::Black => other == &Color::Black,
			Self::Both => true,
		}
	}
}

impl From<UserColor> for Option<Color> {
	fn from(color: UserColor) -> Self {
		match color {
			UserColor::White => Some(Color::White),
			UserColor::Black => Some(Color::Black),
			UserColor::Both => None,
		}
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
	pub moves: Vec<String>,
	pub result: Option<GameResult>,
	pub pgn: Pgn,
}
