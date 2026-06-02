# soroban-paystream

Open-source payment streaming infrastructure for the Stellar ecosystem, built on Soroban smart contracts.

## What is this?

soroban-paystream is a Soroban smart contract that enables real-time, per-second payment streaming on Stellar. Think of it like a tap — a sender opens a stream, USDC drips continuously into the recipient's claimable balance every second, and the recipient withdraws whenever they want.

No money moves every second on-chain. The streaming is a mathematical calculation — elapsed time × rate per second — computed when anyone interacts with the contract. Only three transactions ever cost gas: creating a stream, withdrawing, and cancelling.

## Why does this exist?

Projects like payroll systems, subscription services, and grant distributions on Stellar need payment streaming infrastructure. soroban-paystream is the open-source primitive they can build on — no proprietary lock-in, fully auditable, free to fork.

## Who is this for?

- Developers building payroll or subscription dApps on Stellar
- Contributors who want to work on real Soroban infrastructure
- Anyone who wants to understand how payment streaming works on-chain

## Contract Functions

| Function | Description |
|---|---|
| `create_stream` | Lock USDC and open a stream to a recipient |
| `withdraw` | Recipient pulls out accumulated earnings |
| `pause_stream` | Sender temporarily freezes the stream |
| `resume_stream` | Sender unfreezes and continues streaming |
| `cancel_stream` | Sender stops stream, refunds unstreamed amount |
| `get_stream` | View full stream details |
| `get_claimable` | View how much recipient can withdraw right now |
| `get_streams_by_user` | View all stream IDs for an address |
| `get_stream_count` | View total streams ever created |

## Testnet Deployment

The contract is live on Stellar testnet.

**Contract Address:**
CC2SUYO3WFVMER3SKBUWM3JVI7P4OL73YD6NHWWUAN5OPY4AV46POAWE

You can inspect it on the [Stellar Expert Explorer](https://stellar.expert/explorer/testnet/contract/CC2SUYO3WFVMER3SKBUWM3JVI7P4OL73YD6NHWWUAN5OPY4AV46POAWE).

## How to Build

```bash
# Install Stellar CLI
cargo install --locked stellar-cli --features opt

# Add WASM target
rustup target add wasm32-unknown-unknown

# Clone the repo
git clone https://github.com/OpenStreamFi/soroban-paystream.git
cd soroban-paystream

# Build
stellar contract build
```

## How to Test

```bash
cargo test
```

## How to Deploy to Testnet

```bash
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/soroban_paystream.wasm \
  --network testnet \
  --source YOUR_ACCOUNT_NAME
```

## Contributing

We welcome contributions of all kinds — contract functions, tests, documentation, gas optimization, and security reviews.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for how to get started.

## License

MIT