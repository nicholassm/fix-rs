use std::{io::{BufRead, Error, Write}, vec};

use crate::{dictionary::BaseDictionary, field::{Field, FieldParser}, formatter::{FixFormatter, SimpleFormatter}};

const SOH:                     u8    = b'\x01';
const BEGIN_STRING:            u8    = b'8';
const MAX_BEGIN_STRING_LENGTH: usize = 12;
const CHECK_SUM_TAG:           u32   = 10;

#[derive(Debug)]
struct Parser<F: FixFormatter> {
	field_delimiter:     u8,
	begin_string_parser: Option<BeginStringParser>,
	field_parser:        Option<FieldParser>,
	parsed_fields:       Vec<Field>,
	formatter:           F,
}

impl<F: FixFormatter> Default for Parser<F> {
	fn default() -> Self {
		Self::new(SOH)
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
			begin_string_parser: Some(BeginStringParser::new()),
			field_parser:        None,
			parsed_fields:       Vec::new(),
			formatter:           F::default(),
		}
	}

	fn process(&mut self, input: &mut impl BufRead, output: &mut impl Write) -> Result<(), Error> {
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

		Ok(())
	}

	#[inline]
	fn consume(&mut self, byte: u8) -> Result<Option<Message>, FixError> {
		if byte == self.field_delimiter {
			if let Some(header_parser) = self.begin_string_parser.take() {
				match header_parser.complete() {
					Ok(header_field) => {
						self.parsed_fields.push(header_field);
						// Prepare to parse next field.
						self.field_parser = Some(FieldParser::new());
					}
					Err(e) => {
						// Reset to try again from the beginning.
						self.begin_string_parser = Some(BeginStringParser::new());

						let mut bytes = e.bytes();
						bytes.push(byte);
						return Err(FixError::NotFix(bytes));
					}
				}
			}
			else if let Some(field_parser) = self.field_parser.take() {
				match field_parser.complete() {
					Ok(field) => {
						let tag = field.tag();
						self.parsed_fields.push(field);

						if tag == CHECK_SUM_TAG {
							// End of message - prepare for next message.
							self.begin_string_parser = Some(BeginStringParser::new());

							let message = Message {
								fields: self.parsed_fields.drain(..).collect(),
							};
							return Ok(Some(message));
						}
						else {
							// Prepare to parse next field.
							self.field_parser = Some(FieldParser::new());
						}
					}
					Err(e) => {
						// Reset to try again from the beginning.
						self.begin_string_parser = Some(BeginStringParser::new());

						// Unwind all parsed fields so far.
						let mut bytes = vec![];

						for field in &self.parsed_fields {
							let mut field_bytes = field.bytes();
							bytes.append(&mut field_bytes);
							bytes.push(self.field_delimiter);
						}
						bytes.extend(e.bytes());

						return Err(FixError::NotFix(bytes));
					}
				}
			}
		}
		else {
			if let Some(begin_string_parser) = &mut self.begin_string_parser {
				begin_string_parser.consume(byte)?;
			}
			else if let Some(field_parser) = &mut self.field_parser {
				match field_parser.consume(byte) {
					Ok(()) => {}
					Err(e) => {
						// Reset to try again from the beginning.
						self.begin_string_parser = Some(BeginStringParser::new());

						// Unwind all parsed fields so far.
						let mut bytes = vec![];

						for field in &self.parsed_fields {
							let mut field_bytes = field.bytes();
							bytes.append(&mut field_bytes);
							bytes.push(self.field_delimiter);
						}
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
	Parser::<SimpleFormatter<BaseDictionary>>::new(b';').process(input, output)
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn not_fix() {
		let inputs: Vec<&[u8]> = vec![
			b"Hello, World!",
			b"8=Hello World! ABC",
			b"888=1238=Hello World! 8=FIX.4.2 xyzvwzabc",
			b"\x01\x01\x01\x01",
			b"8=\x01=8\x01\x01\x01",
			b"8=Hello \x01Worl\x01d! ABC",
			b"8=HelloWorld1d! ABC",
		];

		let mut parser = Parser::<SimpleFormatter<BaseDictionary>>::default();

		for input in inputs {
			let mut output = Vec::new();
			parser.process(&mut &input[..], &mut output).unwrap();
			assert_eq!(to_str(&output), to_str(&input));
		}
	}

	#[test]
	fn fix_message() {
		let input = b"8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";

		let mut parser = Parser::<SimpleFormatter<BaseDictionary>>::default();
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
		let input = b"2026-01-10 09:08:08.232 INFO Sending FIX: 8=FIX.4.2\x019=45\x0135=D\x0149=SENDER\x0156=TARGET\x0110=123\x01";

		let mut parser = Parser::<SimpleFormatter<BaseDictionary>>::default();
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
