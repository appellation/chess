use crate::{
	models::{game::Game, user::User},
	State,
};
use chess::Color;
use mongodb::bson::{doc, oid::ObjectId, to_document};
use serde::Deserialize;
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
	#[serde(default)]
	side: CreateGameSide,
}

pub async fn create_game(mut req: Request<State>) -> tide::Result {
	let body: CreateGame = req.body_json().await?;
	let user: &User = req.ext().unwrap();

	let target = req
		.state()
		.db
		.collection("users")
		.find_one(
			doc! { "_id": ObjectId::with_string(&body.target_id)? },
			None,
		)
		.await?;

	match target {
		None => Ok(tide::Response::new(tide::StatusCode::BadRequest)),
		Some(_) => {
			let (white_id, black_id) = match body.side.into() {
				Color::Black => (ObjectId::with_string(&body.target_id)?, user._id.clone()),
				Color::White => (user._id.clone(), ObjectId::with_string(&body.target_id)?),
			};

			let game = Game {
				_id: ObjectId::default(),
				white_id,
				black_id,
				board: chess::Game::new(),
			};

			let save_result = req
				.state()
				.db
				.collection("games")
				.insert_one(to_document(&game)?, None)
				.await?;

			Ok(tide::Body::from_string(save_result.inserted_id.to_string()).into())
		}
	}
}

pub async fn get_game(req: Request<State>) -> tide::Result {
	let game: &Game = dbg!(req.ext().unwrap());
	Ok(tide::Body::from_json(game)?.into())
}
