BUILD_ENV := rust

lint:
	@cargo fmt
	@cargo clippy --all-targets --all-features

fix:
	@cargo clippy --fix --workspace --tests

test:
	@cargo test --workspace --all-features -- --nocapture
