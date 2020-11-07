use crate::{models::game::Game, util::get_user_id, State};
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
	let user_id = get_user_id(&req)?;

	let side_to_move = game.board.side_to_move();

	match (
		side_to_move,
		user_id == game.white_id,
		user_id == game.black_id,
	) {
		(Color::White, true, false) | (Color::Black, false, true) => {
			let chess_move = ChessMove::from_san(&game.board.current_position(), &move_request.san)
				.map_err(|e| tide::Error::from_str(tide::StatusCode::BadRequest, e.to_string()))?;
			game.board.make_move(chess_move);

			let mut conn = req.state().db.acquire().await?;
			sqlx::query!(
				"update games set board = $1 where id = $2",
				game.board.current_position().to_string(),
				game.id
			)
			.execute(&mut conn)
			.await?;

			Ok(tide::Body::from_json(&game)?.into())
		}
		_ => Ok(tide::Response::new(tide::StatusCode::BadRequest)),
	}
}
