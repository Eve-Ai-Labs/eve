
check:
	cargo clippy --no-deps --all-targets -- -Dwarnings
	cargo +nightly fmt --check
	cd web-node; make build
	cargo test

fix:
	cargo fix  --allow-dirty --allow-staged 
	cargo clippy --fix --no-deps --allow-dirty --allow-staged
	cargo +nightly fmt 	

build: build-web-node
	cargo build

OUTPUT_DIR='./web-node-static'
SOURCE_WEB_NODE_DIR='./web-node'

build-web-node:
	cd ${SOURCE_WEB_NODE_DIR}; make build-static
	rm -rf ${OUTPUT_DIR} || true
	mv ${SOURCE_WEB_NODE_DIR}/view/dist ${OUTPUT_DIR}


