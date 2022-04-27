#![allow(dead_code)]
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::ops::{Add, Deref};
use bio::data_structures::suffix_array::{RawSuffixArray, suffix_array};
use serde::{Serialize, Deserialize};
use crate::query_mode::QueryMode;
use crate::prefix_table::PrefixTable;
use crate::commons::*;
use crate::fasta_file::FastaFile;
use std::string::String;
use std::time::Instant;
use serde::de::Unexpected::Str;
use bstr::ByteSlice;

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug)]
pub struct SuffixArray {
	pub sa: RawSuffixArray,
	pub reference: Vec<u8>,
	pub pref_tab: Option<PrefixTable>
}

impl SuffixArray {
	pub fn build(prefix_length: PrefixLength, reference_path: FastaFilePath, output_path: BinaryFilePath) -> bincode::Result<()> {
		let bsa = SuffixArray::new(prefix_length, reference_path).unwrap();
		bsa.save(output_path)
	}

	pub fn new(prefix_length: PrefixLength, reference_path: FastaFilePath) -> std::io::Result<SuffixArray> {
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

	pub fn load(file_name: BinaryFilePath) -> bincode::Result<SuffixArray> {
		let reader = BufReader::new(File::open(file_name).unwrap());
		let decoded : bincode::Result<SuffixArray> = bincode::deserialize_from(reader);
		decoded
	}

	fn display_vec(v: Vec<usize>) -> String {
		let mut s = String::new();
		s = v.iter().map(|u| format!("\t{}", u)).collect();
		s
	}
	pub fn query(index_path: BinaryFilePath, queries_path: FastaFilePath, query_mode: QueryMode, output_path: OutputFilePath) -> std::io::Result<()> {
		let sa = SuffixArray::load(index_path).unwrap();
		let records = FastaFile(queries_path).records();
		let mut positions = vec![];
		for record in records.map(|rec| rec.unwrap()) {
			let rec =  record.to_string();
			let mut parts = rec.split_whitespace();
			let name : String = parts.nth(0).unwrap().chars().skip(1).collect();
			let sequence = parts.last().unwrap();
			// println!("Name: {}", name);
			// let now = Instant::now();
			let res = sa.search(query_mode, &sequence.to_string());
			positions.push(format!("{}\t{}{}\n", name, res.len(), SuffixArray::display_vec(res)))
			// println!("Passed: {}", now.elapsed().as_millis());
		}

		let mut file = OpenOptions::new()
			.create(true)
			.write(true)
			// .append(true)
			.open(output_path)
			.unwrap();
		positions.sort();
		for pos in positions{
			file.write_all(pos.as_bytes());
			// println!("{}", pos);
		}



		Ok(())
	}

	pub fn generate_prefix_table(sa: &RawSuffixArray, reference: Vec<u8>, prefix_length: PrefixLength) -> Option<PrefixTable> {
		prefix_length.map(|k|

			PrefixTable::new(k, reference, sa)


		)}

	pub fn search(&self, query_mode: QueryMode, sequence: &String) -> Vec<usize> {

		let mut interval = Some((0_usize, self.reference.len() - 1));
		if let Some(prefix_table) = &self.pref_tab {
			if sequence.len() >= prefix_table.prefix_length {
				interval = prefix_table.get_interval(sequence);
			}
		}
		let result = if let Some(interval) = interval {
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
			None
		};

		match result {
			None => { vec![] }
			Some(m) => {
				let mut res = vec![self.sa[m]];
				let mut i = m - 1;
				loop {
					let lcp_i_seq = longest_common_prefix(self.reference.as_slice().to_str().unwrap(), self.sa[i], sequence, 0);
					if lcp_i_seq == sequence.len() {
						res.push(self.sa[i]);
						if i == 0 {
							break;
						}
						i = i - 1;
					} else {
						break;
					}
				}
				i = m + 1;
				loop {
					let lcp_i_seq = longest_common_prefix(self.reference.as_slice().to_str().unwrap(), self.sa[i], sequence, 0);
					if lcp_i_seq == sequence.len() {
						res.push(self.sa[i]);
						if i == self.sa.len() - 1 {
							break;
						}
						i = i + 1;
					} else {
						break;
					}
				}
				res
			}
		}
	}

	fn naive_search(&self, interval: (usize, usize), sequence: &String) -> Option<usize> {
		// println!("{:?}", interval);
		if interval.0 > interval.1 {
			return None;
		}

		let m = (interval.0 + interval.1)/2;

		let offset = self.sa[m];
		let substr : String = self.reference.chars().skip(offset).take(sequence.len()).collect();
		// println!("{} vs {}", sequence, substr);
		match substr.cmp(sequence) {
			Ordering::Greater => { self.naive_search((interval.0, m - 1), sequence) }
			Ordering::Equal => { Some(m) }
			Ordering::Less => { self.naive_search((m + 1, interval.1), sequence) }
		}

	}

	fn simple_accelerant_search(&self, left: (usize, usize), right: (usize, usize), sequence: &String) -> Option<usize> {
		let (left_index, lcp_left_seq) = left;
		let (right_index, lcp_right_seq) = right;

		let center = (left_index + right_index)/2;
		let lcp_center_seq = lcp_left_seq + longest_common_prefix(
			&self.reference.to_str().unwrap(), self.sa[center] + lcp_left_seq,
			sequence, 0 + lcp_left_seq,
		);

		if lcp_left_seq == sequence.len() {
			return Some(left_index);
		}
		if lcp_right_seq == sequence.len() {
			return Some(right_index);
		}
		if lcp_center_seq == sequence.len() {
			return Some(center);
		}

		if left_index + 1 >= right_index {
			return None;
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
				return self.simple_accelerant_search((center, lcp_center_seq), right, sequence)
			}
			Ordering::Equal => {
				let seq_next = sequence.as_bytes()[lcp_center_seq];
				let center_next = self.reference[self.sa[center] + lcp_center_seq];
				if seq_next < center_next {
					return self.simple_accelerant_search(left, (center, lcp_center_seq), sequence)
				} else {
					return self.simple_accelerant_search((center, lcp_center_seq), right, sequence)
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

	// #[test]
	fn save_load_pair() {
		SuffixArray::build(None, "data/ecoli.fa".to_string(), format!("results/ecoli_prefix_table_none")).unwrap();
	}
	// #[test]
	fn save_load_pair_with_prefix_table() {
		for i in 1..=15 {
			SuffixArray::build(Some(i), "data/ecoli.fa".to_string(), format!("results/ecoli_prefix_table{}", i)).unwrap();
		}
	}
	// #[test]
	fn query() {
		SuffixArray::query(
			format!("results/ecoli_prefix_table{}", 8),
			"data/query.fa".to_string(),
			QueryMode::Simpaccel,
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

		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"alp".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"lpa".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"alpal".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"alpalpalp".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Naive, &"lap".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"alp".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"alpa".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Naive, &"al".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Naive, &"la".to_string()));
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

		assert_ne!(sa.search(QueryMode::Simpaccel, &"alp".to_string()), vec![]);
		assert_ne!(sa.search(QueryMode::Simpaccel, &"lpa".to_string()), vec![]);
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"alpal".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"alpalpalp".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"lap".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"alp".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"alpa".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"al".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"la".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"lal".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"lap".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"lalp".to_string()));
		// assert_eq!(vec![], sa.search(QueryMode::Simpaccel, &"alalallaaaaalallp".to_string()));
		// assert_ne!(vec![], sa.search(QueryMode::Simpaccel, &"alpalp".to_string()));
	}
}
