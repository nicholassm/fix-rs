use crate::parser::field::Field;

#[derive(Debug)]
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
