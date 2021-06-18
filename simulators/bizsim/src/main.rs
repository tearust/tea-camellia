mod global_context;

use clap::Clap;
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
		} else {
			user_action_check(&global_context);
		}
		global_context.block_height += 1;
	}
	global_context
}

/// Iterate all users, check if user wake up. If user is awake, run the user logic for all
/// possible actions, such as buy/sell token/seeds/slots. If nothing to do, just skip this user.
fn user_action_check(_global_context: &GlobalContext) {
	info!("user action check");
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
	Ok(())
}
