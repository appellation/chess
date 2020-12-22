use crate::{
	models::{db, game::Game, user::User},
	State,
};
use sqlx::types::Uuid;
use std::{convert::TryInto, future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_game<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let user = req.ext::<User>().unwrap();
		let game_id = req.param("game_id")?;
		let pool = &req.state().db;
		if game_id == "current" {
			let mut games = sqlx::query_as!(
				db::Game,
				r#"select games.id, games.white_id, games.black_id, games.board, games.moves, games.result, games.created_at, games.modified_at
from games
left join users on users.id = games.black_id
	or users.id = games.white_id
where users.id = $1 and games.result is null
limit 2"#,
				user.id
			)
			.fetch_all(pool)
			.await?;

			if games.len() == 1 {
				req.set_ext::<Game>(games.remove(0).try_into()?);
				Ok(next.run(req).await)
			} else {
				Err(tide::Error::from_str(
					StatusCode::BadRequest,
					"multiple games in progress",
				))
			}
		} else if game_id == "previous" {
			let maybe_game = sqlx::query_as!(
				db::Game,
				r#"select games.id, games.white_id, games.black_id, games.board, games.moves, games.result, games.created_at, games.modified_at
from games
left join users on users.id = games.black_id
	or users.id = games.white_id
where users.id = $1 and games.result is not null
order by modified_at desc
limit 1"#,
				user.id
			)
			.fetch_optional(pool)
			.await?;

			match maybe_game {
				Some(game) => {
					req.set_ext::<Game>(game.try_into()?);
					Ok(next.run(req).await)
				}
				None => Ok(Response::new(StatusCode::NotFound)),
			}
		} else {
			let game_id = game_id.parse::<Uuid>()?;
			let maybe_game =
				sqlx::query_as!(db::Game, "select * from games where id = $1", game_id)
					.fetch_optional(pool)
					.await?;

			match maybe_game {
				Some(game) => {
					req.set_ext::<Game>(game.try_into()?);
					Ok(next.run(req).await)
				}
				None => Ok(Response::new(StatusCode::NotFound)),
			}
		}
	})
}
