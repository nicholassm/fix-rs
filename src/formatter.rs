use std::io::Write;

use crate::{args::Args, dictionary::Dictionary, filter::Filter, parser::{field::Field, message::Message}};

pub trait FixFormatter: Default {
	fn new(args: &Args) -> Self;
	fn format(&self, message: &Message, output: &mut impl Write) -> std::io::Result<()>;
}

#[derive(Debug, Default)]
pub struct SimpleFormatter<D: Dictionary, F: Filter> {
	show_all_fields:       bool,
	original_tag_ordering: bool,
	dictionary:            D,
	filter:                F,
}

impl<D: Dictionary, F: Filter> FixFormatter for SimpleFormatter<D, F> {
	fn new(args: &Args) -> Self {
		Self {
			show_all_fields:       args.show_all_fields,
			original_tag_ordering: args.original_tag_ordering,
			dictionary:            D::default(),
			filter:                F::default(),
		}
	}

	fn format(&self, message: &Message, output: &mut impl Write) -> std::io::Result<()> {
		let mut fields = self.relevant_fields(message);

		// Find max tag width for alignment of tag name.
		let width = fields.iter()
			.flat_map(|f| self.dictionary.tag_name(f.tag()).map(|name| name.len()))
			.max()
			.unwrap_or(0);

		if !self.original_tag_ordering {
			fields.sort_by(|f1, f2| f1.tag().cmp(&f2.tag()));
		}

		for field in fields {
			let tag           = field.tag();
			let formatted_tag = match self.dictionary.tag_name(tag) {
				Some(name) => format!("{:>6} : {:<width$} = ", tag.number(), name),
				None       => format!("{:>6}   {:>width$} = ", tag.number(),   ""),
			};

			output.write_all(formatted_tag.as_bytes())?;
			output.write_all(field.value_bytes())?;
			output.write_all(b"\n")?;
		}

		Ok(())
	}
}

impl<D: Dictionary, F: Filter> SimpleFormatter<D, F> {
	fn relevant_fields<'a>(&'a self, message: &'a Message) -> Vec<&'a Field> {
		message
			.into_iter()
			.filter(|f| self.show_all_fields || self.filter.relevant(f.tag()))
			.collect()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dictionary::BaseDictionary;
	use crate::filter::BaseFilter;
	use crate::parser::COMMAND_NAME;
	use crate::parser::field::Field;
	use crate::parser::tag::Tag;

	/// Helper for creating a field from tag and str.
	fn to_field(tag: u32, value: &str) -> Field {
		let tag = Tag::try_from(tag).unwrap();
		Field::new(tag, value.as_bytes().to_vec())
	}

	#[test]
	fn simple_formatter() {
		// Given:
		let formatter = SimpleFormatter::<BaseDictionary, BaseFilter>::default();
		let message   = Message::new(
			vec![
				to_field( 8, "FIX.4.2"),
				to_field( 9, "45"),
				to_field(35, "D"),
				to_field(49, "SENDER"),
				to_field(56, "TARGET"),
				to_field(10, "123")
			]
		);
		let mut output = vec![];

		// When:
		formatter.format(&message, &mut output).unwrap();

		// Then:
		let output_str = String::from_utf8(output).unwrap();
		insta::assert_snapshot!(&output_str, @r"
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		");
	}

	#[test]
	fn simple_formatter_with_all_fields() {
		// Given:
		use clap::Parser;
		let args      = Args::parse_from(&[COMMAND_NAME, "-a"]);
		let formatter = SimpleFormatter::<BaseDictionary, BaseFilter>::new(&args);
		let message   = Message::new( 
			vec![
				to_field( 8, "FIX.4.2"),
				to_field( 9, "45"),
				to_field(35, "D"),
				to_field(49, "SENDER"),
				to_field(56, "TARGET"),
				to_field(10, "123")
			]
		);
		let mut output = vec![];

		// When:
		formatter.format(&message, &mut output).unwrap();

		// Then:
		let output_str = String::from_utf8(output).unwrap();
		insta::assert_snapshot!(&output_str, @r"
		   8 : BeginString  = FIX.4.2
		   9 : BodyLength   = 45
		  10 : CheckSum     = 123
		  35 : MsgType      = D
		  49 : SenderCompID = SENDER
		  56 : TargetCompID = TARGET
		  ");
	}
}
