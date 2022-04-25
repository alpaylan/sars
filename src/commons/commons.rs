
pub type PrefixLength = Option<usize>;
pub type FastaFilePath = String;
pub type BinaryFilePath = String;
pub type OutputFilePath = String;

pub fn longest_common_prefix(s1: &str, offset1: usize, s2: &str, offset2: usize) -> usize {
	let s1 = s1.as_bytes();
	let s2 = s2.as_bytes();
	let mut prefix_length = 0;
	println!("{:?} {:?}", s1, s2);
	loop {
		let char1 = s1.get(offset1 + prefix_length);
		let char2 = s2.get(offset2 + prefix_length);
		match (char1, char2) {
			(Some(c1), Some(c2)) => {
				if c1 == c2 {
					prefix_length += 1;
				} else {
					return prefix_length;
				}
			}
			(_, _) => {
				return prefix_length;
			}
		}
	}
}


#[cfg(test)]
mod test {
	use crate::commons::commons::longest_common_prefix;
	#[test]
	fn small_test() {
		assert_eq!(longest_common_prefix(&"alpalp".to_string(), 0, &"alp".to_string(), 0), 3);
		assert_eq!(longest_common_prefix(&"lpalp".to_string(), 0, &"alp".to_string(), 0), 0);
		assert_eq!(longest_common_prefix(&"lpalp".to_string(), 2, &"alp".to_string(), 0), 3);
		assert_eq!(longest_common_prefix(&"apalp".to_string(), 3, &"alp".to_string(), 1), 2);
	}
}