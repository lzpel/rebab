pub use clap::Parser;
use regex::Regex;
use std::str::FromStr;

/// Simple reverse proxy CLI
#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
	/// Port to listen on
	#[arg(long, env = "PORT", default_value_t = 8080)]
	pub port: u16,
	#[arg(long, env = "ROUTE", value_delimiter = ';')]
	pub route: Vec<Route>,
}

#[derive(Debug, Clone)]
pub struct Route {
	pub port: u16,
	pub path: Regex,
	pub path_into: String,
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

impl Route{
	pub fn prints<'a>(route: impl Iterator<Item = &'a Self> + Clone){
		let max_len = route.clone().map(|r| r.path.as_str().len()).max().unwrap_or(0);
		for (i, route) in route.enumerate() {
			println!(
				"route {:>2} {:width$} => {}:{}",
				i,
				route.path.as_str(),
				route.port,
				route.path_into,
				width = max_len
			);
		}
	}
}