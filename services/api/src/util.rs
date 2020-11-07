use crate::State;
use sqlx::types::Uuid;
use tide::{Error, Request};

pub fn get_user_id(req: &Request<State>) -> Result<Uuid, Error> {
	Ok(req
		.header("x-user-id")
		.ok_or_else(|| tide::Error::from_str(tide::StatusCode::Unauthorized, "missing x-user-id"))?
		.last()
		.as_str()
		.parse()?)
}
