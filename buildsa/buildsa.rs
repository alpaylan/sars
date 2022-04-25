use clap::{arg, command, Command};

use sars::{BinaryFilePath, FastaFilePath, SuffixArray};

fn main() {
	let matches = command!()
		.arg(arg!([reference] "the path to a FASTA format file containing the reference of which you will build the suffix array"))
		.arg(arg!([output] "the program will write a single binary output file to a file with this name, that contains a serialized version of the input string and the suffix array"))
		.arg(arg!(-p --preftab <k> "if the option --preftab is passed to the buildsa executable (with the parameter k), then a prefix table will be built atop the suffix array, capable of jumping to the suffix array interval corresponding to any prefix of length k")
			.required(false)
			.takes_value(true)
		)
		.get_matches();

	let prefix_table = matches.value_of("preftab").map(|num| num.parse::<usize>().unwrap());
	let reference = matches.value_of("reference").unwrap();
	let output = matches.value_of("output").unwrap();

	SuffixArray::build(prefix_table, reference.to_string(), output.to_string());

}