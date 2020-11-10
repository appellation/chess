use crate::{
	models::{game::Game, user::User},
	State,
};
use chess::Color;
use serde::Deserialize;
#[cfg(not(feature = "sql-validation"))]
use sqlx::prelude::*;
use sqlx::types::Uuid;
use tide::Request;

pub mod board;
pub mod moves;

#[derive(Debug, Deserialize, Eq, PartialEq)]
enum CreateGameSide {
	White,
	Black,
	Random,
}

impl From<CreateGameSide> for Color {
	fn from(side: CreateGameSide) -> Self {
		match side {
			CreateGameSide::Black => Self::Black,
			CreateGameSide::White => Self::White,
			CreateGameSide::Random => {
				if rand::random::<bool>() {
					Self::Black
				} else {
					Self::White
				}
			}
		}
	}
}

impl Default for CreateGameSide {
	fn default() -> Self {
		CreateGameSide::Random
	}
}

#[derive(Debug, Deserialize)]
struct CreateGame {
	#[serde(with = "crate::serde::uuid")]
	target_id: Uuid,
	#[serde(default)]
	side: CreateGameSide,
}

pub async fn create_game(mut req: Request<State>) -> tide::Result {
	let body: CreateGame = req.body_json().await?;
	let user = req.ext::<User>().unwrap();

	let mut conn = req.state().db.acquire().await?;

	let (white_id, black_id) = match body.side.into() {
		Color::Black => (body.target_id, user.id),
		Color::White => (user.id, body.target_id),
	};

	#[cfg(not(feature = "sql-validation"))]
	let id: Uuid = sqlx::query(
		"insert into games (white_id, black_id, board) values ($1, $2, $3) returning id",
	)
	.bind(white_id)
	.bind(black_id)
	.bind(chess::Board::default().to_string())
	.fetch(&mut conn)
	.next()
	.await?
	.ok_or_else(|| {
		tide::Error::from_str(
			tide::StatusCode::InternalServerError,
			"unable to create game",
		)
	})?
	.try_get("id")?;

	#[cfg(feature = "sql-validation")]
	let id = sqlx::query!(
		"insert into games (white_id, black_id, board) values ($1, $2, $3) returning id",
		white_id,
		black_id,
		chess::Board::default().to_string()
	)
	.fetch_one(&mut conn)
	.await?
	.id;

	Ok(tide::Body::from_string(id.to_string()).into())
}

pub async fn get_game(req: Request<State>) -> tide::Result {
	let game: &Game = req.ext().unwrap();
	Ok(tide::Body::from_json(game)?.into())
}
