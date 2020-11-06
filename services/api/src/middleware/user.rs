use crate::{models::user::User, State};
use mongodb::bson::{doc, from_document, oid::ObjectId};
use std::{future::Future, pin::Pin};
use tide::{Next, Request, Response, Result, StatusCode};

pub fn get_user<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async {
		let user_id = req.header("x-user-id");
		match user_id {
			Some(user_id) => {
				let user = req
					.state()
					.db
					.collection("users")
					.find_one(
						doc! { "_id": ObjectId::with_string(&user_id.last().to_string())? },
						None,
					)
					.await?;

				match user {
					Some(doc) => {
						let user: User = from_document(doc)?;
						req.set_ext(user);
						Ok(next.run(req).await)
					}
					None => Ok(Response::new(StatusCode::Unauthorized)),
				}
			}
			None => Ok(Response::new(StatusCode::Unauthorized)),
		}
	})
}
