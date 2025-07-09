#[cfg(test)]
mod tests;

use anyhow::Result;
use axum::serve;
use hyper_folder::{get_app, state};
use tokio::net::TcpListener;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
	let state = state::State::from_env();

	tracing_subscriber::fmt()
		.with_env_filter(EnvFilter::try_from_default_env().unwrap_or_default())
		.init();

	let app = get_app(state.clone());
	let listener = TcpListener::bind(("0.0.0.0", state.args.port))
		.await
		.expect("should listen");

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");

	Ok(())
}
