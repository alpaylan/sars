use std::fs::File;
use std::io::BufReader;
use bio::io::fasta::Records;
use crate::commons::commons::FastaFilePath;

pub(crate) struct FastaFile(pub FastaFilePath);

impl FastaFile {
	pub fn records(&self) -> Records<BufReader<File>> {
		let file = File::open(&self.0).unwrap();
		let fasta_reader = bio::io::fasta::Reader::new(file);
		fasta_reader.records()
	}
}

