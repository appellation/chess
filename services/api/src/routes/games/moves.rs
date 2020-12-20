use crate::{State, models::game::Game};
use chess::{ChessMove, Color};
use serde::Deserialize;
use tide::{Request, StatusCode};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "action", content = "data")]
enum MoveRequest {
	MakeMove(ChessMove),
	AcceptDraw,
	OfferDraw,
	DeclareDraw,
	Resign,
}

pub async fn make_move(mut req: Request<State>) -> tide::Result {
	let move_request: MoveRequest = req.body_json().await?;
	let mut game = req.ext::<Game>().unwrap().clone();
	let user_color = req.ext::<Color>().unwrap().clone();

	let is_users_turn = user_color == game.side_to_move;

	let pool = &req.state().db;
	let mut txn = pool.begin().await?;

	match move_request {
		MoveRequest::MakeMove(m) if is_users_turn => {
			game.board.make_move(m);
			game.moves.push(m);
			sqlx::query!(
				"update games set moves = array_append(moves, $1) where id = $2",
				m.to_string(),
				game.id
			)
			.execute(&mut txn)
			.await?;
		}
		MoveRequest::AcceptDraw if is_users_turn => {
			game.board.accept_draw();
		}
		MoveRequest::OfferDraw if is_users_turn => {
			game.board.offer_draw(user_color);
		}
		MoveRequest::DeclareDraw => {
			game.board.declare_draw();
		}
		MoveRequest::Resign => {
			game.board.resign(user_color);
		}
		_ => return Ok(tide::Error::from_str(StatusCode::BadRequest, "Not your turn").into())
	}

	game.reload();

	let result: Option<&str> = game.board.result().map(|r| r.into());
	sqlx::query!(
		"update games set board = $1, result = $2 where id = $3",
		game.board.current_position().to_string(),
		result,
		game.id
	)
	.execute(&mut txn)
	.await?;

	txn.commit().await?;

	let game = game.with_users(pool).await?;
	Ok(tide::Body::from_json(&game)?.into())
}
