use crate::{models::game::Game, util::get_user_id, State};
use async_std::stream::StreamExt;
use chess::Color;
use serde::Deserialize;
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
	let user_id = get_user_id(&req)?;

	let mut conn = req.state().db.acquire().await?;

	let (white_id, black_id) = match body.side.into() {
		Color::Black => (body.target_id, user_id),
		Color::White => (user_id, body.target_id),
	};

	let row = sqlx::query!(
		"insert into games (white_id, black_id, board) values ($1, $2, $3) returning id",
		white_id,
		black_id,
		chess::Board::default().to_string()
	)
	.fetch_one(&mut conn)
	.await?;

	Ok(tide::Body::from_string(row.id.to_string()).into())
}

pub async fn get_game(req: Request<State>) -> tide::Result {
	let game: &Game = req.ext().unwrap();
	Ok(tide::Body::from_json(game)?.into())
}
