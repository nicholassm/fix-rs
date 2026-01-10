use std::io::Write;

use crate::{dictionary::Dictionary, parser::Message};

pub trait FixFormatter: Default {
	fn format(&self, message: &Message, output: &mut impl Write) -> std::io::Result<()>;
}

#[derive(Debug, Default)]
pub struct SimpleFormatter<D: Dictionary> {
	dictionary: D,
}

impl<D: Dictionary> FixFormatter for SimpleFormatter<D> {
	fn format(&self, message: &Message, output: &mut impl Write) -> std::io::Result<()> {
		// Find max tag width for alignment of tag name.
		let width = message.into_iter()
			.flat_map(|f| self.dictionary.tag_name(f.tag()).map(|name| name.len()))
			.max()
			.unwrap_or(0);

		for field in message {
			let tag           = field.tag();
			let formatted_tag = match self.dictionary.tag_name(tag) {
				Some(name) => format!("{:>6} : {:<width$} = ", tag, name),
				None       => format!("{:>6}   {:>width$} = ", tag,   ""),
			};

			output.write_all(formatted_tag.as_bytes())?;
			output.write_all(field.value_bytes())?;
			output.write_all(b"\n")?;
		}

		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dictionary::BaseDictionary;
	use crate::field::Field;
	use crate::parser::Message;

	/// Helper for creating a field from tag and str.
	fn to_field(tag: u32, value: &str) -> Field {
		Field::new(tag, value.as_bytes().to_vec())
	}

	#[test]
	fn test_simple_formatter() {
		let formatter = SimpleFormatter::<BaseDictionary>::default();
		let message   = Message::new( 
			vec![
				to_field( 8, "FIX.4.2"),
				to_field( 9, "45"),
				to_field(35, "D"),
				to_field(49, "SENDER"),
				to_field(56, "TARGET"),
			]
		);
		let mut output = vec![];
		formatter.format(&message, &mut output).unwrap();
		let output_str = String::from_utf8(output).unwrap();
		insta::assert_snapshot!(&output_str, @r"
		 8 : BeginString  = FIX.4.2
		 9 : BodyLength   = 45
		35 : MsgType      = D
		49 : SenderCompID = SENDER
		56 : TargetCompID = TARGET
		");
	}
}

