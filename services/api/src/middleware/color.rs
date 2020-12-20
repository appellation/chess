use crate::{
	models::{game::Game, user::User},
	State,
};
use std::{future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_user_color<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let user = req.ext::<User>().unwrap();
		let game = req.ext::<Game>().unwrap();

		match game.color_of(user) {
			Some(user_color) => {
				req.set_ext(user_color);
				Ok(next.run(req).await)
			}
			None => Ok(Response::new(StatusCode::NotFound)),
		}
	})
}
