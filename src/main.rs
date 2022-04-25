
mod querysa;
mod commons;
mod buildsa;

use bio::data_structures::suffix_array::suffix_array;
use commons::suffix_array::SuffixArray;
use commons::prefix_table::PrefixTable;
use crate::commons::query_mode::QueryMode;

fn main() {
	SuffixArray::query(
		"results/ecoli_prefix_table8".to_string(),
		"data/query.fa".to_string(),
		QueryMode::Simpaccel,
		"results/query_results.txt".to_string()
	).unwrap()
}
