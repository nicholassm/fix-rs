use std::fmt::Display;

use crate::parser::FixError;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Tag(u32);

impl Display for Tag {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str(&self.0.to_string())
	}
}

impl TryFrom<u32> for Tag {
	type Error = FixError;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		if value > 0 {
			Ok(Tag(value))
		}
		else {
			Err(FixError::NotFix(value.to_string().into_bytes()))
		}
	}
}

impl TryFrom<Vec<u8>> for Tag {
	type Error = FixError;

	fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
		let tag = String::from_utf8(value).map_err(|e| FixError::NotFix(e.into_bytes()))?;
		let tag = tag.parse::<u32>().map_err(|_| FixError::NotFix(tag.into_bytes()))?;
		Self::try_from(tag)
	}
}

impl Tag {
	pub fn number(&self) -> u32 {
		self.0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn zero_is_not_a_valid_tag() {
		assert!(Tag::try_from(0u32).is_err());
	}

	#[test]
	fn text_is_not_a_valid_tag() {
		assert_eq!(Tag::try_from(to_vec("12abc")), Err(FixError::NotFix(to_vec("12abc"))));
	}

	#[test]
	fn valid_tag() {
		assert_eq!(Tag::try_from(to_vec("42")), Ok(Tag(42)))
	}

	fn to_vec(str: &str) -> Vec<u8> {
		str.as_bytes().to_vec()
	}
}
