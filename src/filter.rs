use crate::parser::tag::Tag;

pub trait Filter: Default {
	fn relevant(&self, tag: Tag) -> bool;
}

#[derive(Debug, Default)]
pub struct BaseFilter;

impl Filter for BaseFilter {
	fn relevant(&self, tag: Tag) -> bool {
		// Expand on this.
		!matches!(tag.number(), 8 | 9 | 10)
	}
}
