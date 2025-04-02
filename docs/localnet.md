# Local testing

Create config for orchestrator and 2 nodes and run its:
```bash
cargo run -p eve-node -- init --quic /ip4/127.0.0.1/udp/9999/quic-v1 --webrtc /ip4/127.0.0.1/udp/9998/webrtc-direct -n /ip4/127.0.0.1/udp/9900/quic-v1 -n /ip4/127.0.0.1/udp/9901/quic-v1

cargo run -p eve-node -- run eve/orch
cargo run -p eve-node -- run eve/node_0
cargo run -p eve-node -- run eve/node_1
```

Create another node (`node_2`, `node_3`) with orchestrator pubkey (`base.pub_key`) from config `eve/orch/config.yaml`:
```bash
cargo run -p eve-node -- cfg-node --orch http://127.0.0.1:1733 -p /ip4/127.0.0.1/udp/9902/quic-v1 eve/node_2

cargo run -p eve-node -- cfg-node --orch http://127.0.0.1:1733 -p /ip4/127.0.0.1/udp/9903/quic-v1 eve/node_3
```

Create `localnet` account if not already exist:
```bash
cargo run -p eve -- account create localnet --rpc http://127.0.0.1:1733
```

Add node to orchestrator whitelist with node pubkey (`base.pub_key`) from `eve/node_1/config.yaml`:
```bash
cargo run -p eve -- node add -p <NODE_PUBKEY> -a /ip4/127.0.0.1/udp/9902/quic-v1 -j <JWT_TOKEN> localnet

cargo run -p eve -- node add -p <NODE_PUBKEY> -a /ip4/127.0.0.1/udp/9903/quic-v1 -j <JWT_TOKEN> localnet
```

Run node (node_2)

```bash
cargo run -p eve-node -- run eve/node_2
cargo run -p eve-node -- run eve/node_3
```

Stop orchestrator (Ctrl-C) and run again:

```bash
cargo run -p eve-node -- run eve/orch

cargo run -p eve -- node list localnet
```

Nodes should succesfull connected

Stop `node_1`, `node_2` and run again.

```bash
cargo run -p eve-node -- run eve/node_1
cargo run -p eve-node -- run eve/node_2

cargo run -p eve -- node list localnet
```

Nodes should succesfull connected.
