use crate::{
	models::user::{AccountType, User},
	State,
};
use serde::Deserialize;
use sqlx::types::Uuid;
use std::{future::Future, pin::Pin};
use tide::{Error, Next, Request, Result, StatusCode};

#[derive(Debug, Clone, Deserialize)]
struct AccountTypeQuery {
	account_type: Option<AccountType>,
}

pub fn get_user<'a>(
	mut req: Request<State>,
	next: Next<'a, State>,
) -> Pin<Box<dyn Future<Output = Result> + Send + 'a>> {
	Box::pin(async move {
		let account_type = req.header("x-account-type").map(|h| h.last().as_str());
		let account_id = req
			.header("x-user-id")
			.ok_or_else(|| Error::from_str(StatusCode::Unauthorized, "missing x-user-id header"))?
			.as_str();

		let pool = &req.state().db;

		let user: User = match account_type {
			Some(account_type) => {
				let user = sqlx::query!(
					"select * from get_or_create_user($1, $2)",
					account_id,
					account_type
				)
				.fetch_one(pool)
				.await?;

				User {
					id: user.id.unwrap(),
				}
			}
			None => {
				let account_id = account_id.parse::<Uuid>()?;
				sqlx::query_as!(User, "select * from users where id = $1", account_id)
					.fetch_one(pool)
					.await
					.map_err(|_| Error::from_str(StatusCode::Unauthorized, "user doesn't exist"))?
			}
		};

		req.set_ext(user);
		Ok(next.run(req).await)
	})
}
