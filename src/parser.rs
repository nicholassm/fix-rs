use std::{io::{BufRead, Error, Write}, vec};

use crate::{dictionary::BaseDictionary, field::{Field, FieldParser}, formatter::{FixFormatter, SimpleFormatter}};

const SOH:                     u8    = b'\x01';
const BEGIN_STRING:            u8    = b'8';
const MAX_BEGIN_STRING_LENGTH: usize = 20;
const CHECK_SUM_TAG:           u32   = 10;

#[derive(Debug)]
struct Parser<F: FixFormatter> {
	field_delimiter: u8,
	parser_state:    ParserState,
	parsed_fields:   Vec<Field>,
	formatter:       F,
}

impl<F: FixFormatter> Default for Parser<F> {
	fn default() -> Self {
		Self::new(SOH)
	}
}

#[derive(Debug)]
enum ParserState {
	HeaderField {
		parser: BeginStringParser,
	},
	Field {
		parser: FieldParser,
	},
}

impl ParserState {
	fn new() -> Self {
		Self::HeaderField {
			parser: BeginStringParser::new(),
		}
	}

	fn new_field() -> Self {
		Self::Field {
			parser: FieldParser::new(),
		}
	}

	fn consume(&mut self, byte: u8) -> Result<(), FixError> {
		match self {
			ParserState::HeaderField { parser } => parser.consume(byte),
			ParserState::Field       { parser } => {
				match parser.consume(byte) {
					Ok(()) => Ok(()),
					Err(e) => {
						self.reset();
						Err(e)
					}
				}
			}
		}
	}

	fn unwind(self) -> Vec<u8> {
		match self {
			ParserState::HeaderField { parser } => parser.field_parser.bytes(),
			ParserState::Field       { parser } => parser.bytes(),
		}
	}

	fn reset(&mut self) {
		*self = ParserState::new();
	}

	fn finish_field(&mut self) -> Result<Field, FixError> {
		// Assume field is successfully completed.
		let old_state = std::mem::replace(self, ParserState::new_field());

		match old_state.complete() {
			Ok(field) => {
				if field.tag() == CHECK_SUM_TAG {
					// End of message - reset to initial state.
					self.reset();
				}
				Ok(field)
			}
			Err(e)    => {
				self.reset();
				Err(e)
			}
		}
	}

	fn complete(self) -> Result<Field, FixError> {
		match self {
			ParserState::HeaderField { parser } => parser.complete(),
			ParserState::Field       { parser } => parser.complete(),
		}
	}
}

#[derive(Debug)]
struct BeginStringParser {
	field_parser: FieldParser,
}

impl BeginStringParser {
	fn new() -> Self {
		Self {
			field_parser: FieldParser::new(),
		}
	}

	fn consume(&mut self, byte: u8) -> Result<(), FixError> {
		if self.field_parser.fresh() && byte != BEGIN_STRING {
			return Err(FixError::NotFixStart);
		}
		self.field_parser.consume(byte)?;
		
		if self.field_parser.tag_bytes_count() > 1 // Only one tag byte ('8') is allowed.
		|| self.field_parser.value_bytes_count() > MAX_BEGIN_STRING_LENGTH {
			let old_field_parser = std::mem::replace(&mut self.field_parser, FieldParser::new());
			return Err(FixError::NotFix(old_field_parser.bytes()));
		}
		Ok(())
	}

	fn complete(self) -> Result<Field, FixError> {
		self.field_parser.complete()
	}
}

pub struct Message {
	fields: Vec<Field>,
}

impl Message {
	pub fn new(fields: Vec<Field>) -> Self {
		Self { fields }
	}
}

impl<'a> IntoIterator for &'a Message {
	type IntoIter = std::slice::Iter<'a, Field>;
	type Item     = &'a Field;

	fn into_iter(self) -> std::slice::Iter<'a, Field> {
		self.fields.iter()
	}
}

#[derive(Debug)]
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
	fn new(field_delimiter: u8) -> Self {
		Self {
			field_delimiter,
			parser_state:  ParserState::new(),
			parsed_fields: Vec::new(),
			formatter:     F::default(),
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
		if byte == self.field_delimiter {
			match self.parser_state.finish_field() {
				Ok(field) => {
					let tag = field.tag();
					self.parsed_fields.push(field);

					if tag == CHECK_SUM_TAG {
						let message = Message {
							fields: self.parsed_fields.drain(..).collect(),
						};
						return Ok(Some(message));
					}
				}
				Err(e) => {
					// Unwind all parsed fields so far.
					let mut bytes = self.unwind_fields();
					bytes.extend(e.bytes());
					bytes.push(byte); // Include the delimiter that caused the error.

					return Err(FixError::NotFix(bytes));
				}
			}
		}
		else {
			match self.parser_state.consume(byte) {
				Ok(()) => {}
				Err(e) => {
					if self.parsed_fields.is_empty() {
						// No parsed fields yet - just return parser_state's error.
						return Err(e);
					}
					else {
						// Unwind all parsed fields so far.
						let mut bytes = self.unwind_fields();
						bytes.extend(e.bytes());

						return Err(FixError::NotFix(bytes));
					}
				}
			}
		}

		Ok(None)
	}
}

pub fn process(input: &mut impl BufRead, output: &mut impl Write) -> Result<(), Error> {
	Parser::<SimpleFormatter<BaseDictionary>>::default().process(input, output)
}

#[cfg(test)]
mod tests {
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
		];

		for input in inputs {
			let parser     = Parser::<SimpleFormatter<BaseDictionary>>::default();
			let mut output = Vec::new();
			parser.process(&mut &input[..], &mut output).unwrap();
			assert_eq!(to_str(&output), to_str(&input));
		}
	}

	#[test]
	fn fix_message() {
		let input      = b"8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = Parser::<SimpleFormatter<BaseDictionary>>::default();
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		 8 : BeginString  = FIX.4.2
		 9 : BodyLength   = 45
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		10 : CheckSum     = 123
		");
	}

	#[test]
	fn fix_message_with_custom_separator() {
		let input      = b"8=FIX.4.2|9=45|35=D|49=SENDER|56=TARGET|10=123|";
		let parser     = Parser::<SimpleFormatter<BaseDictionary>>::new(b'|');
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		 8 : BeginString  = FIX.4.2
		 9 : BodyLength   = 45
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		10 : CheckSum     = 123
		");
	}

	#[test]
	fn embedded_fix_message() {
		let input      = b"2026-01-10 09:08:08.232 INFO Sending FIX: 8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";
		let parser     = Parser::<SimpleFormatter<BaseDictionary>>::default();
		let mut output = Vec::new();
		parser.process(&mut &input[..], &mut output).unwrap();

		insta::assert_snapshot!(to_str(&output), @r"
		2026-01-10 09:08:8:08.8.232 INFO Sending FIX: 
		     8 : BeginString  = FIX.4.2
		     9 : BodyLength   = 45
		    35 : MsgType      = D
		    49 : SenderCompID = SENDER
		    56 : TargetCompID = TARGET
		    10 : CheckSum     = 123
		");
	}

	fn to_str(bytes: &[u8]) -> &str {
		str::from_utf8(bytes).unwrap()
	}
}
