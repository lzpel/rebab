%:
	@echo "Unknown target '$@' skipping"create:
create:	
	cargo init .
generate:
	cargo fmt
	uv run python markdown_import.py README.md
	cargo run --example generate
run: generate
	cargo run -- --input config.json
test-1:
	cargo run -- --frontend 0.0.0.0:9000 \
		--rule "port=8001,command=sleep 3" \
		--rule "port=8002,command=sleep 4" \
		--rule "port=8003,command=sleep 5"
test-2:
	cargo run -- --frontend 0.0.0.0:9000 \
		--rule "port=8001,command=git log --oneline -n 50" \
		--rule "port=8002,command=git log --stat -n 10" \
		--rule "port=8003,command=cargo tree"
search-%:
	@git grep --color -r --text -n '$*' .