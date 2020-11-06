use mongodb::Database;

#[derive(Debug, Clone)]
pub struct State {
	pub db: Database,
}
