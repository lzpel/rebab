pub mod parameter;
use clap::Parser;
fn main() {
	let args = parameter::Args::parse();
	println!("{:?}", args);
}
