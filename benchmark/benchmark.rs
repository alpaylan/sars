use std::fmt::format;
use std::fs::File;
use std::io::BufReader;
use std::time::{Duration, Instant};
use bio::alignment::AlignmentOperation::Ins;
use sars::fasta_file::FastaFile;
use sars::*;
use bio::data_structures::suffix_array::suffix_array;
use bio::io::bed::Records;


fn bench_suffix_array_construction() {
	let records = FastaFile("data/ecoli.fa".to_string()).records();
	let mut reference = [records.last().unwrap().unwrap().seq(), "$".as_bytes()].concat();
	loop {
		let now = Instant::now();
		suffix_array(&reference);
		println!("Length: {} || Time: {}", reference.len(), now.elapsed().as_millis());
		reference = Vec::from(reference.split_at(reference.len() / 2).1);
		if reference.len() <= 10 {
			break;
		}
	}
}

fn bench_naive_query_full_reference() {
	// for i in 1..=15 {
		let records  = FastaFile("data/query.fa".to_string()).records();
		let sa = SuffixArray::load(format!("results/ecoli_prefix_table_none").to_string()).unwrap();
		let mut total_time = 0;
		for record in records.map(|rec| rec.unwrap()).take(10) {
			let rec = record.to_string();
			let mut parts = rec.split_whitespace();
			let sequence = parts.last().unwrap();
			let now = Instant::now();
			sa.search(QueryMode::Simpaccel, &sequence.to_string());
			total_time += now.elapsed().as_millis();
		}
		println!("PrefixTable: None || Total Time: {}", Duration::from_millis(total_time as u64).as_secs());
	// }
}

fn query_different_length() {
	let records = FastaFile("data/ecoli.fa".to_string()).records();
	let mut reference = [records.last().unwrap().unwrap().seq(), "$".as_bytes()].concat();
	loop {
		let sa = suffix_array(&reference);
		let len = reference.len();
		let pref_tab = None;
		let sa = SuffixArray {sa, reference: reference.clone(), pref_tab};
		let mut total_time = 0;
		let records = FastaFile("data/query.fa".to_string()).records();
		for record in records.map(|rec| rec.unwrap()).take(10) {
			let rec = record.to_string();
			let mut parts = rec.split_whitespace();
			let sequence = parts.last().unwrap();
			let now = Instant::now();
			sa.search(QueryMode::Naive, &sequence.to_string());
			total_time += now.elapsed().as_millis();
		}
		println!("Length: {} || Time: {}", len, total_time);
		reference = Vec::from(reference.split_at(reference.len() / 2).1);
		if reference.len() <= 10 {
			break;
		}
	}
}
fn main () {
	// bench_suffix_array_construction();
	// bench_naive_query_full_reference();
	query_different_length();
}