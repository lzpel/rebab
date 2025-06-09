format:
	cargo fmt
test:
	cargo run -- -h
	cargo run -- -V
	cargo run -- --port 8080 --route "^/api/(.*)=>8081:/api/\1" --route "^(.*)=>8082:\1"
	PORT=8080 ROUTE="^/api/(.*)=>8081:/api/\1;^(.*)=>8082:\1" cargo run