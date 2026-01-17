use std::fmt::Display;

use crate::parser::FixError;

const TAG_DELIMITER:  u8    = b'=';
const MAX_TAG_LENGTH: usize = 6;

#[derive(Debug, Default)]
pub struct Field {
	tag: u32,
	/// Bytes for the value.
	/// (Does not include the field delimiter.)
	value_bytes: Vec<u8>,
}

impl Field {
	pub fn new(tag: u32, value_bytes: Vec<u8>) -> Self {
		Self {
			tag, value_bytes
		}
	}

	pub fn bytes(&self) -> Vec<u8> {
		let mut bytes = self.tag.to_string().into_bytes();
		bytes.push(TAG_DELIMITER);
		bytes.extend(self.value_bytes.iter());
		bytes
	}

	pub fn value_bytes(&self) -> &[u8] {
		&self.value_bytes
	}
	
	pub fn tag(&self) -> u32 {
		self.tag
	}
}

impl Display for Field {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}={}", self.tag, String::from_utf8_lossy(&self.value_bytes))
	}
}

#[derive(Debug, PartialEq)]
pub enum FieldParser {
	ParseTag {
		tag:   Vec<u8>
	},
	ParseValue{
		tag:   Vec<u8>,
		value: Vec<u8>
	},
}

impl Default for FieldParser {
	fn default() -> Self {
		FieldParser::ParseTag {
			tag: Vec::new()
		}
	}
}

impl FieldParser {
	pub fn new () -> Self {
		Self::default()
	}

	pub fn fresh(&self) -> bool {
		match self {
			FieldParser::ParseTag   { tag              } => tag.is_empty(),
			FieldParser::ParseValue { tag: _, value: _ } => false,
		}
	}

	pub fn consume(&mut self, byte: u8) -> Result<(), FixError> {
		match self {
			FieldParser::ParseTag { tag } => {
				if byte == TAG_DELIMITER {
					let value = Vec::new();
					// Move tag to next state.
					let tag   = std::mem::take(tag);
					*self     = FieldParser::ParseValue { tag, value };
				}
				else {
					tag.push(byte);
					if tag.len() > MAX_TAG_LENGTH || !byte.is_ascii_digit(){
						let tag = std::mem::take(tag); // Resets parser.
						return Err(FixError::NotFix(tag))
					}
				}
			}
			FieldParser::ParseValue { tag: _, value } => {
				value.push(byte);
			}
		}
		Ok(())
	}

	pub fn tag_bytes_count(&self) -> usize {
		match self {
			FieldParser::ParseTag   { tag           } => tag.len(),
			FieldParser::ParseValue { tag, value: _ } => tag.len(),
		}
	}

	pub fn value_bytes_count(&self) -> usize {
		match self {
			FieldParser::ParseTag   { tag: _        } => 0,
			FieldParser::ParseValue { tag: _, value } => value.len(),
		}
	}

	pub fn bytes(self) -> Vec<u8> {
		match self {
			FieldParser::ParseTag   { tag        } => tag,
			FieldParser::ParseValue { tag, value } => {
				let mut bytes = tag;
				bytes.push(TAG_DELIMITER);
				bytes.extend(value.iter());
				bytes
			}
		}
	}

	pub fn complete(self) -> Result<Field, FixError> {
		match self {
			FieldParser::ParseTag   { tag                } /* No value. */     => Err(FixError::NotFix(tag)),
			FieldParser::ParseValue { ref tag, ref value } if value.is_empty() => Err(FixError::NotFix(self.bytes())),
			FieldParser::ParseValue { tag, value }                             => {
				// Parsing below cannot fail due to the check above.
				let tag = String::from_utf8(tag).expect("Should only be ASCII digits.");
				let tag = tag.parse::<u32>().expect("Should be a valid u32.");
				Ok(Field::new(tag, value))
			}
		}
	}
}
