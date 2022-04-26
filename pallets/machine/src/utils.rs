use crate::types::*;
use sp_std::prelude::*;

pub fn encode_ra_request_content(
	tea_id: &TeaPubKey,
	target_tea_id: &TeaPubKey,
	is_pass: bool,
) -> Vec<u8> {
	let mut buf = tea_id.to_vec();
	buf.extend_from_slice(target_tea_id);
	match is_pass {
		true => buf.push(1),
		false => buf.push(0),
	}
	buf
}
