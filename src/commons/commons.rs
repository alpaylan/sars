use std::path::Component::Prefix;

pub type PrefixLength = Option<usize>;
pub type FastaFilePath = String;
pub type BinaryFilePath = String;
pub type OutputFilePath = String;

pub fn longest_common_prefix(s1: &str, offset1: usize, s2: &str, offset2: usize) -> usize {
	let mut s1 = &s1.as_bytes()[offset1..];
	let mut s2 = &s2.as_bytes()[offset2..];

	if s1.len() < s2.len() {
		s2 = &s2[..s1.len()];
	} else {
		s1 = &s1[..s2.len()];
	}
	for i in 0..s1.len() {
		if s1[i] != s2[i] {
			return i;
		}
	}
	return s1.len();
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