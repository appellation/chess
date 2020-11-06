use crate::{
	models::{game::Game, user::User},
	State,
};
use chess::{ChessMove, Color};
use mongodb::bson::doc;
use serde::{Deserialize, Serialize};
use tide::Request;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct MoveRequest {
	san: String,
}

pub async fn make_move(mut req: Request<State>) -> tide::Result {
	let move_request: MoveRequest = req.body_json().await?;
	let mut game = req.ext::<Game>().unwrap().clone();
	let user: &User = req.ext::<User>().unwrap();

	let side_to_move = game.board.side_to_move();

	match (
		side_to_move,
		user._id == game.white_id,
		user._id == game.black_id,
	) {
		(Color::White, true, false) | (Color::Black, false, true) => {
			let chess_move = ChessMove::from_san(&game.board.current_position(), &move_request.san)
				.map_err(|e| tide::Error::from_str(tide::StatusCode::BadRequest, e.to_string()))?;
			game.board.make_move(chess_move);

			req.state()
				.db
				.collection("games")
				.update_one(
					doc! { "_id": &game._id },
					doc! { "$set": { "board": game.board.current_position().to_string() } },
					None,
				)
				.await?;

			Ok(tide::Body::from_json(&game)?.into())
		}
		_ => Ok(tide::Response::new(tide::StatusCode::BadRequest)),
	}
}
