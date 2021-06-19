mod global_context;

use clap::Clap;
use rand::{thread_rng, Rng};
use global_context::GlobalContext;
use pallet_cml::generator::init_genesis;

#[macro_use]
extern crate log;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Yan Mingzhi <realraindust@gmail.com>")]
pub struct Opts {
	#[clap(short = 'h', long, default_value = "3")]
	pub end_block_height: u32,

	#[clap(short = 'o', long)]
	pub disable_output: bool,
}

fn main() -> anyhow::Result<()> {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
	let opts: Opts = Opts::parse();

	let context = simulate(opts.end_block_height);
	if !opts.disable_output {
		output(context)?;
	}
	Ok(())
}

fn simulate(end_block_height: u32) -> GlobalContext {
	let mut global_context = GlobalContext::new();
	for block_height in 0..end_block_height {
		info!("block_height {}", block_height);
		if global_context.block_height == 0 {
			global_context.genesis_seeds = Some(init_genesis());
			global_context.a_lucky_draw_box = global_context
				.genesis_seeds
				.clone()
				.unwrap()
				.a_seeds.iter()
				.map(|s| s.id)
				.collect::<Vec<u64>>()
				.to_vec();
			global_context.b_lucky_draw_box = global_context
				.genesis_seeds
				.clone()
				.unwrap()
				.a_seeds.iter()
				.map(|s| s.id)
				.collect::<Vec<u64>>()
				.to_vec();
			global_context.c_lucky_draw_box = global_context
				.genesis_seeds
				.clone()
				.unwrap()
				.a_seeds.iter()
				.map(|s| s.id)
				.collect::<Vec<u64>>()
				.to_vec();
		} else {
			user_action_check(&mut global_context);
		}
		global_context.block_height += 1;
	}
	global_context
}

fn lucky_draw(
	mut global_context: &mut GlobalContext,
	// mut a_box: Vec<u64>,
	// mut b_box: Vec<u64>, 
	// mut c_box: Vec<u64>, 
	a_coupon: u32, 
	b_coupon: u32, 
	c_coupon: u32, 
)->Vec<u64>{
	let a_box = &mut global_context.a_lucky_draw_box;
	let b_box = &mut global_context.b_lucky_draw_box;
	let c_box = &mut global_context.c_lucky_draw_box;
	let mut seed_ids = Vec::new();
	let mut rng = thread_rng();
	for _ in 0..a_coupon{
		if a_box.clone().len() > 0{
			let r: u32 = rng.gen();
			let rand_index = (r as f64 / u32::MAX as f64 * a_box.clone().len() as f64) as usize;
			info!("rand	index {:?}, len {:?}", rand_index, a_box.clone().len());
			let seed_id = a_box.swap_remove(rand_index);
			seed_ids.push(seed_id);
		}
	}
	info!("a {:?}", &seed_ids);
	for _ in 0..b_coupon{
		if b_box.clone().len() > 0{
			let r: u32 = rng.gen();
			let rand_index = (r as f64 / u32::MAX as f64 * b_box.clone().len() as f64) as usize;
			info!("rand	index {:?}, len {:?}", rand_index, b_box.clone().len());
			let seed_id = b_box.swap_remove(rand_index);
			seed_ids.push(seed_id);
		}
	}
	info!("b {:?}", &seed_ids);
	for _ in 0..c_coupon{
		if c_box.clone().len() > 0{
			let r: u32 = rng.gen();
			let rand_index = (r as f64 / u32::MAX as f64 * c_box.clone().len() as f64) as usize;
			info!("rand	index {:?}, len {:?}", rand_index, c_box.clone().len());
			let seed_id = c_box.swap_remove(rand_index);
			seed_ids.push(seed_id);
		}
	}
	
	info!("c {:?}", &seed_ids);
	seed_ids
}
/// Iterate all users, check if user wake up. If user is awake, run the user logic for all
/// possible actions, such as buy/sell token/seeds/slots. If nothing to do, just skip this user.
fn user_action_check(global_context:  &mut GlobalContext) {
	info!("user action check");
	let draw_seed_ids = lucky_draw(
		global_context,
		2,
		4,
		6,
	);
	info!("lucky draw : {:?}", draw_seed_ids);
}

/// Output function print debug information to inspection
fn output(global_context: GlobalContext) -> anyhow::Result<()> {
	let seeds = global_context
		.genesis_seeds
		.ok_or(anyhow::anyhow!("genesis seed is none"))?;

	let a_seeds = seeds.a_seeds;
	for s in a_seeds {
		info!("a seeds {:?}", &s);
	}
	let b_seeds = seeds.b_seeds;
	for s in b_seeds {
		info!("b seeds {:?}", &s);
	}
	let c_seeds = seeds.c_seeds;
	for s in c_seeds {
		info!("c seeds {:?}", &s);
	}
	info!("A lucky draw box is {:?}", global_context.a_lucky_draw_box);

	Ok(())
}
