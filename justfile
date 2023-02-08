build:
	cargo build

test:
	cargo test

lint:
	cargo +nightly clippy --all-targets -- -D warnings

gen: build gen-schema gen-typescript

gen-schema:
	./scripts/schema.sh

gen-typescript:
	# git checkout typescript/contracts # Clear out any old or invalid state.
	yarn --cwd ./typescript install --frozen-lockfile
	yarn --cwd ./typescript build
	yarn --cwd ./typescript codegen

optimize:
	cargo install cw-optimizoor || true
	cargo cw-optimizoor Cargo.toml

workspace-optimize:
	docker run --rm -v "$(pwd)":/code \
		--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
		--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
		--platform linux/amd64 \
		cosmwasm/workspace-optimizer:0.12.11
