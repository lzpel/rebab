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

### With JSON config file

```bash
rebab --input config.json
```

#### `config.json`

```json <!--config.json -->
{
	"frontend": "0.0.0.0:8080",
	"comment": "Example configuration with command execution",
	"rules": [
		{
			"frontend_prefix": "/api/",
			"backend_port": 8000,
			"command": "echo API server started",
			"comment": "This rule will execute 'echo API server started' with PORT=8000"
		},
		{
			"frontend_prefix": "/frontend/",
			"backend_port": 3000,
			"command": "echo Frontend server started",
			"comment": "This rule will execute 'echo Frontend server started' with PORT=3000"
		},
		{
			"backend_port": 9000,
			"comment": "Default route without command"
		}
	]
}
```


### CLI-only (without config file)

You can also configure `rebab` entirely via command-line arguments:

```bash
rebab --frontend 0.0.0.0:8080 \
  --rule "prefix=/api/,port=8000" \
  --rule "prefix=/example/,host=example.com" \
  --rule "port=3000"
```

**Rule format:** `key=value,key=value,...`

Available keys:
- `prefix` (or `frontend_prefix`): Path prefix to match
- `host` (or `backend_host`): Backend hostname or IP
- `port` (or `backend_port`): Backend port number

You can specify multiple `--rule` arguments; they are evaluated in order (first match wins).

### Hybrid mode

You can also combine both approaches—load a base config from JSON and override or add rules via CLI:

```bash
rebab --input config.json --frontend 127.0.0.1:9090 --rule "prefix=/extra,port=4000"
```

The complete JSON Schema for config.json is available at [src/schema.json](src/schema.json).

### Config schema (informal)

* `frontend` (string): Socket address to listen on (e.g., `0.0.0.0:8080`)
* `rules[]`:

  * `frontend_prefix` (string|null): Path prefix to match. If omitted, matches everything.
  * `backend_host` (string|null): Backend host or IP. Defaults to `localhost` if omitted.
  * `backend_port` (integer|null): Backend port. Defaults to the **frontend** port if omitted.
  * `command` (string|null): Optional command to execute when the rule is loaded. The `PORT` environment variable will be set to `backend_port` if specified.

Rules are evaluated in order; the **first** match wins.

## Process Management

When a rule includes a `command` field, `rebab` will:

1. Execute the command as a subprocess when the proxy starts
2. Set the `PORT` environment variable to the value of `backend_port` (if specified)
3. Log which command is being executed to standard output (format: `rebab: PORT=8000 npm run start:api`)
4. Monitor all subprocesses continuously
5. **Terminate all processes** if any subprocess fails or exits with a non-zero status code

This makes `rebab` ideal for development environments where you want to start multiple services (API, frontend, etc.) with a single command.

### Example with commands

```json
{
  "frontend": "0.0.0.0:8080",
  "rules": [
    {
      "frontend_prefix": "/api/",
      "backend_port": 8000,
      "command": "npm run start:api"
    },
    {
      "frontend_prefix": "/",
      "backend_port": 3000,
      "command": "npm run start:frontend"
    }
  ]
}
```

**Output:**
```
rebab: PORT=8000 npm run start:api
rebab: PORT=3000 npm run start:frontend
rebab: start listen 0.0.0.0:8080
```

In this example, both `npm run start:api` and `npm run start:frontend` will be started automatically. If either process fails, all processes will be terminated and `rebab` will exit.

## Examples

* `/api/users` → `localhost:8000/api/users`
* `/example/docs` → `example.com:8080/example/docs`
* `/anything-else` → `localhost:3000/anything-else`

## Notes

* Designed for HTTP/1.1; hop-by-hop headers (`Connection`, `TE`, etc.) are removed on proxying.
* In docker-compose, `backend_host` can be a service name (e.g., `"api"`).
* All subprocesses are automatically terminated when `rebab` exits or when any subprocess fails.
