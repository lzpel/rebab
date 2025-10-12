use clap::Parser;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
const SCHEMA: &'static str = include_str!("schema.json");

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Router {
	#[schemars(title = "Socket address to listen on", example = "0.0.0.0:8080")]
	pub frontend: std::net::SocketAddr,
	#[schemars(
		title = "Routing rules",
		description = "Routes are evaluated in order; the first matching rule is applied."
	)]
	pub rules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Rule {
	#[schemars(
		title = "Path prefix",
		description = "Matches request paths that start with this prefix. Matches all paths if omitted.",
		example = "/api/"
	)]
	pub frontend_prefix: Option<String>,
	#[schemars(
		title = "Backend host name or IP address",
		description = "Examples: 10.84.1.84, google.com, etc. Defaults to 'localhost' if omitted.",
		example = "example.com"
	)]
	pub backend_host: Option<String>,
	#[schemars(
		title = "Backend port number",
		description = "Examples: 3000, 8080, etc. Defaults to the frontend port if omitted.",
		example = "8080"
	)]
	pub backend_port: Option<u16>,
}

#[derive(Debug, Parser, Clone)]
#[command(author, version, after_help=SCHEMA)]
pub struct Args {
	#[arg(
		short = 'i',
		long = "input",
		value_name = "FILE",
		required = true,
		help = "path for config json"
	)]
	pub input: PathBuf,
}
pub fn parse() -> Args {
	crate::config::Args::parse()
}
pub fn load(args: &Args) -> Result<Router, String> {
	let v = std::fs::read_to_string(&args.input).map_err(|v| {
		format!(
			"no file {}\n{v:?}",
			&args.input.to_string_lossy().to_string()
		)
	})?;
	serde_json::from_str(&v).map_err(|v| v.to_string())
}
