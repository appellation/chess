use crate::{
	models::{game::Game, user::User},
	State,
};
use sqlx::{prelude::*, types::Uuid};
use std::{future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_game<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let user = req.ext::<User>().unwrap();
		let game_id = dbg!(req.param("game_id"))?;
		let mut conn = req.state().db.acquire().await?;
		if game_id == "current" {
			let mut games = sqlx::query_as::<_, Game>(
				r#"select *
from games
left join users on users.id = games.black_id
	or users.id = games.white_id
where users.id = $1 and games.result is null
limit 2"#,
			)
			.bind(user.id)
			.fetch_all(&mut conn)
			.await?;

			if games.len() == 1 {
				req.set_ext(games.remove(0));
				Ok(next.run(req).await)
			} else {
				Err(tide::Error::from_str(
					StatusCode::BadRequest,
					"multiple games in progress",
				))
			}
		} else {
			let game_id = game_id.parse::<Uuid>()?;
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
		}
	})
}
