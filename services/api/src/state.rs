use sqlx::postgres::PgPool;

#[derive(Debug, Clone)]
pub struct State {
	pub db: PgPool,
}
