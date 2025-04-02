NOW:=$(shell date +%Y%m%d%H%M%S)

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

build-web-node:
	cd web-node; make build
	rm -rf web-node-static || true
	cp -r web-node/view web-node-static
	find ./web-node-static/ -type f -iregex '.*\.\(js\|html\|css\)' -print  -exec sed -i -E 's/\?b\=[0-9]{10,}/?b=${NOW}/g' {} \;
	sed -i -E s/\'webnode__web_bg.wasm\'/\'webnode__web_bg.wasm?b=${NOW}\'/g web-node-static/wasm/webnode__web.js
