use anyhow::Result;
use askama::Template;
use axum::{
	extract::{Request, State},
	http::{header, StatusCode},
	response::{IntoResponse, Response},
	serve, Router,
};
use camino::Utf8Path;
use clap::Parser;
use glob::glob;
use std::fs;
use tokio::net::TcpListener;
use tower_http::{compression::CompressionLayer, trace::TraceLayer};

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		let AppError(err) = self;

		(
			StatusCode::INTERNAL_SERVER_ERROR,
			err.backtrace().to_string(),
		)
			.into_response()
	}
}

impl<E> From<E> for AppError
where
	E: Into<anyhow::Error>,
{
	fn from(err: E) -> Self {
		Self(err.into())
	}
}

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct AppOptions {
	#[arg(long, default_value_t = 8080)]
	pub port: u16,
	pub directory: String,
}

#[tokio::main]
async fn main() -> Result<()> {
	let args = AppOptions::parse();

	tracing_subscriber::fmt()
		.compact()
		.with_max_level(tracing::Level::DEBUG)
		.init();

	let app = Router::new()
		.fallback(handler)
		.with_state(args.clone())
		.layer(CompressionLayer::new())
		.layer(TraceLayer::new_for_http());
	let listener = TcpListener::bind(("0.0.0.0", args.port))
		.await
		.expect("should listen");

	tracing::debug!("listening on port {}", args.port);

	serve(listener, app.into_make_service())
		.await
		.expect("server should start");

	Ok(())
}

#[derive(Template)]
#[template(path = "index.html")]
struct View {
	pub list: Vec<String>,
}

async fn handler(State(args): State<AppOptions>, request: Request) -> Result<Response, AppError> {
	let path = request.uri().path().to_string();
	let is_index = path.to_string().ends_with('/');
	let path = path.trim_start_matches('/').to_string();
	let mut path = Utf8Path::new(args.directory.as_str()).join(path);
	let full_path = path.clone();

	if is_index {
		path.push("index.html");
	}

	if let (Some(content_type), Ok(body)) = (
		path.clone()
			.extension()
			.or(Some("txt"))
			.and_then(|ext| mime_guess::from_ext(ext).first()),
		fs::read(path),
	) {
		Ok((
			StatusCode::OK,
			[(
				header::CONTENT_TYPE,
				format!("{content_type}; charset=utf-8"),
			)],
			body,
		)
			.into_response())
	} else {
		let mut directory = full_path.as_path();

		if !is_index {
			directory = directory.parent().expect("should have a parent");
		}

		let results = glob(format!("{directory}**/*.html").as_str()).ok();
		let mut list = Vec::new();

		if let Some(paths) = results {
			for path in paths.flatten() {
				let path = Utf8Path::from_path(&path).expect("should be a utf path");

				list.push(path.to_string());

				if list.len() == 100 {
					break;
				}
			}
		}

		let html = View { list }.render()?;

		Ok((
			StatusCode::NOT_FOUND,
			[(header::CONTENT_TYPE, "text/html; charset=utf-8")],
			html,
		)
			.into_response())
	}
}
