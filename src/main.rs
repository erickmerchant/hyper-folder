mod args;
mod error;
mod routes;

use anyhow::Result;
use args::Args;
use axum::{serve, Router};
use clap::Parser;
use error::Error;
use routes::fallback;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> Result<()> {
	let args = Args::parse();
	let port = args.port;

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(tracing::Level::DEBUG)
		.init();

	let app = Router::new()
		.fallback(fallback::handler)
		.with_state(args)
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
