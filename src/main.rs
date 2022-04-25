
mod querysa;
mod commons;
mod buildsa;

use commons::suffix_array::SuffixArray;
use commons::prefix_table::PrefixTable;
use crate::commons::query_mode::QueryMode;

fn main() {
	let lhs = SuffixArray::new(Some(1), "data/ecoli.fa".to_string()).unwrap();
	lhs.save(format!("results/ecoli_prefix_table{}", 1)).unwrap();
	let rhs = SuffixArray::load(format!("results/ecoli_prefix_table{}", 1)).unwrap();
	assert_eq!(lhs, rhs);

}
