use crate::{State, models::game::{Game, UserColor}};
use chess::{ChessMove, Color};
use serde::Deserialize;
use tide::{Request, StatusCode};

#[derive(Debug, Deserialize, Eq, PartialEq)]
#[serde(tag = "action", content = "data")]
enum MoveRequest {
	MakeMove(String),
	AcceptDraw,
	OfferDraw,
	DeclareDraw,
	Resign,
}

pub async fn make_move(mut req: Request<State>) -> tide::Result {
	let move_request: MoveRequest = req.body_json().await?;
	let mut game = req.ext::<Game>().unwrap().clone();
	let user_color = req.ext::<UserColor>().unwrap().clone();

	let is_users_turn = user_color == game.side_to_move;

	let pool = &req.state().db;
	let mut txn = pool.begin().await?;

	match move_request {
		MoveRequest::MakeMove(san) if is_users_turn => {
			let board_move = ChessMove::from_san(&game.board.current_position(), &san)?;

			game.board.make_move(board_move);
			game.moves.push(san.clone());

			sqlx::query!(
				"update games set moves = array_append(moves, $1) where id = $2",
				san,
				game.id
			)
			.execute(&mut txn)
			.await?;
		}
		MoveRequest::AcceptDraw if is_users_turn => {
			game.board.accept_draw();
		}
		MoveRequest::OfferDraw => {
			game.board.offer_draw(Option::<Color>::from(user_color).unwrap_or(Color::Black));
		}
		MoveRequest::DeclareDraw => {
			game.board.declare_draw();
		}
		MoveRequest::Resign => {
			game.board.resign(Option::<Color>::from(user_color).unwrap_or(Color::Black));
		}
		_ => return Ok(tide::Error::from_str(StatusCode::BadRequest, "Not your turn").into()),
	}

	game.reload();

	let result: Option<&str> = game.board.result().map(|r| r.into());
	sqlx::query!(
		"update games set board = $1, result = $2, modified_at = now() where id = $3",
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
