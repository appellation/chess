use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};
use std::fmt;
use strum::IntoStaticStr;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoStaticStr)]
pub enum AccountType {
	Discord,
}

impl fmt::Display for AccountType {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{:?}", self)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAccount {
	#[serde(with = "crate::serde::uuid")]
	pub user_id: Uuid,
	pub account_id: String,
	pub account_type: AccountType,
}
