.PHONY: wasm wasm-publish test bench clean

# Build WASM package for npm
wasm:
	wasm-pack build --release --target nodejs -- --features wasm
	@sed -i 's/"name": "pick-distinct-colors"/"name": "pick-distinct-colors-wasm"/' pkg/package.json
	@echo "Package ready at pkg/ ($(du -h pkg/*.wasm | cut -f1) wasm)"

# Build WASM targeting bundlers (webpack, vite, etc.)
wasm-bundler:
	wasm-pack build --release --target bundler -- --features wasm
	@sed -i 's/"name": "pick-distinct-colors"/"name": "pick-distinct-colors-wasm"/' pkg/package.json

# Publish WASM package to npm
wasm-publish: wasm
	cd pkg && npm publish

# Run all tests
test:
	cargo test

# Run all tests including parallel feature
test-all:
	cargo test
	cargo test --features parallel

# Run benchmarks
bench:
	cargo bench

# Build CLI in release mode
cli:
	cargo build --features cli --release

# Clean build artifacts
clean:
	cargo clean
	rm -rf pkg/
