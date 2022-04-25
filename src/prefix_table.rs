use std::borrow::Borrow;
use std::cmp::{max, min};
use std::time::Instant;
use bio::data_structures::suffix_array::{RawSuffixArray};
use serde::{Serialize, Deserialize};
use rustc_hash::FxHashMap;
use bstr::{B, ByteSlice};

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct PrefixTable {
	pub(crate) prefix_length: usize,
	pub(crate) prefix_table: FxHashMap<Vec<u8>, (usize, usize)>
}

impl PrefixTable {
	pub fn new(prefix_length: usize, mut reference: Vec<u8>, sa: &RawSuffixArray) -> PrefixTable {
		let mut prefix_table = FxHashMap::default();
		let mut current_prefix = vec![0; prefix_length];
		let mut l = 0;
		// println!("Len: {}", sa.len());
		let now = Instant::now();
		for (i, offset) in sa.iter().enumerate() {
			let mut prefix_changed = false;
			for i in 0..min(prefix_length, reference.len() - *offset) {
				if current_prefix[i] != reference[*offset + i] {
					prefix_changed = true;
					break;
				}
			}
			if prefix_changed  {
				if l != 0 {
					prefix_table.insert(current_prefix.clone(), (l - 1, i - 1));
				}
				current_prefix = vec![0; prefix_length];
				// println!("{} {} {:?}", prefix_length, offset, current_prefix);
				for i in 0..min(prefix_length, reference.len() - *offset) {
					current_prefix[i] = reference[*offset + i];
				}
				l = i + 1;
			}
		}
		println!("Time: {:?}", now.elapsed().as_secs());
		PrefixTable { prefix_length, prefix_table }
	}
	pub fn get_interval(&self, sequence: &String) -> Option<(usize, usize)> {
		let (prefix, _) = sequence.split_at(self.prefix_length);
		self.prefix_table.get(prefix.as_bytes()).map(|range| range.clone())
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Borrow;
	use bio::data_structures::suffix_array::suffix_array;
	use crate::PrefixTable;
	# [test]
	fn small_example() {
		let str = "alpalpalp$".to_string();
		let sa = suffix_array("alpalpalp$".as_bytes());
		let pt = PrefixTable::new(3, "alpalpalp$".to_string().into_bytes(), &sa);
		for (_, range) in pt.prefix_table.borrow() {
			for i in range.0..=range.1 {
				let prefix : String = str.chars().skip(sa[i]).take(3).collect();
				println!("{}", prefix);
			}
		}
		println!("{:?}", sa);
		println!("{:?}", pt);
	}
}