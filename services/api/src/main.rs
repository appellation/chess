mod assets;
mod middleware;
mod models;
mod routes;
mod state;

pub use state::State;

#[async_std::main]
async fn main() -> tide::Result<()> {
	tide::log::start();

	let mongodb = mongodb::Client::with_uri_str("mongodb+srv://chess:gAu7mwq8xSvQr5jj@cluster0.ugntk.mongodb.net/chess?retryWrites=true&w=majority").await?;

	let state = State {
		db: mongodb.database("chess"),
	};

	let mut app = tide::with_state(state);
	app.with(middleware::user::get_user);
	app.at("/games").post(routes::games::create_game);
	app.at("/games/:game_id")
		.with(middleware::game::get_game)
		.get(routes::games::get_game);
	app.at("/games/:game_id/moves")
		.with(middleware::game::get_game)
		.put(routes::games::moves::make_move);
	app.at("/games/:game_id/board")
		.with(middleware::game::get_game)
		.get(routes::games::board::get_board);
	app.listen("127.0.0.1:8080").await?;
	Ok(())
}
