# EVE

### Getting started

To work with the test mode, make sure that `ollama` is running. To run, execute the command below:

```bash
ollama serve
```

Now you can start a test instance of the node, use the following cargo command:

```bash
cargo run --bin eve-node -- test-run
```

This command will:

1. Build the project
2. Run the binary named `eve-node`
3. Pass the `test-run` argument to initiate test mode

### To generate configs for 2 nodes and orchestrator

```bash
cargo run -p eve-node -- init -n /ip4/127.0.0.1/udp/9901/quic-v1 -n /ip4/127.0.0.1/udp/9902/quic-v1 --quic /ip4/127.0.0.1/udp/9903/quic-v1 --webrtc /ip4/127.0.0.1/udp/9904/webrtc-direct --jwt c9ec179a3fbc9f22cb2370fef360604235f412ac953d9bb2f5616deb7d98bc74
```

then to run nodes:

```bash
cargo run -p eve-node -- run eve/orch
cargo run -p eve-node -- run eve/node_0
cargo run -p eve-node -- run eve/node_1
```

### Generating a node configuration

Now, please generate the configuration using the following command:

```bash
eve-node cfg-node --orch-quic <ORCH_QUIC> --orch-key <ORCH_KEY> [PATH]
```

#### Required Parameters

1. `--orch-quic <ORCH_QUIC>`: The orchestrator's QUIC multiaddress.
Example: `/ip4/192.168.1.1/udp/9090/quic-v1`

2. `--orch-key <ORCH_KEY>`: The orchestrator's public key.

#### Optional Parameters

1. `[PATH]`
2. `--ollama-url <OLLAMA_URL>`
3. `--ai-model <AI_MODEL>`
4. `--p2p-address <P2P_ADDRESS>`

#### Usage example

To generate a configuration, run:

```bash
eve-node cfg-node --orch-quic /ip4/192.168.1.1/udp/9090/quic-v1 --orch-key <YOUR_ORCH_KEY>
```

The node configuration will be initialized and ready for further setup.

## Working with Accounts

### Creating accounts

To create a new profile, use the following command (aliases: `accounts`, `profile`, `profiles`):

```bash
eve account create --rpc <RPC> [NAME]
```

This command will:

1. Create a profile and store it in `$HOME/.eve/config.yaml`.
2. Assign the provided name `[NAME]` to the profile. If no name is specified, the default profile will be used.
3. Generate a new private key unless one is specified manually.

#### Arguments

1. `<NAME>` - Name of the profile.

#### Options

1. `-r, --rpc <RPC>` - URL (required).
2. `-k, --key <PRIVATE_KEY>`
3. `-y, --yes`

### Listing accounts

To view all stored profiles, use:

```bash
eve account list
```

This will display profiles stored in `$HOME/.eve/config.yaml`.

### Deleting accounts

To delete a profile, run:

```bash
eve account delete <NAME>
```

This will remove the specified profile from `$HOME/.eve/config.yaml`.

#### Arguments

1. `<NAME>`

#### Options

1. `-y, --yes`

### Airdrop

To issue (airdrop) funds to the account (default: `default`, 10 requests per hour):

```bash
eve account airdrop <NAME>
```

To check the balance, run:

```bash
eve account balance <NAME>
```

## Sending Queries

To send a request to DeepSeek and receive a response, use the following command (aliases: `send`, `question`, `run`, `request`):

```bash
eve ask [OPTIONS] <QUERY>
```

This command will:

1. Send the provided `<QUERY>` to DeepSeek.
2. Return a response based on the given input.
3. Open an interactive preview mode displaying three response variations from DeepSeek.

#### Navigation in preview mode

1. Use Up/Down arrow keys to switch between responses.
2. Press ESC to exit preview mode.

#### Arguments

1. `<QUERY>` – The question or request to send to DeepSeek.

#### Options

1. `-p, --profile <PROFILE>`
2. `-w, --waiting-time <WAITING_TIME>`
3. `-s, --session <SESSION>`
4. `-c, --clean`
5. `-j, --json`
6. `-y, --yes`

## Requesting Previous Responses

To retrieve a previously generated response from DeepSeek, use the following command (aliases: `answers`, `result`, `results`):

```bash
eve answer [OPTIONS] [SESSION]
```

This command will:

1. Fetch a stored response from a previous `eve ask` request.
2. Use the provided query ID to retrieve the exact response.
3. Allow filtering by session and profile to access specific stored answers.

#### Usage example

1. Send a question and receive a response:

```bash
eve ask "What is blockchain?"
```

After execution, a query ID will be generated.

2. Retrieve the response later using the query ID:

```bash
eve answer -q <QUERY_ID>
```

This will return the exact response associated with the given query ID.

3. Retrieve a response from a specific session:

```bash
eve answer sessionName
```

This fetches the latest saved response from `sessionName`.

#### Arguments

1. `[SESSION]` – Session name (chat name) used to store the history (default: `default`).

#### Options

1. `-q, --query-id <QUERY_ID>` - Retrieve a specific response using the query ID (generated after `eve ask`).
2. `-p, --profile <PROFILE>`
3. `-j, --json`

## Working with Nodes

The `eve node` command allows you to manage nodes by listing, adding, or deleting them.

### Listing nodes

```bash
eve node list [RPC]
```

#### Description

Displays all available nodes.

#### Arguments

1. `[RPC]`

### Adding nodes

After generating the node configuration, the next step is to add the node to the network. Run the command:

```bash
eve node add [OPTIONS] --jwt <JWT> --public-key <PUBLIC_KEY> [RPC]
```

#### Description

Adds a new node to the network.

#### Usage example

To add a node with a specific JWT secret and public key:

```bash
eve node add --jwt 4f9c87f682b1402fad29c45ac8a04bd4832ce17f7b52a2e6bff4da1efc4f6355 --public-key 16e7e3b8c3b6293b50d7c1b2a99e17e5d4b9a6f5c92c9b497e7bb1a3345a2c61
```

If the orchestrator requires a specific RPC address, provide it at the end:

```bash
eve node add --jwt 4f9c87f682b1402fad29c45ac8a04bd4832ce17f7b52a2e6bff4da1efc4f6355 --public-key 16e7e3b8c3b6293b50d7c1b2a99e17e5d4b9a6f5c92c9b497e7bb1a3345a2c61 orchestrator-address 
```

If a node address is required:

```bash
eve node add --jwt 4f9c87f682b1402fad29c45ac8a04bd4832ce17f7b52a2e6bff4da1efc4f6355 --public-key 16e7e3b8c3b6293b50d7c1b2a99e17e5d4b9a6f5c92c9b497e7bb1a3345a2c61 --address /ip4/127.0.0.1/tcp/10000
```

After adding the node, it will be registered in the network and ready for use.

#### Options

1. `-j, --jwt <JWT>` – JWT secret (example: `4f9c87f682b1402fad29c45ac8a04bd4832ce17f7b52a2e6bff4da1efc4f6355`).
2. `-p, --public-key <PUBLIC_KEY>` – Node's peer public key (retrieved from the node configuration).
3. `-a, --address <ADDRESS>`
4. `-y, --yes`

Now you can run the added node:

```bash
eve-node run [PATH]
```

#### Arguments

1. `[PATH]` - The directory for the node [default: `./.eve`]

### Deleting nodes

```bash
eve node delete [OPTIONS] --jwt <JWT> --public-key <PUBLIC_KEY> [RPC]
```

#### Description

Removes a node from the network.

#### Arguments

1. `[RPC]`

#### Options

1. `-j, --jwt <JWT>`
2. `-p, --public-key <PUBLIC_KEY>`
3. `-y, --yes`

## AI Web Node

AI Web Node - a web interface for working with AI models. Allows to select, run, and manage processes, as well as monitor balance and system parameters.

### Launching the process

A drop-down list with model options is available at the top of the interface. You can select one of the available models, after which it becomes possible to launch it.
For instance, it can be `DeepSeek-R1-Distill-Qwen-1.5B-Q2`. Run it by clicking the "Start" button and wait for the process to complete. You cannot use the settings until the process is completed.

### Balance and processes

The balance is displayed in the top panel of the interface and is updated automatically. You can update it manually by clicking on the balance itself.
Actions will be displayed in the "Process" section, and if errors or changes occur, the user may receive notifications in this section.

```
## License

This project is licensed under the GNU General Public License v2.0 - see the [LICENSE](LICENSE) file for details.
```
