## Joystream-Node - full node/validator

![ Nodes for Joystream](./validator-node-banner.svg)

joystream-node is the main server application that connects to the network, synchronizes the blockchain with other nodes and produces blocks if configured as a validator node.

To setup a full node and validator review the [advanced guide from the helpdesk](https://github.com/Joystream/helpdesk/tree/master/roles/validators).

### Pre-built Binaries

The latest pre-built binaries can be downloaded from the [releases](https://github.com/Joystream/joystream/releases) page.
Generally these will be built from master branch and relevant to the current active testnet.

### Building from source

Clone the repository and install build tools:

```bash
git clone https://github.com/Joystream/joystream.git

cd joystream/

./setup.sh
```

compile the node and runtime

```bash
cargo build --release
```

This produces the binary in `./target/release/joystream-node`

### Running Local development chain

```bash
cargo run --release -- --dev
```

If you repeatedly need to restart a new chain,
This script will build and run a fresh new local development chain purging existing chain data:

```bash
./scripts/run-dev-chain.sh
```

### Joystream public testnet

Use the `--chain` argument, and specify the path to the genesis chain.json file for that public network. The joystream public networks are found in [../testnets/](../testnets/)

```bash
cargo run --release -- --chain testnets/rome.json
```

### Tests and code quality

Running unit tests

```bash
cargo test --all
```

Running full suite of checks.. tests, formatting and linting:

```bash
yarn cargo-checks
```

Always format your rust code with `cargo fmt` before commiting:

```bash
cargo fmt --all
```

### Integration tests

```bash
./scripts/run-test-chain.sh
yarn workspace joystream-testing test
```

To run the integration tests with a different chain, you can omit step running the local development chain and set the node URL using `NODE_URL` environment variable.

### Installing a release build

If you are building a tagged release from master branch and want to install the executable to your path follow the step below.

This will install the executable `joystream-node` to your `~/.cargo/bin` folder, which you would normally have in your `$PATH` environment.

```bash
cargo install joystream-node --path node/
```

Now you can run and connect to the rome testnet:

```bash
joystream-node --chain testnets/rome.json
```
