use node_primitives::BlockNumber;
use pallet_cml::GenesisSeeds;

pub struct GlobalContext {
	pub block_height: BlockNumber,
	pub genesis_seeds: Option<GenesisSeeds>,
	// pub users: Vec<user::User>,
}

impl GlobalContext {
	pub fn new() -> Self {
		GlobalContext {
			block_height: 0,
			genesis_seeds: None,
			// users: Vec::new(),
		}
	}
}
