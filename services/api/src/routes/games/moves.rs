use crate::{
	models::{game::Game, user::User},
	State,
};
use chess::{ChessMove, Color};
use serde::{Deserialize, Serialize};
use tide::Request;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
struct MoveRequest {
	san: String,
}

pub async fn make_move(mut req: Request<State>) -> tide::Result {
	let move_request: MoveRequest = req.body_json().await?;
	let mut game = req.ext::<Game>().unwrap().clone();
	let user = req.ext::<User>().unwrap();

	let side_to_move = game.board.side_to_move();

	match (
		side_to_move,
		user.id == game.white_id,
		user.id == game.black_id,
	) {
		(Color::White, true, false) | (Color::Black, false, true) => {
			let chess_move = ChessMove::from_san(&game.board.current_position(), &move_request.san)
				.map_err(|e| tide::Error::from_str(tide::StatusCode::BadRequest, e.to_string()))?;
			game.board.make_move(chess_move);
			game.moves.push(chess_move);
			game.side_to_move = !game.side_to_move;
			let result: Option<&str> = game.board.result().map(|r| r.into());

			let pool = &req.state().db;

			sqlx::query!(
				"update games set board = $1, moves = array_append(moves, $2), result = $3 where id = $4",
				game.board.current_position().to_string(),
				chess_move.to_string(),
				result,
				game.id
			)
			.execute(pool)
			.await?;

			let game = game.with_users(pool).await?;
			Ok(tide::Body::from_json(&game)?.into())
		}
		_ => Ok(tide::Response::new(tide::StatusCode::BadRequest)),
	}
}
