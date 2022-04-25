use clap::{arg, command, Command};
use sars::QueryMode;

use sars::suffix_array::SuffixArray;

fn main() {
	let matches = command!()
		.arg(arg!([index] "the path to the binary file containing your serialized suffix array"))
		.arg(arg!([queries] "the path to an input file in FASTA format containing a set of records. You will need to care about both the name and sequence of these fasta records, as you will report the output using the name that appears for a record. Note, query sequences can span more than one line (headers will always be on one line)."))
		.arg(arg!([querymode] "this argument should be one of two strings; either naive or simpaccel. If the string is naive you should perform your queries using the naive binary search algorithm. If the string is simpaccel you should perform your queries using the “simple accelerant” algorithm we covered in class")
			     .possible_values(["naive", "simpaccel"]))
		.arg(arg!([output] "the name to use for the resulting output.")
		)
		.get_matches();

	let index_path = matches.value_of("index").unwrap().to_string();
	let queries_path = matches.value_of("queries").unwrap().to_string();
	let querymode = matches.value_of("querymode").unwrap();
	let output_path = matches.value_of("output").unwrap().to_string();
	let query_mode = match querymode {
		"naive" => QueryMode::Naive,
		"simpaccel" => QueryMode::Simpaccel,
		_ => unreachable!()
	};
	SuffixArray::query(index_path, queries_path, query_mode, output_path);
}