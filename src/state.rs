use crate::args::Args;
use clap::Parser;

#[derive(Clone)]
pub struct State {
	pub args: Args,
}

impl Default for State {
	fn default() -> Self {
		let args = Args::parse();

		Self { args }
	}
}
