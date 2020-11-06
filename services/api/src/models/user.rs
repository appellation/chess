use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct User {
	pub _id: ObjectId,
	pub connected_accounts: Option<Accounts>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Accounts {
	Discord(HashMap<String, DiscordAccount>),
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DiscordAccount {
	id: String,
	access_token: String,
	refresh_token: String,
}
