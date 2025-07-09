pub mod error;
pub mod routes;
pub mod state;

use axum::Router;
use routes::file;
use std::sync::Arc;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub fn get_app(state: state::State) -> Router {
	let state = Arc::new(state);

	Router::new()
		.fallback(file::file_handler)
		.with_state(state.clone())
		.layer(CompressionLayer::new())
		.layer(TraceLayer::new_for_http())
}
