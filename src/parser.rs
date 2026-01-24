use std::{io::{BufRead, Error, Write}, vec};

use crate::{args::Args, dictionary::BaseDictionary, filter::BaseFilter, formatter::{FixFormatter, SimpleFormatter}, parser::{field::Field, message::Message, state::{CHECK_SUM, ParserState}}};

pub(crate) mod field;
pub(crate) mod state;
pub(crate) mod begin_string;
pub(crate) mod tag;
pub(crate) mod message;

pub const COMMAND_NAME: &str  = "nfix";

pub fn process(input: &mut impl BufRead, output: &mut impl Write, args: Args) -> Result<(), Error> {
	let parser = Parser::<SimpleFormatter<BaseDictionary, BaseFilter>>::new(args);
	parser.process(input, output)
}

#[derive(Debug)]
struct Parser<F: FixFormatter> {
	field_delimiter: u8,
	parser_state:    ParserState,
	parsed_fields:   Vec<Field>,
	formatter:       F,
}

#[derive(Debug, PartialEq)]
pub enum FixError {
	NotFixStart,
	NotFix(Vec<u8>),
}

impl FixError {
	pub fn bytes(self) -> Vec<u8> {
		match self {
			FixError::NotFixStart   => vec![],
			FixError::NotFix(bytes) => bytes,
		}
	}
}

impl<F: FixFormatter> Parser<F> {
	fn new(args: Args) -> Self {
		let field_delimiter = args.field_separator as u8;
		Self {
			field_delimiter,
			parser_state:  ParserState::new(),
			parsed_fields: Vec::new(),
			formatter:     F::new(&args),
		}
	}

	fn process(mut self, input: &mut impl BufRead, output: &mut impl Write) -> Result<(), Error> {
		// Read bytes and process one byte at a time as a FIX message can be split across multiple reads.
		// Note also that a single read can also contain multiple FIX messages.

		let mut buffer = input.fill_buf()?;

		while !buffer.is_empty() {
			for byte in buffer.iter() {
				match self.consume(*byte) {
					Ok(None)                     => {} // Parser consumed byte.
					Ok(Some(message))            => {
						// Write message on new line.
						output.write_all(b"\n")?;
						self.formatter.format(&message, output)?;
					}
					Err(FixError::NotFixStart)   => {
						output.write_all(&[*byte])?;
					}
					Err(FixError::NotFix(bytes)) => {
						output.write_all(&bytes)?;
					}
				}
			}

			let len = buffer.len();
			input.consume(len);
			buffer = input.fill_buf()?;
		}

		self.end_of_input(output)?;

		Ok(())
	}

	fn unwind_fields(&mut self) -> Vec<u8> {
		// Unwind parsed fields.
		let mut bytes = vec![];
		for field in self.parsed_fields.drain(..) {
			let mut field_bytes = field.bytes();
			bytes.append(&mut field_bytes);
			bytes.push(self.field_delimiter);
		}
		bytes
	}

	fn end_of_input(mut self, output: &mut impl Write) -> Result<(), Error> {
		// Unwind any parsed fields.
		let mut bytes = self.unwind_fields();
		// Unwind current parser state.
		let ongoing_bytes = self.parser_state.unwind();
		bytes.extend(ongoing_bytes);

		output.write_all(&bytes)?;

		Ok(())
	}

	#[inline]
	fn consume(&mut self, byte: u8) -> Result<Option<Message>, FixError> {
		if byte != self.field_delimiter {
			self.parse_field(byte)
		}
		else {
			self.finish_field()
		}
	}

	#[inline]
	fn parse_field(&mut self, byte: u8) -> Result<Option<Message>, FixError> {
		match self.parser_state.consume(byte) {
			Ok(()) => Ok(None),
			Err(e) => {
				if self.parsed_fields.is_empty() {
					// No parsed fields yet - just return parser_state's error.
					Err(e)
				}
				else {
					// Unwind all parsed fields so far.
					let mut bytes = self.unwind_fields();
					bytes.extend(e.bytes());

					Err(FixError::NotFix(bytes))
				}
			}
		}
	}

	fn finish_field(&mut self) -> Result<Option<Message>, FixError> {
		match self.parser_state.finish_field() {
			Ok(field) => {
				let tag = field.tag();
				self.parsed_fields.push(field);

				if tag.number() == CHECK_SUM {
					let message = Message::new(self.parsed_fields.drain(..).collect());
					Ok(Some(message))
				}
				else {
					Ok(None)
				}
			}
			Err(e) => {
				// Unwind all parsed fields so far.
				let mut bytes = self.unwind_fields();
				bytes.extend(e.bytes());
				bytes.push(self.field_delimiter); // Include the delimiter that caused the error.

				Err(FixError::NotFix(bytes))
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::filter::BaseFilter;

	use super::*;

	#[test]
	fn not_fix() {
		let inputs: Vec<&[u8]> = vec![
			b"Hello, World!",
			b"8=Hello World!",
			b"888=1238=Hello World! 8=FIX.4.2",
			b"\x01\x01\x01\x01",
			b"8=\x01=8\x01\x01\x01",
			b"8=Hello \x01Worl\x01d! ABC",
			b"8=HelloWorld1d! ABC",
			b"8=8=8=HelloWorld1d! ABC",
		];

		for input in inputs {
			let parser     = create_default_parser();
			let mut output = Vec::new();
			parser.process(&mut &input[..], &mut output).unwrap();
			assert_eq!(to_str(&output), to_str(&input));
		}
	}

	#[test]
	fn fix_message() {
		let input      = b"8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = create_default_parser();
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		");
	}

	#[test]
	fn fix_message_with_all_fields() {
		let input      = b"8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = create_parser_with_args(&[COMMAND_NAME, "-a"]);
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		   8 : BeginString  = FIX.4.2
		   9 : BodyLength   = 45
		  10 : CheckSum     = 123
		  35 : MsgType      = D
		  49 : SenderCompID = SENDER
		  56 : TargetCompID = TARGET
		  ");
	}

	#[test]
	fn fix_message_with_custom_separator() {
		let input      = b"8=FIX.4.2|9=45|35=D|49=SENDER|56=TARGET|10=123|";
		let parser     = create_parser_with_args(&[COMMAND_NAME, "-s", "|"]);
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		");
	}

	#[test]
	fn fix_message_with_custom_separator_with_all_fields() {
		let input      = b"8=FIX.4.2|9=45|35=D|49=SENDER|56=TARGET|10=123|";
		let parser     = create_parser_with_args(&[COMMAND_NAME, "-s", "|", "-a"]);
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
 		  8 : BeginString  = FIX.4.2
 		  9 : BodyLength   = 45
 		 10 : CheckSum     = 123
 		 35 : MsgType      = D
 		 49 : SenderCompID = SENDER
 		 56 : TargetCompID = TARGET
 		 ");
	}

	#[test]
	fn embedded_fix_message() {
		let input      = b"2026-01-10 09:08:08.232 INFO Sending FIX: 8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = create_default_parser();
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		2026-01-10 09:08:08.232 INFO Sending FIX: 
		    35 : MsgType      = D
		    49 : SenderCompID = SENDER
		    56 : TargetCompID = TARGET
		");
	}

	#[test]
	fn embedded_fix_message_with_all_fields() {
		let input      = b"2026-01-10 09:08:08.232 INFO Sending FIX: 8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = create_parser_with_args(&[COMMAND_NAME, "-a"]);
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		2026-01-10 09:08:08.232 INFO Sending FIX: 
		     8 : BeginString  = FIX.4.2
		     9 : BodyLength   = 45
		    10 : CheckSum     = 123
		    35 : MsgType      = D
		    49 : SenderCompID = SENDER
		    56 : TargetCompID = TARGET
		");
	}
	fn create_default_parser() -> Parser<SimpleFormatter<BaseDictionary, BaseFilter>> {
		create_parser_with_args(&[COMMAND_NAME])
	}

	fn create_parser_with_args(args: &[&str]) -> Parser<SimpleFormatter<BaseDictionary, BaseFilter>> {
		use clap::Parser;
		let args = Args::parse_from(args);
		create_parser(args)
	}

	fn create_parser(args: Args) -> Parser<SimpleFormatter<BaseDictionary, BaseFilter>> {
		Parser::<SimpleFormatter<BaseDictionary, BaseFilter>>::new(args)
	}

	fn to_str(bytes: &[u8]) -> &str {
		str::from_utf8(bytes).unwrap()
	}
}
