use crate::{models::game::Game, State};
use mongodb::bson::{doc, from_document, oid::ObjectId};
use std::{future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_game<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let game_id = dbg!(ObjectId::with_string(req.param("game_id")?)?);
		let maybe_game = req
			.state()
			.db
			.collection("games")
			.find_one(doc! { "_id": game_id.clone() }, None)
			.await?;

		match maybe_game {
			Some(doc) => {
				let game: Game = from_document(doc)?;
				req.set_ext(game);
				Ok(next.run(req).await)
			}
			None => Ok(Response::new(StatusCode::NotFound)),
		}
	})
}
