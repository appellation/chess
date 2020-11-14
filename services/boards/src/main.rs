use async_std::task::spawn_blocking;
use std::convert::TryInto;
use tide::{Request, Result, http::Mime, Response, StatusCode, Body, Error};

mod assets;
mod render;

async fn handle_request(req: Request<()>) -> Result {
	let board: chess::Board = urlencoding::decode(req.url().path().strip_prefix("/").unwrap_or_default())?.as_str().try_into()?;

	let accept = req
		.header("accept")
		.map(|h| h.last())
		.and_then(|h| h.as_str().parse::<Mime>().ok());

	match accept.as_ref().map(|mime| mime.essence()) {
		Some("text/plain") | Some("*/*") => Ok(Body::from_string(board.to_string()).into()),
		Some("image/png") => {
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
		_ => Ok(Response::new(StatusCode::NotAcceptable)),
	}
}

#[async_std::main]
async fn main() -> Result<()> {
	tide::log::start();

	let mut app = tide::new();
	app.at("/*").get(handle_request);

	app.listen("0.0.0.0:8081").await?;
	Ok(())
}
