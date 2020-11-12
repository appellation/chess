mod assets;
mod middleware;
mod models;
mod routes;
mod serde;
mod state;

use sqlx::postgres::PgPool;
pub use state::State;
use std::{env, time::Duration};

#[async_std::main]
async fn main() -> tide::Result<()> {
	#[cfg(debug_assertions)]
	dotenv::dotenv()?;
	tide::log::start();

	let pool = PgPool::builder()
		.max_size(5)
		.connect_timeout(Duration::new(10, 0))
		.build(&env::var("DATABASE_URL")?)
		.await?;

	let state = State { db: pool };

	let mut app = tide::with_state(state);
	app.at("/games")
		.with(middleware::user::get_user)
		.post(routes::games::create_game);
	app.at("/games/:game_id")
		.with(middleware::user::get_user)
		.with(middleware::game::get_game)
		.get(routes::games::get_game);
	app.at("/games/:game_id/moves")
		.with(middleware::user::get_user)
		.with(middleware::game::get_game)
		.with(middleware::user::get_user)
		.put(routes::games::moves::make_move);
	app.at("/games/:game_id/board")
		.with(middleware::user::get_user)
		.with(middleware::game::get_game)
		.get(routes::games::board::get_board);
	app.listen("0.0.0.0:8080").await?;
	Ok(())
}
