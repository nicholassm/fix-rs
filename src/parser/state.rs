use crate::parser::{FixError, begin_string::BeginStringParser, field::{Field, FieldParser}};

pub const CHECK_SUM: u32 = 10;

#[derive(Debug)]
pub enum ParserState {
	HeaderField {
		parser: BeginStringParser,
	},
	Field {
		parser: FieldParser,
	},
}

impl ParserState {
	pub fn new() -> Self {
		Self::HeaderField {
			parser: BeginStringParser::new(),
		}
	}

	pub fn new_field() -> Self {
		Self::Field {
			parser: FieldParser::new(),
		}
	}

	pub fn consume(&mut self, byte: u8) -> Result<(), FixError> {
		match self {
			ParserState::HeaderField { parser } => parser.consume(byte),
			ParserState::Field       { parser } => parser.consume(byte),
		}
	}

	pub fn unwind(self) -> Vec<u8> {
		match self {
			ParserState::HeaderField { parser } => parser.bytes(),
			ParserState::Field       { parser } => parser.bytes(),
		}
	}

	pub fn reset(&mut self) {
		*self = ParserState::new();
	}

	pub fn finish_field(&mut self) -> Result<Field, FixError> {
		// Assume field is successfully completed.
		let old_state = std::mem::replace(self, ParserState::new_field());

		match old_state.complete() {
			Ok(field) => {
				if field.tag().number() == CHECK_SUM {
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
