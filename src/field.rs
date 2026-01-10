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
enum State {
	ParseTag, ParseValue
}

#[derive(Debug)]
pub struct FieldParser {
	state:       State,
	tag_bytes:   Vec<u8>,
	value_bytes: Vec<u8>,
}

impl FieldParser {
	pub fn new() -> Self {
		Self {
			state:       State::ParseTag,
			tag_bytes:   vec![],
			value_bytes: vec![],
		}
	}

	#[inline]
	pub fn fresh(&self) -> bool {
		self.tag_bytes.is_empty() && self.state == State::ParseTag
	}

	#[inline]
	pub fn consume(&mut self, byte: u8) -> Result<(), FixError> {
		match self.state {
			State::ParseTag => {
				if byte == TAG_DELIMITER {
					self.state = State::ParseValue;
				}
				else {
					self.tag_bytes.push(byte);
					if self.tag_bytes.len() > MAX_TAG_LENGTH || !byte.is_ascii_digit(){
						return Err(FixError::NotFix(self.tag_bytes.clone()))
					}
				}
			}
			State::ParseValue => {
				self.value_bytes.push(byte);
			}
		}
		Ok(())
	}

	pub fn tag_bytes_count(&self) -> usize {
		self.tag_bytes.len()
	}

	pub fn value_bytes_count(&self) -> usize {
		self.value_bytes.len()
	}

	pub fn bytes(self) -> Vec<u8> {
		let mut bytes = self.tag_bytes;
		if self.state == State::ParseValue { 
			bytes.push(TAG_DELIMITER);
			bytes.extend(self.value_bytes.iter());
		}
		bytes
	}

	pub fn complete(self) -> Result<Field, FixError> {
		if self.state == State::ParseTag // Never read a value.
		|| (self.state == State::ParseValue && self.value_bytes.is_empty()) { // Value is empty.
			return Err(FixError::NotFix(self.bytes()))
		}

		let tag_string = String::from_utf8(self.tag_bytes.clone()).expect("Should only be ASCII digits.");
		match tag_string.parse::<u32>() {
			Ok(tag) => {
				Ok(Field::new(tag, self.value_bytes))
			}
			Err(_) => {
				Err(FixError::NotFix(self.tag_bytes))
			}
		}
	}
}
