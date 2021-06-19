mod mock;
mod simulator;

use clap::Clap;

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Yan Mingzhi <realraindust@gmail.com>")]
pub struct Opts {
	#[clap(short = 'h', long, default_value = "3")]
	pub end_block_height: u64,
}

fn main() {
	env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
	let opts: Opts = Opts::parse();

	simulator::start_simulation(opts.end_block_height);
}
