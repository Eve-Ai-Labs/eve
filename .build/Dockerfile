FROM ubuntu:22.04 AS build
ARG WHITELIST_FORM
ARG ORCHESTRATOR_RPC
ENV PATH="${PATH}:/root/.cargo/bin"
WORKDIR /opt/app
RUN apt update && \
	apt install -y curl build-essential pkg-config libssl-dev libclang-dev clang
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | bash -s -- -y
RUN rustup update nightly && \
	rustup update stable && \
	rustup target add wasm32-unknown-unknown && \
	cargo install wasm-pack
RUN curl -fsSL https://deb.nodesource.com/setup_20.x | bash - &&\
	apt-get install -y nodejs
COPY . .
RUN npm install webpack webpack-cli sass-loader sass css-loader style-loader --save-dev
RUN make build-web-node

FROM ubuntu:22.04 AS node
WORKDIR /opt/app
RUN apt update && apt install -y openssl ca-certificates
COPY --from=build /opt/app/target/release/eve-node .

FROM nginx AS web-node
WORKDIR /opt/app
COPY --from=build /opt/app/web-node-static /usr/share/nginx/html
COPY ./.build/nginx/nginx.conf /etc/nginx/nginx.conf
COPY /.build/nginx/default.conf ./.build/nginx/json-logs.conf /etc/nginx/conf.d/
CMD ["nginx", "-g", "daemon off;"]
