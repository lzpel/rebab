use clap::Parser;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

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
	#[serde(alias = "prefix")]
	pub frontend_prefix: Option<String>,
	#[schemars(
		title = "Backend host name or IP address",
		description = "Examples: 10.84.1.84, google.com, etc. Defaults to 'localhost' if omitted.",
		example = "example.com"
	)]
	#[serde(alias = "host")]
	pub backend_host: Option<String>,
	#[schemars(
		title = "Backend port number",
		description = "Examples: 3000, 8080, etc. Defaults to the frontend port if omitted.",
		example = "8080"
	)]
	#[serde(alias = "port")]
	pub backend_port: Option<u16>,
	#[schemars(
		title = "Command to execute",
		description = "Optional command to execute when this rule is loaded. PORT environment variable will be set to backend_port if specified.",
		example = "npm run dev"
	)]
	pub command: Option<String>,
}

impl FromStr for Rule {
	type Err = String;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		let s = s.replace(',', "&");
		serde_urlencoded::from_str(&s).map_err(|e| e.to_string())
	}
}

#[derive(Debug, Parser, Clone)]
#[command(
	name = "rebab",
	author,
	version,
	about = "A simple reverse proxy with process management",
	long_about = "rebab is a lightweight reverse proxy that allows you to route traffic to different backends based on path prefixes. It can also manage backend processes (like 'npm run dev') automatically.",
	after_help = "EXAMPLES:
    # Use a configuration file
    $ rebab -i config.json

    # Simple routing: /api -> 3000, others -> 8080
    $ rebab --rule \"prefix=/api,port=3000\" --rule \"port=8080\"

    # Automatic process management (sets PORT=3000 for the command)
    $ rebab --rule \"prefix=/api,port=3000,command=npm run dev\"

    # Custom frontend address and mixed rules
    $ rebab --frontend 127.0.0.1:9000 --rule \"port=8080\""
)]
pub struct Args {
	#[arg(
		short = 'i',
		long = "input",
		value_name = "FILE",
		help = "Path to the configuration JSON file"
	)]
	pub input: Option<PathBuf>,

	#[arg(
		long,
		value_name = "ADDR",
		help = "Socket address to listen on (default: 0.0.0.0:8080)"
	)]
	pub frontend: Option<std::net::SocketAddr>,

	#[arg(
		long = "rule",
		value_name = "RULE",
		value_parser = Rule::from_str,
		help = "Add a routing rule. Format: 'prefix=/path,host=localhost,port=80,command=...'"
	)]
	pub rules: Vec<Rule>,
}

pub fn parse() -> Args {
	crate::config::Args::parse()
}

pub fn load(args: &Args) -> Result<Router, String> {
	let mut router = Router {
		frontend: "0.0.0.0:8080".parse().unwrap(),
		rules: vec![],
	};
	if let Some(input) = &args.input {
		let v = std::fs::read_to_string(input)
			.map_err(|v| format!("no file {}\n{v:?}", &input.to_string_lossy().to_string()))?;
		router = serde_json::from_str(&v).map_err(|v| v.to_string())?
	}

	if let Some(frontend) = args.frontend {
		router.frontend = frontend;
	}

	// CLIで指定されたルールを追加
	router.rules.extend(args.rules.clone());

	Ok(router)
}
