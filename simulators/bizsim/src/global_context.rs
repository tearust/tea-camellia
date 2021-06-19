use node_primitives::BlockNumber;
use pallet_cml::GenesisSeeds;

pub struct GlobalContext {
	pub block_height: BlockNumber,
	pub genesis_seeds: Option<GenesisSeeds>,
	pub a_lucky_draw_box: Vec<u64>,
	pub b_lucky_draw_box: Vec<u64>,
	pub c_lucky_draw_box: Vec<u64>,
	// pub users: Vec<user::User>,
}

impl GlobalContext {
	pub fn new() -> Self {
		GlobalContext {
			block_height: 0,
			genesis_seeds: None,
			a_lucky_draw_box: vec![],
			b_lucky_draw_box: vec![],
			c_lucky_draw_box: vec![],
			// users: Vec::new(),
		}
	}
}
