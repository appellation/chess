use crate::{
	models::{
		game::Game,
		user::{AccountType, User},
	},
	State,
};
use chess::Color;
use serde::Deserialize;
#[cfg(not(feature = "sql-validation"))]
use sqlx::{postgres::PgRow, prelude::*, types::Uuid};
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
	target_id: String,
	account_type: Option<AccountType>,
	#[serde(default)]
	side: CreateGameSide,
}

pub async fn create_game(mut req: Request<State>) -> tide::Result {
	let body: CreateGame = req.body_json().await?;
	let user = req.ext::<User>().unwrap();

	let mut conn = req.state().db.acquire().await?;

	let target_id = match body.account_type {
		#[cfg(feature = "sql-validation")]
		Some(account_type) => {
			let account_type: &str = account_type.into();
			sqlx::query!(
				"select id from get_or_create_user($1, $2)",
				body.target_id,
				account_type
			)
			.fetch_one(&mut conn)
			.await?
			.id
			.unwrap()
		}
		#[cfg(not(feature = "sql-validation"))]
		Some(account_type) => {
			let account_type: &str = account_type.into();
			sqlx::query("select id from get_or_create_user($1, $2)")
				.bind(body.target_id)
				.bind(account_type)
				.map(|row: PgRow| row.try_get("id"))
				.fetch_one(&mut conn)
				.await??
		}
		None => body.target_id.parse()?,
	};

	let (white_id, black_id) = match body.side.into() {
		Color::Black => (target_id, user.id),
		Color::White => (user.id, target_id),
	};

	#[cfg(feature = "sql-validation")]
	let is_already_playing = sqlx::query!("select id from games where (white_id = $1 or black_id = $1) and (white_id = $2 or black_id = $2) and result <> null", white_id, black_id)
		.fetch_optional(&mut conn)
		.await?
		.is_some();

	#[cfg(not(feature = "sql-validation"))]
	let is_already_playing = sqlx::query("select id from games where (white_id = $1 or black_id = $1) and (white_id = $2 or black_id = $2) and result <> null")
		.bind(white_id)
		.bind(black_id)
		.fetch(&mut conn)
		.next()
		.await?
		.is_some();

	if is_already_playing {
		let mut res = tide::Response::new(tide::StatusCode::BadRequest);
		res.set_body("already playing");
		return Ok(res)
	}

	#[cfg(not(feature = "sql-validation"))]
	let id: Uuid = sqlx::query(
		"insert into games (white_id, black_id, board) values ($1, $2, $3) returning id",
	)
	.bind(white_id)
	.bind(black_id)
	.bind(chess::Board::default().to_string())
	.map(|row: PgRow| row.try_get("id"))
	.fetch_one(&mut conn)
	.await??;

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
