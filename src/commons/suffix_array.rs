#![allow(dead_code)]

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::ops::Deref;
use bio::data_structures::suffix_array::{RawSuffixArray, suffix_array};
use serde::{Serialize, Deserialize};
use crate::commons::query_mode::QueryMode;
use crate::commons::prefix_table::PrefixTable;
use crate::commons::commons::*;
use crate::commons::fasta_file::FastaFile;
use std::string::String;
use serde::de::Unexpected::Str;
use bstr::ByteSlice;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct SuffixArray<'pt> {
	pub sa: RawSuffixArray,
	pub(crate) reference: Vec<u8>,
	#[serde(borrow)]
	pub(crate) pref_tab: Option<PrefixTable<'pt>>
}

impl<'pt> SuffixArray<'pt> {
	pub fn build(prefix_length: PrefixLength, reference_path: FastaFilePath, output_path: BinaryFilePath) -> bincode::Result<()> {
		let bsa = SuffixArray::new(prefix_length, reference_path).unwrap();
		bsa.save(output_path)
	}

	pub fn new(prefix_length: PrefixLength, reference_path: FastaFilePath) -> std::io::Result<SuffixArray<'pt>> {
		let records = FastaFile(reference_path).records();
		let reference = [records.last().unwrap().unwrap().seq(), "$".as_bytes()].concat();
		let sa = suffix_array(&reference);
		let pref_tab = SuffixArray::generate_prefix_table(&sa, reference.clone(), prefix_length);
		Ok(SuffixArray {sa, reference, pref_tab })
	}

	pub fn save(&self, file_name: BinaryFilePath) -> bincode::Result<()> {
		let writer = BufWriter::new(File::create( file_name).unwrap());
		bincode::serialize_into(writer, self)
	}

	pub fn load(file_name: BinaryFilePath) -> bincode::Result<SuffixArray<'pt>> {
		// let reader = BufReader::new(File::open(file_name).unwrap());
		// let decoded : bincode::Result<SuffixArray> = bincode::deserialize_from(reader);
		// decoded
		todo!()
	}

	pub fn query(index_path: BinaryFilePath, queries_path: FastaFilePath, query_mode: QueryMode, output_path: OutputFilePath) -> std::io::Result<()> {
		let sa = SuffixArray::load(index_path).unwrap();
		let records = FastaFile(queries_path).records();
		let mut positions = vec![];
		for record in records.map(|rec| rec.unwrap()) {
			let rec =  record.to_string();
			let mut parts = rec.split_whitespace();
			let name = parts.nth(0).unwrap().to_string();
			let sequence = parts.last().unwrap();
			if sa.search(query_mode, &sequence.to_string()) {
				positions.push(String::from(name));
			}
			println!("{:?}", positions);
		}

		// println!("{:?}", positions);
		Ok(())
	}

	fn generate_prefix_table(sa: &RawSuffixArray, reference: Vec<u8>, prefix_length: PrefixLength) -> Option<PrefixTable<'pt>> {
		prefix_length.map(|k|

			PrefixTable::new(k, reference, sa)


		)}

	pub(crate) fn search(&self, query_mode: QueryMode, sequence: &String) -> bool {
		let mut interval = Some((0_usize, self.reference.len() - 1));
		if let Some(prefix_table) = &self.pref_tab {
			if sequence.len() >= prefix_table.prefix_length {
				interval = prefix_table.get_interval(sequence);
			}
		}
		if let Some(interval) = interval {
			match query_mode {
				QueryMode::Naive => {
					self.naive_search(interval, sequence)
				}
				QueryMode::Simpaccel => {
					let lcp_left = longest_common_prefix(self.reference.as_slice().to_str().unwrap(), self.sa[interval.0], sequence, 0);
					let lcp_right = longest_common_prefix(&self.reference.as_slice().to_str().unwrap(), self.sa[interval.1], sequence, 0);
					self.simple_accelerant_search((interval.0, lcp_left), (interval.1, lcp_right), sequence)
				}
			}
		} else {
			false
		}
	}

	fn naive_search(&self, interval: (usize, usize), sequence: &String) -> bool {
		// println!("{:?}", interval);
		if interval.0 > interval.1 {
			return false;
		}

		let m = (interval.0 + interval.1)/2;

		let offset = self.sa[m];
		let substr : String = self.reference.chars().skip(offset).take(sequence.len()).collect();
		// println!("{} vs {}", sequence, substr);
		match substr.cmp(sequence) {
			Ordering::Greater => { self.naive_search((interval.0, m - 1), sequence) }
			Ordering::Equal => { true }
			Ordering::Less => { self.naive_search((m + 1, interval.1), sequence) }
		}

	}

	fn simple_accelerant_search(&self, left: (usize, usize), right: (usize, usize), sequence: &String) -> bool {
		let (left_index, lcp_left_seq) = left;
		let (right_index, lcp_right_seq) = right;

		let center = (left_index + right_index)/2;
		let lcp_center_seq = longest_common_prefix(
			&self.reference.to_str().unwrap(), self.sa[center],
			sequence, 0,
		);

		if lcp_left_seq == sequence.len() || lcp_right_seq == sequence.len() || lcp_center_seq == sequence.len() {
			return true;
		}

		if left_index + 1 >= right_index {
			return false;
		}

		let lcp_center_left = longest_common_prefix(
			&self.reference.to_str().unwrap(), self.sa[left_index],
			&self.reference.to_str().unwrap(), self.sa[center]
		);

		match lcp_center_left.cmp(&lcp_left_seq) {
			Ordering::Less => {
				self.simple_accelerant_search(left, (center, lcp_center_seq), sequence)
			}
			Ordering::Greater => {
				self.simple_accelerant_search((center, lcp_center_seq), right, sequence)
			}
			Ordering::Equal => {
				if lcp_center_seq == sequence.len() {
					return true;
				}
				let seq_next = sequence.chars().nth(lcp_center_seq + 1);
				let center_next = self.reference.chars().nth(self.sa[center] + lcp_center_seq + 1);
				if seq_next < center_next {
					self.simple_accelerant_search(left, (center, lcp_center_seq), sequence)
				} else {
					self.simple_accelerant_search((center, lcp_center_seq), right, sequence)
				}
			}

		}



	}
}


#[cfg(test)]
mod tests {
	use bio::data_structures::suffix_array::suffix_array;
	use crate::commons::query_mode::QueryMode;
	use crate::PrefixTable;
	use super::SuffixArray;

	#[test]
	fn save_load_pair() {
		let lhs = SuffixArray::new(Some(1), "data/ecoli.fa".to_string()).unwrap();
		lhs.save(format!("results/ecoli_prefix_table{}", 1)).unwrap();
		let rhs = SuffixArray::load(format!("results/ecoli_prefix_table{}", 1)).unwrap();
		assert_eq!(lhs, rhs);
	}
	#[test]
	fn save_load_pair_with_prefix_table() {
		let lhs = SuffixArray::new(Some(8), "data/ecoli.fa".to_string()).unwrap();
		lhs.save("results/result.txt".to_string()).unwrap();
		let rhs = SuffixArray::load("results/result.txt".to_string()).unwrap();
		assert_eq!(lhs, rhs);
	}
	#[test]
	fn query() {
		SuffixArray::query(
			"results/result.txt".to_string(),
			"data/query.fa".to_string(),
			QueryMode::Naive,
			"results/query_results.txt".to_string()
		).unwrap()

	}
	#[test]
	fn naive_search() {
		let reference = "alpalpalp$".to_string().into_bytes();
		let sa = suffix_array(&*reference);
		let pref_tab = Some(PrefixTable::new(3, reference.clone(), &sa));
		let sa = SuffixArray {
			sa ,
			reference,
			pref_tab
		};

		assert!(sa.search(QueryMode::Naive, &"alp".to_string()));
		assert!(sa.search(QueryMode::Naive, &"lpa".to_string()));
		assert!(sa.search(QueryMode::Naive, &"alpal".to_string()));
		assert!(sa.search(QueryMode::Naive, &"alpalpalp".to_string()));
		assert!(!sa.search(QueryMode::Naive, &"lap".to_string()));
		assert!(sa.search(QueryMode::Naive, &"alp".to_string()));
		assert!(sa.search(QueryMode::Naive, &"alpa".to_string()));
		assert!(sa.search(QueryMode::Naive, &"al".to_string()));
		assert!(!sa.search(QueryMode::Naive, &"la".to_string()));
	}
	#[test]
	fn naive_search_fail_fast() {
		let seq = "apl".to_string();
		let sa = suffix_array("alpalpalp$".as_bytes());
		let pt = PrefixTable::new(3, "alpalpalp$".to_string().into_bytes(), &sa);
		assert_eq!(pt.get_interval(&seq), None);
	}
	#[test]
	fn simple_accelerant_search() {
		let reference = "alpalpalp$".to_string().into_bytes();
		let sa = suffix_array(&reference);
		// let pref_tab = Some(PrefixTable::new(10, &reference, &sa));
		let pref_tab = None;
		let sa = SuffixArray {
			sa ,
			reference,
			pref_tab
		};

		assert!(sa.search(QueryMode::Simpaccel, &"alp".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"lpa".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"alpal".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"alpalpalp".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"lap".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"alp".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"alpa".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"al".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"la".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"lal".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"lap".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"lalp".to_string()));
		assert!(!sa.search(QueryMode::Simpaccel, &"alalallaaaaalallp".to_string()));
		assert!(sa.search(QueryMode::Simpaccel, &"alpalp".to_string()));
	}
}
