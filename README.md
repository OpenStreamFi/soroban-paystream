# soroban-paystream

A streaming payments contract built on Stellar's Soroban smart contract platform. Enables real-time, per-second token streaming between addresses — with pause, resume, and cancel support.

## Features

- **Create streams** — lock a deposit and stream it to a recipient over time
- **Withdraw** — recipient claims earned tokens at any time
- **Pause / Resume** — sender can pause a stream; elapsed pause time is tracked and excluded from claimable amounts
- **Cancel** — sender cancels, recipient gets what they've earned, sender gets the refund
- **On-chain events** — typed events emitted on every state change
- **Per-user stream index** — query all stream IDs for any address

## Project Structure

```text
contracts/paystream/
└── src/
    ├── lib.rs        ← module registration & entry point
    ├── types.rs      ← Stream struct, StreamStatus enum
    ├── storage.rs    ← all reads/writes to contract storage
    ├── events.rs     ← typed contractevent definitions & publishers
    ├── errors.rs     ← custom error codes
    └── contract.rs   ← all public contract functions
```

## Contract Functions

| Function | Caller | Description |
|---|---|---|
| `create_stream(sender, recipient, token, deposit, start_time, end_time)` | sender | Creates a new stream and locks the deposit |
| `withdraw(stream_id)` | recipient | Claims all currently earned tokens |
| `pause_stream(stream_id)` | sender | Pauses the stream, recording the pause timestamp |
| `resume_stream(stream_id)` | sender | Resumes stream, adds pause duration to total_paused |
| `cancel_stream(stream_id)` | sender | Pays recipient their share, refunds remainder to sender |
| `get_claimable(stream_id)` | anyone | Returns claimable token amount right now |
| `get_stream(stream_id)` | anyone | Returns the full Stream struct |
| `get_streams_by_user(user)` | anyone | Returns all stream IDs for a given address |
| `get_stream_count()` | anyone | Returns total number of streams created |

## Build

```bash
stellar contract build
```

WASM output: `target/wasm32v1-none/release/soroban_paystream.wasm`

## Requirements

- Rust + `wasm32-unknown-unknown` target
- `stellar-cli` ≥ 25.x

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```
