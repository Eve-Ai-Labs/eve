
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


NOW:=$(shell date +%Y%m%d%H%M%S)
OUTPUT_DIR='./web-node-static'
SOURCE_WEB_NODE_DIR='./web-node'

build-web-node:
	cd ${SOURCE_WEB_NODE_DIR}; make build
	rm -rf ${OUTPUT_DIR} || true
	cp -r ${SOURCE_WEB_NODE_DIR}/view ${OUTPUT_DIR}
	find ${OUTPUT_DIR} -type f -iregex '.*\.\(js\|html\|css\)' -print  -exec sed -i -E 's/\?b\=[0-9]{10,}/?b=${NOW}/g' {} \;
	sed -i -E s/\'webnode__web_bg.wasm\'/\'webnode__web_bg.wasm?b=${NOW}\'/g ${OUTPUT_DIR}/wasm/webnode__web.js
	cd ${OUTPUT_DIR}; npm run build
	mv ${OUTPUT_DIR}/dist/* ${OUTPUT_DIR}
	sed -i -E s/\<link[[:space:]]*rel=\"stylesheet\".*//g ${OUTPUT_DIR}/index.html
	sed -i -E s#/scripts/index.js#bundle.js#g ${OUTPUT_DIR}/index.html


