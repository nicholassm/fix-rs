//! Module with specialized parser for finding the start of a FIX message.
//! The `BeginStringParser` tries to minimize how many bytes are parsed and then unwinded if
//! it turns out not to be the beginning of a FIX message.

use crate::parser::{FixError, field::{Field, FieldParser}};

const BEGIN_STRING:            u8    = b'8';
const MAX_BEGIN_STRING_LENGTH: usize = 20;

#[derive(Debug)]
pub struct BeginStringParser {
	field_parser: FieldParser,
}

impl BeginStringParser {
	pub fn new() -> Self {
		Self {
			field_parser: FieldParser::new(),
		}
	}

	pub fn bytes(self) -> Vec<u8> {
		self.field_parser.bytes()
	}

	pub fn consume(&mut self, byte: u8) -> Result<(), FixError> {
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

	pub fn complete(self) -> Result<Field, FixError> {
		self.field_parser.complete()
	}
}
