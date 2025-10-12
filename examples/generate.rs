use rebab::config;
use schemars;
use serde_json;
use std::fs;
fn main() {
	generate_schema("src/schema.json");
}

fn generate_schema(dest_path: impl AsRef<std::path::Path>) {
	let schema = schemars::schema_for!(config::Router);
	let v = serde_json::to_string_pretty(&schema).unwrap();
	fs::write(&dest_path, v).unwrap();
}
