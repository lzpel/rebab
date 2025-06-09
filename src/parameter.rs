pub use clap::Parser;
use regex::Regex;
use std::str::FromStr;

/// Simple reverse proxy CLI
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
	/// Port to listen on
	#[arg(long, env = "PORT", default_value_t = 8080)]
	port: u16,
	#[arg(long, env = "ROUTE", value_delimiter = ';')]
	route: Vec<Route>,
}

#[derive(Debug, Clone)]
pub struct Route {
	port: u16,
	path: Regex,
	path_into: String,
}

impl FromStr for Route {
	type Err = &'static str;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let parts: Vec<&str> = s.split("=>").collect();
		if parts.len() != 2 {
			return Err("Expected format: PATH_REGEX=>PORT:PATH_REPLACEMENT");
		}
		let path = Regex::new(parts[0]).map_err(|_| "cannot compile path as regex")?;
		let rest: Vec<&str> = parts[1].splitn(2, ':').collect();
		if rest.len() != 2 {
			return Err("Expected format after =>: PORT:PATH_REPLACEMENT");
		}
		let port: u16 = rest[0].parse().map_err(|_| "port cannot parse as u16")?;
		let path_into = rest[1].into();
		Ok(Self {
			port,
			path,
			path_into,
		})
	}
}
