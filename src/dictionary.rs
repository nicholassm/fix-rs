
pub trait Dictionary: Default {
	fn tag_name(&self, tag: u32) -> Option<&'static str>;
}

#[derive(Debug, Default)]
pub struct BaseDictionary;

impl Dictionary for BaseDictionary {
	fn tag_name(&self, tag: u32) -> Option<&'static str> {
		match tag {
			8  => Some("BeginString"),
			9  => Some("BodyLength"),
			35 => Some("MsgType"),
			49 => Some("SenderCompID"),
			56 => Some("TargetCompID"),
			34 => Some("MsgSeqNum"),
			52 => Some("SendingTime"),
			10 => Some("CheckSum"),
			_  => None,
		}
	}
}
