WEB_ROOT = $(shell realpath .)
WEB_VIEW = $(WEB_ROOT)/view
WASM_TARGET = $(WEB_ROOT)/target/wasm
WASM_DIR = $(WEB_VIEW)/wasm

build:
	wasm-pack build $(WEB_ROOT) \
		--no-typescript \
		--target web \
		--out-dir $(WASM_TARGET) \
		--reference-types \
		--out-name webnode__web \
		--no-opt
	rm -rf $(WASM_DIR)|| true
	mv $(WASM_TARGET) $(WASM_DIR)

check:
	cargo clippy -p web-node --no-deps --all-targets -- -Dwarnings
	cargo +nightly fmt -p web-node --check

fix:
	cargo fix -p web-node --allow-dirty --allow-staged 
	cargo clippy -p web-node --fix --no-deps --allow-dirty --allow-staged
	cargo +nightly fmt 	-p web-node

# install wasm-pack
install:
	cargo install wasm-pack --force
	# curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

test:
	wasm-pack test --firefox --headless
