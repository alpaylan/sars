
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Clone, PartialEq, Debug, Copy)]
pub enum QueryMode {
	Naive,
	Simpaccel
}
