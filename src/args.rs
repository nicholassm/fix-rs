use clap::Parser;

const SOH: char = '\x01';

/// Parse FIX messages on stdin and output on stdout.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// Separator character between fields.
	/// Only ascii values are supported.
	/// Defaults to SOH ('\x01').
	#[arg(short='s', long, default_value_t = SOH)]
	pub field_separator: char,

	/// Show all fields and not just the most relevant fields.
	#[arg(short = 'a', long, default_value_t = false)]
	pub show_all_fields: bool,

	/// Keep original ordering of tags.
	#[arg(short = 'o', long, default_value_t = false)]
	pub original_tag_ordering: bool,
}
