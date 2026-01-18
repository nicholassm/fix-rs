use crate::args::Args;
use clap::Parser;

mod parser;
mod formatter;
mod dictionary;
mod args;

fn main() -> std::io::Result<()> {
	let args = Args::parse();
	parser::process(&mut std::io::stdin().lock(), &mut std::io::stdout(), args)
}
