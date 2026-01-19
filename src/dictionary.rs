use std::collections::HashMap;

use crate::parser::tag::Tag;

pub trait Dictionary: Default {
	fn tag_name(&self, tag: Tag) -> Option<&str>;
}

/// Dictionary for FIX 5.0.
/// (Other dictionaries should be generated from FIX specs. This is future work.)
#[derive(Debug)]
pub struct BaseDictionary {
	map: HashMap<Tag, String>,
}

impl Default for BaseDictionary {
	fn default() -> Self {
		let mut map = HashMap::new();

		insert(&mut map,   1, "Account");
		insert(&mut map,   6, "AvgPx");
		insert(&mut map,   8, "BeginString");
		insert(&mut map,   9, "BodyLength");
		insert(&mut map,  10, "CheckSum");
		insert(&mut map,  11, "ClOrdID");
		insert(&mut map,  12, "Commission");
		insert(&mut map,  13, "CommType");
		insert(&mut map,  14, "CumQty");
		insert(&mut map,  15, "Currency");
		insert(&mut map,  17, "ExecID");
		insert(&mut map,  21, "HandlInst");
		insert(&mut map,  22, "IDSource");
		insert(&mut map,  30, "LastMkt");
		insert(&mut map,  31, "LastPx");
		insert(&mut map,  32, "LastQty");
		insert(&mut map,  34, "MsgSeqNum");
		insert(&mut map,  35, "MsgType");
		insert(&mut map,  37, "OrderID");
		insert(&mut map,  38, "OrderQty");
		insert(&mut map,  39, "OrdStatus");
		insert(&mut map,  40, "OrdType");
		insert(&mut map,  44, "Price");
		insert(&mut map,  48, "SecurityID");
		insert(&mut map,  49, "SenderCompID");
		insert(&mut map,  50, "SenderSubID");
		insert(&mut map,  52, "SendingTime");
		insert(&mut map,  54, "Side");
		insert(&mut map,  55, "Symbol");
		insert(&mut map,  56, "TargetCompID");
		insert(&mut map,  57, "TargetSubID");
		insert(&mut map,  59, "TimeInForce");
		insert(&mut map,  60, "TransactTime");
		insert(&mut map,  63, "SettlmntTyp");
		insert(&mut map, 115, "OnBehalfOfCompID");
		insert(&mut map, 167, "SecurityType");
		insert(&mut map, 207, "SecurityExchange");

		Self { map }
	}
}

fn insert(map: &mut HashMap<Tag, String>, tag_num: u32, tag_name: &'static str) {
	let tag = Tag::try_from(tag_num).expect(&format!("Should be a valid tag {}", tag_num));
	map.insert(tag, tag_name.to_string());
}

impl Dictionary for BaseDictionary {
	fn tag_name(&self, tag: Tag) -> Option<&str> {
		self.map.get(&tag).map(|s| s.as_str())
	}
}
