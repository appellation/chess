use crate::{
	models::{
		db,
		game::Game,
		user::{AccountType, User},
	},
	State,
};
use chess::Color;
use serde::Deserialize;
use std::convert::TryInto;
use tide::Request;

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
	target_id: String,
	account_type: Option<AccountType>,
	#[serde(default)]
	side: CreateGameSide,
}

pub async fn create_game(mut req: Request<State>) -> tide::Result {
	let body: CreateGame = req.body_json().await?;
	let user = req.ext::<User>().unwrap();

	let pool = &req.state().db;

	let target_id = match body.account_type {
		Some(account_type) => {
			let account_type: &str = account_type.into();
			sqlx::query!(
				"select id from get_or_create_user($1, $2)",
				body.target_id,
				account_type
			)
			.fetch_one(pool)
			.await?
			.id
			.unwrap()
		}
		None => body.target_id.parse()?,
	};

	let (white_id, black_id) = match body.side.into() {
		Color::Black => (target_id, user.id),
		Color::White => (user.id, target_id),
	};

	let is_already_playing = sqlx::query!("select id from games where (white_id = $1 or black_id = $1 or white_id = $2 or black_id = $2) and result is null", white_id, black_id)
		.fetch_optional(pool)
		.await?
		.is_some();

	if is_already_playing {
		let mut res = tide::Response::new(tide::StatusCode::BadRequest);
		res.set_body("already playing");
		return Ok(res);
	}

	let game: Game = sqlx::query_as!(
		db::Game,
		"insert into games (white_id, black_id, board) values ($1, $2, $3) returning *",
		white_id,
		black_id,
		chess::Board::default().to_string()
	)
	.fetch_one(pool)
	.await?
	.try_into()?;

	let game = game.with_users(pool).await?;
	Ok(tide::Body::from_json(&game)?.into())
}

pub async fn get_game(req: Request<State>) -> tide::Result {
	let pool = &req.state().db;
	let game = req.ext::<Game>().unwrap().clone().with_users(pool).await?;
	Ok(tide::Body::from_json(&game)?.into())
}
