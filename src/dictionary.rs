use crate::parser::tag::Tag;

pub trait Dictionary: Default {
	fn tag_name(&self, tag: Tag) -> Option<&'static str>;
}

#[derive(Debug, Default)]
pub struct BaseDictionary;

impl Dictionary for BaseDictionary {
	fn tag_name(&self, tag: Tag) -> Option<&'static str> {
		match tag.number() {
			8  => Some("BeginString"),
			9  => Some("BodyLength"),
			10 => Some("CheckSum"),
			34 => Some("MsgSeqNum"),
			35 => Some("MsgType"),
			49 => Some("SenderCompID"),
			52 => Some("SendingTime"),
			56 => Some("TargetCompID"),
			_  => None,
		}
	}
}
