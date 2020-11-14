use serde::{Deserialize, Serialize};
use sqlx::{types::Uuid, FromRow};
use std::str::FromStr;
use strum::{IntoStaticStr, EnumString};

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, IntoStaticStr, EnumString)]
pub enum AccountType {
	Discord,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserAccount {
	#[serde(with = "crate::serde::uuid")]
	pub user_id: Uuid,
	pub account_id: String,
	pub account_type: AccountType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithAccounts {
	#[serde(with = "crate::serde::uuid")]
	pub id: Uuid,
	pub accounts: Vec<UserAccount>,
}

impl UserWithAccounts {
	pub async fn fetch<'exec, E>(id: &Uuid, conn: E) -> Result<Self, sqlx::Error>
	where
		E: sqlx::Executor<'exec, Database = sqlx::Postgres>
	{
		let accounts = sqlx::query!("select * from user_accounts where user_id = $1", id)
			.fetch_all(conn)
			.await?;

		Ok(UserWithAccounts {
			id: id.clone(),
			accounts: accounts.into_iter().map(|account| Ok(UserAccount {
				user_id: id.clone(),
				account_id: account.account_id,
				account_type: AccountType::from_str(&account.account_type).map_err(|e| sqlx::Error::Decode(Box::new(e)))?
			})).collect::<Result<_, sqlx::Error>>()?
		})
	}
}
