# rebab

[![GitHub License](https://img.shields.io/github/license/lzpel/rebab)](https://github.com/lzpel/rebab/blob/main/LICENSE)
[![Crates.io](https://img.shields.io/crates/v/rebab.svg?logo=rust)](https://crates.io/crates/rebab)

A tiny, rule-based reverse proxy written in Rust.
It listens on a single frontend socket address and forwards each incoming request to a backend determined by the **first matching rule**. Perfect for local dev and simple edge routing without bringing in a full Nginx stack.

## Features

* 🧭 **First-match routing** by path prefix
* 🧪 Minimal config (`config.json`)
* 🔁 Forwards all methods/bodies; strips hop-by-hop headers
* 🐳 Works nicely in docker-compose (service name DNS like `api:8080`)

## Installation

You can install `rebab` directly from [crates.io](https://crates.io/) using Cargo:

```bash
cargo install rebab
```

This will place the `rebab` binary into your Cargo `bin` directory (usually `~/.cargo/bin`).

If you are developing locally instead, you can also build and run it manually:

```bash
cargo build --release
cargo run -- --input config.json
```

## Usage

```bash
rebab --input config.json
```

### `config.json`

```json <!--config.json -->
{
	"frontend": "0.0.0.0:8080",
	"comment": "Routing follows the first matching rule.",
	"rules":[
		{
			"frontend_prefix": "/api/",
			"backend_port": 8000,
			"comment": "Requests whose path starts with 'api' are routed to localhost:8000."
		},
		{
			"frontend_prefix": "/example/",
			"backend_host": "example.com",
			"comment": "Requests whose path starts with 'example' are routed to example.com (the port inherits the frontend port 8080)."
		},
		{
			"backend_port": 3000,
			"comment": "All other requests are routed to localhost:3000."
		}
	]
}
```

The complete JSON Schema for config.json is available at [src/schema.json](src/schema.json).

### Config schema (informal)

* `frontend` (string): Socket address to listen on (e.g., `0.0.0.0:8080`)
* `rules[]`:

  * `frontend_prefix` (string|null): Path prefix to match. If omitted, matches everything.
  * `backend_host` (string|null): Backend host or IP. Defaults to `localhost` if omitted.
  * `backend_port` (integer|null): Backend port. Defaults to the **frontend** port if omitted.

Rules are evaluated in order; the **first** match wins.

## Examples

* `/api/users` → `localhost:8000/api/users`
* `/example/docs` → `example.com:8080/example/docs`
* `/anything-else` → `localhost:3000/anything-else`

## Notes

* Designed for HTTP/1.1; hop-by-hop headers (`Connection`, `TE`, etc.) are removed on proxying.
* In docker-compose, `backend_host` can be a service name (e.g., `"api"`).