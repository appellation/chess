use async_std::task::spawn_blocking;
use std::convert::TryInto;
use tide::{http::Mime, Body, Error, Request, Response, Result, StatusCode};

mod assets;
mod render;

async fn handle_request(req: Request<()>) -> Result {
	let board: chess::Board =
		urlencoding::decode(req.url().path().strip_prefix("/").unwrap_or_default())?
			.as_str()
			.try_into()?;

	let body = spawn_blocking(move || {
		let svg: render::Svg = board.try_into()?;
		Ok::<Vec<u8>, Error>(svg.try_into()?)
	})
	.await?;

	let mut res = Response::new(StatusCode::Ok);
	res.set_body(body);
	res.set_content_type("image/png");
	Ok(res)
}

#[async_std::main]
async fn main() -> Result<()> {
	tide::log::start();

	let mut app = tide::new();
	app.at("/*").get(handle_request);

	app.listen("0.0.0.0:8081").await?;
	Ok(())
}
