#![allow(dead_code)]

use codec::{Decode, Encode};
use node_primitives::BlockNumber;

pub const GENESIS_SEED_A_COUNT: u64 = 10;
pub const GENESIS_SEED_B_COUNT: u64 = 30;
pub const GENESIS_SEED_C_COUNT: u64 = 60;
pub const TEAM_PERCENTAGE: u64 = 80;

pub const BLOCKS_IN_A_MONTH: u32 = 438000; //365*24*600/12
pub const BLOCKS_IN_A_DAY: u32 = 14400; //24*600
pub const BLOCKS_IN_HALF_DAY: u32 = 7200; //24*600

///The base value of life span of a Camellia. The actually value will be a random deviation on this base value
pub const BASE_LIFESPAN_A: BlockNumber = 24 * BLOCKS_IN_A_MONTH;
pub const BASE_LIFESPAN_B: BlockNumber = 24 * BLOCKS_IN_A_MONTH;
pub const BASE_LIFESPAN_C: BlockNumber = 12 * BLOCKS_IN_A_MONTH;

///the random deviation in percentage. for the lifespan of an Camellia
pub const DEVIATION: u8 = 10; //This means a deviation between +5% and -5% for an individual camellia lifespan

///The performance unit. Performance is the indicator of a Camellia's outcome rate. It only has relative meaning
pub type Performance = u32;

///The base value of performance for different seeds type
pub const BASE_PERFORMANCE_A: Performance = 40000;
pub const BASE_PERFORMANCE_B: Performance = 20000;
pub const BASE_PERFORMANCE_C: Performance = 10000;

///the random deviation in percentage. for the lifespan of an Camellia
pub const PERFORMANCE_DEVIATION: u8 = 10; //This means a deviation between +5% and -5% for an individual camellia performance

#[derive(Encode, Decode, Clone, Debug)]
pub struct DefrostSchedule {
	cliff: BlockNumber,
	interval: BlockNumber,
	cliff_percentage: u8,
	percentage: u8,
}

/// For investors, they have a different defrost schedule. there is 10 % defrost seeds at the genesis block. after that , defrost 5% every month
///
pub const INVESTOR_S_DEFROST_SCHEDULE: DefrostSchedule = DefrostSchedule {
	cliff: 0,
	cliff_percentage: 10,
	interval: BLOCKS_IN_A_MONTH, // average how many blocks in a month
	percentage: 5,
};

/// Compare with investors, team has a different defrost schedule. there is no defrost seeds in the first two months. starting from the third month, defrost 5% every month
///
pub const TEAM_DEFROST_SCHEDULE: DefrostSchedule = DefrostSchedule {
	cliff: 2 * BLOCKS_IN_A_MONTH,
	interval: BLOCKS_IN_A_MONTH,
	cliff_percentage: 0,
	percentage: 5,
};

///when set defrost time, we need to add some random deviation so that those seeds wont defrost at the same time
/// this deviation is set up to three days earlier or later from standard time. The seeds wont be distributed evenly over those period of time
///
pub const DEFROST_RANDOM_BLOCK_RANGE: u32 = 3 * 24 * 600; //three days earlier or later
