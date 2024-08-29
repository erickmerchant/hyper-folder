use crate::Error;
use anyhow::Result;
use axum::{
	extract::{Request, State},
	http::{header, StatusCode},
	response::{IntoResponse, Response},
};
use camino::Utf8Path;
use std::{fs, sync::Arc};

pub async fn handler(
	State(state): State<Arc<crate::State>>,
	request: Request,
) -> Result<Response, Error> {
	let path = request.uri().path();
	let is_index = path.ends_with('/');
	let path = path.trim_start_matches('/');
	let mut path = Utf8Path::new(state.args.directory.as_str()).join(path);

	if is_index {
		path.push("index.html");
	}

	if let Some(content_type) = mime_guess::from_path(&path).first() {
		if let Ok(body) = fs::read(path) {
			return Ok((
				StatusCode::OK,
				[(
					header::CONTENT_TYPE,
					format!("{content_type}; charset=utf-8"),
				)],
				body,
			)
				.into_response());
		}
	}

	Ok(StatusCode::NOT_FOUND.into_response())
}
