%:
	@echo "Unknown target '$@' skipping"create:
create:	
	cargo init .
generate:
	cargo fmt
	python markdown_import.py README.md
	cargo run --example generate
run: generate
	cargo run -- --input config.json
