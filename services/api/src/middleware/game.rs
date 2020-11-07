use crate::{models::game::Game, State};
use sqlx::{prelude::*, types::Uuid};
use std::{future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_game<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let game_id = req.param("game_id")?.parse::<Uuid>()?;
		let mut conn = req.state().db.acquire().await?;
		let maybe_game = sqlx::query_as::<_, Game>("select * from games where id = $1")
			.bind(game_id)
			.fetch_optional(&mut conn)
			.await?;

		match maybe_game {
			Some(game) => {
				req.set_ext(game);
				Ok(next.run(req).await)
			}
			None => Ok(Response::new(StatusCode::NotFound)),
		}
	})
}
