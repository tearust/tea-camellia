use super::cli::Cli;
use std::cmp::min;

impl Cli {
	pub fn genesis_seed(&self) -> [u8; 32] {
		if let Some(s) = self.genesis_seed.as_ref() {
			seed_from_string(s)
		} else {
			seed_from_string("tearust")
		}
	}
}

fn seed_from_string(s: &str) -> [u8; 32] {
	let mut seed = [0; 32];
	let str_bytes = s.as_bytes();
	let len = min(seed.len(), str_bytes.len());

	for i in 0..len {
		seed[i] = str_bytes[i];
	}
	seed
}
