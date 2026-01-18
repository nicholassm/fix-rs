use crate::parser::{FixError, tag::Tag};

const TAG_DELIMITER:  u8    = b'=';
const MAX_TAG_LENGTH: usize = 6;

#[derive(Debug)]
pub struct Field {
	tag: Tag,
	/// Bytes for the value.
	/// (Does not include the field delimiter.)
	value_bytes: Vec<u8>,
}

impl Field {
	pub fn new(tag: Tag, value_bytes: Vec<u8>) -> Self {
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
	
	pub fn tag(&self) -> Tag {
		self.tag
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
			FieldParser::ParseTag   { tag } => tag.is_empty(),
			FieldParser::ParseValue { ..  } => false,
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
			FieldParser::ParseValue { value, .. } => {
				value.push(byte);
			}
		}
		Ok(())
	}

	pub fn tag_bytes_count(&self) -> usize {
		match self {
			FieldParser::ParseTag   { tag     } => tag.len(),
			FieldParser::ParseValue { tag, .. } => tag.len(),
		}
	}

	pub fn value_bytes_count(&self) -> usize {
		match self {
			FieldParser::ParseTag   { ..        } => 0,
			FieldParser::ParseValue { value, .. } => value.len(),
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
				let tag = Tag::try_from(tag)?;
				Ok(Field::new(tag, value))
			}
		}
	}
}
