use crate::{
	models::user::{AccountType, User},
	State,
};
use serde::Deserialize;
#[cfg(not(feature = "sql-validation"))]
use sqlx::prelude::*;
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

		let mut conn = req.state().db.acquire().await?;

		let user: User = match account_type {
			#[cfg(feature = "sql-validation")]
			Some(account_type) => {
				sqlx::query_as!(
					User,
					"select * from get_or_create_user($1, $2)",
					account_id,
					account_type
				)
				.fetch_one(&mut conn)
				.await?
			}
			#[cfg(not(feature = "sql-validation"))]
			Some(account_type) => {
				sqlx::query_as("select * from get_or_create_user($1, $2)")
					.bind(account_id)
					.bind(account_type)
					.fetch_one(&mut conn)
					.await?
			}
			#[cfg(feature = "sql-validation")]
			None => {
				let account_id = account_id.parse::<Uuid>()?;
				sqlx::query_as!(User, "select * from users where id = $1", account_id)
					.fetch_one(&mut conn)
					.await
					.map_err(|_| Error::from_str(StatusCode::Unauthorized, "user doesn't exist"))?
			}
			#[cfg(not(feature = "sql-validation"))]
			None => {
				let account_id = account_id.parse::<Uuid>()?;
				sqlx::query_as("select * from users where id = $1")
					.bind(account_id)
					.fetch_one(&mut conn)
					.await
					.map_err(|_| Error::from_str(StatusCode::Unauthorized, "user doesn't exist"))?
			}
		};

		req.set_ext(user);
		Ok(next.run(req).await)
	})
}
