mod args;
mod error;
mod routes;
mod state;

use anyhow::Result;
use axum::{serve, Router};
use error::Error;
use routes::fallback;
use state::State;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> Result<()> {
	let state = State::default();
	let port = state.args.port;
	let state = Arc::new(state);

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(tracing::Level::DEBUG)
		.init();

	let app = Router::new()
		.fallback(fallback::handler)
		.with_state(state)
		.layer(CompressionLayer::new())
		.layer(TraceLayer::new_for_http());
	let listener = TcpListener::bind(("0.0.0.0", port))
		.await
		.expect("should listen");

	tracing::debug!("listening on port {}", port);

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");

	Ok(())
}
