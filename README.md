# soroban-paystream

[![Build](https://img.shields.io/github/actions/workflow/status/OpenStreamFi/soroban-paystream/ci.yml?branch=main&label=build)](https://github.com/OpenStreamFi/soroban-paystream/actions)
[![Tests](https://img.shields.io/badge/tests-22%20passing-brightgreen)](https://github.com/OpenStreamFi/soroban-paystream)
[![License](https://img.shields.io/badge/License-MIT-green)](./LICENSE)
[![Stellar](https://img.shields.io/badge/Stellar-Soroban-7D00FF?logo=stellar)](https://stellar.org)
[![Rust](https://img.shields.io/badge/Rust-2021-000000?logo=rust)](https://www.rust-lang.org)

**Open-source payment streaming infrastructure for the Stellar ecosystem.**

Send XLM by the second — no middlemen, no payroll software, no closed APIs. Just a smart contract anyone can build on.


[Overview](#overview) • [Architecture](#architecture) • [Contract Functions](#contract-functions) • [Getting Started](#getting-started) • [Testing](#testing) • [Roadmap](#roadmap) • [Contributing](#contributing)

---

## Overview

soroban-paystream is a Soroban smart contract that enables **real-time, per-second XLM streaming** between any two Stellar addresses. A sender locks a deposit, and from that moment the recipient's claimable balance grows every second — like a tap that's always open.

### What This Brings to Stellar

Stellar is one of the cheapest and fastest blockchains for payments. But until now, there was no open-source primitive for continuous, programmable payment flows. Projects building payroll systems, subscription services, creator monetisation tools, or grant distribution on Stellar had to build streaming logic themselves — or use closed, proprietary solutions.

soroban-paystream fills that gap. It is the **open-source streaming layer** for the Stellar ecosystem — free to use, free to fork, fully auditable.

### Why It Works This Way

No money moves every second on-chain. That would be expensive and slow on any blockchain. Instead, the contract tracks elapsed time and computes what the recipient has earned on demand:
```
elapsed     = current_time - start_time - total_paused_time
earned      = elapsed × rate_per_second
claimable   = earned - already_claimed
```

Only **three transactions** ever cost gas for the entire lifetime of a stream — creating it, withdrawing from it, and cancelling it. Everything else is pure math.

### Who This Is For

- Teams building **payroll dApps** on Stellar who need streaming salary infrastructure
- Projects distributing **grants or vesting schedules** to contributors
- Developers building **subscription or pay-per-use** products on Soroban
- Contributors who want to work on **real, deployed Soroban infrastructure**
- Anyone who wants to understand how payment streaming works on-chain

---

## Architecture

### System Overview

```text
┌─────────────────────────────────────────────────────────────┐
│                   paystream-app (Frontend)                   │
│   ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐   │
│   │ Create      │  │  Stream     │  │  Stream         │   │
│   │ Stream Form │  │  Dashboard  │  │  History Table  │   │
│   └─────────────┘  └─────────────┘  └─────────────────┘   │
└─────────────────────────────────────────────────────────────┘
│
▼
┌─────────────────────────────────────────────────────────────┐
│                     Integration Layer                        │
│      ┌──────────────────┐      ┌────────────────────┐      │
│      │   Stellar SDK    │      │  Freighter Wallet  │      │
│      └──────────────────┘      └────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
│
▼
┌─────────────────────────────────────────────────────────────┐
│              Stellar Network — Soroban Smart Contract        │
│                                                             │
│   create_stream()    withdraw()      pause_stream()         │
│   resume_stream()    cancel_stream() get_claimable()        │
│   get_stream()       get_streams_by_user()                  │
│   get_stream_count()                                        │
└─────────────────────────────────────────────────────────────┘
```

### Stream Lifecycle

```text
[create_stream]
│
▼
┌─────────┐
│  Active │ ◄────────────────────┐
└────┬────┘                      │
│                           │
┌────▼──────┐            [resume_stream]
│  Paused   │────────────────────┘
└────┬──────┘
│
[cancel_stream / time ends]
│
▼
┌───────────┐    ┌───────────┐
│ Cancelled │    │ Completed │
└───────────┘    └───────────┘
```

### Project Structure

```text
soroban-paystream/
└── src/
    ├── lib.rs          ← module registration, entry point
    ├── types.rs        ← Stream struct, StreamStatus enum
    ├── storage.rs      ← all reads/writes to contract storage
    ├── events.rs       ← typed contract events
    ├── errors.rs       ← custom error codes
    ├── contract.rs     ← all 9 public contract functions
    └── tests.rs        ← 22 unit tests across all functions
```

---

## Tech Stack

| Layer | Technology | Purpose |
|---|---|---|
| **Smart Contract** | Rust 2021 | Contract language |
| **Contract Framework** | Soroban SDK v25 | Stellar smart contract platform |
| **Frontend** | React + TypeScript | Stream management dashboard |
| **Wallet** | Freighter | Stellar wallet connection |
| **Build Tool** | Vite | Frontend bundler |
| **Styling** | Tailwind CSS | UI styling |

---

## Contract Functions

### Core Functions

| Function | Caller | Description |
|---|---|---|
| `create_stream(sender, recipient, token, deposit, start_time, end_time)` | Sender | Locks XLM and opens a stream. Returns a stream ID. |
| `withdraw(stream_id)` | Recipient | Claims all currently earned tokens. |
| `pause_stream(stream_id)` | Sender | Freezes the stream. Elapsed pause time is tracked and excluded from earnings. |
| `resume_stream(stream_id)` | Sender | Unfreezes the stream and continues from where it stopped. |
| `cancel_stream(stream_id)` | Sender | Pays recipient their earned share, refunds remainder to sender. |

### View Functions

| Function | Caller | Returns |
|---|---|---|
| `get_claimable(stream_id)` | Anyone | How much the recipient can withdraw right now |
| `get_stream(stream_id)` | Anyone | The full Stream struct |
| `get_streams_by_user(address)` | Anyone | All stream IDs for a given address |
| `get_stream_count()` | Anyone | Total number of streams ever created |

### Error Codes

| Code | Error | Description |
|---|---|---|
| 1 | `StreamNotFound` | No stream exists with that ID |
| 2 | `NotAuthorized` | Caller is not allowed to perform this action |
| 3 | `StreamNotActive` | Stream is cancelled or completed |
| 4 | `StreamAlreadyPaused` | Cannot pause an already paused stream |
| 5 | `StreamNotPaused` | Cannot resume a stream that is not paused |
| 6 | `NothingToClaim` | Recipient has no earned balance to withdraw |
| 7 | `InvalidAmount` | Deposit is zero or negative |
| 8 | `InvalidTimeRange` | end_time is not after start_time |
| 9 | `StreamAlreadyEnded` | end_time is already in the past |

---

## Testnet Deployment

The contract is live on Stellar testnet.

**Contract Address:**
```
CC2SUYO3WFVMER3SKBUWM3JVI7P4OL73YD6NHWWUAN5OPY4AV46POAWE
``` 
Inspect it on the [Stellar Expert Explorer →](https://stellar.expert/explorer/testnet/contract/CC2SUYO3WFVMER3SKBUWM3JVI7P4OL73YD6NHWWUAN5OPY4AV46POAWE)

---

## Getting Started

### Prerequisites

- Rust + `wasm32-unknown-unknown` target
- Stellar CLI ≥ 25.x

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

### Clone and Build

```bash
git clone https://github.com/OpenStreamFi/soroban-paystream.git
cd soroban-paystream
stellar contract build
```

### Run Tests

```bash
cargo test
```

### Deploy to Testnet

```bash
# Generate and fund a deployer account
stellar keys generate deployer --network testnet
stellar keys fund deployer --network testnet

# Deploy
stellar contract deploy \
  --wasm target/wasm32v1-none/release/soroban_paystream.wasm \
  --source deployer \
  --network testnet
```

---

## Testing

The contract has **22 unit tests** covering every function and edge case.

| Test Group | Tests | What is covered |
|---|---|---|
| `create_stream` | 4 | Success, invalid amount, invalid time range, already ended |
| `get_claimable` | 3 | Before start, mid-stream, after end |
| `withdraw` | 4 | Success, nothing to claim, marks completed, no double claim |
| `pause / resume` | 4 | Pause success, resume success, already paused, pause time excluded |
| `cancel` | 4 | Active stream, paused stream, already cancelled, withdraw after cancel |
| `view functions` | 3 | Not found, user index, count increment |

Run all tests:
```bash
cargo test
```

Expected output:
```
running 22 tests
test result: ok. 22 passed; 0 failed
```

---

## Roadmap

### Phase 1 — Contract ✅
- [x] Stream creation with per-second rate calculation
- [x] Withdraw, pause, resume, cancel functions
- [x] Per-user stream index
- [x] Typed on-chain events
- [x] 22 unit tests, 0 warnings
- [x] Deployed to Stellar testnet

### Phase 2 — Frontend ✅
- [x] Freighter wallet connect
- [x] Create stream form
- [x] Live stream dashboard with claimable polling
- [x] Withdraw, pause, resume, cancel actions
- [x] Role-gated buttons and friendly error messages

### Phase 3 — Production Hardening
- [x] GitHub Actions CI pipeline
- [ ] Gas optimization audit
- [ ] Security review and access control audit
- [ ] End-to-end integration tests
- [ ] Mainnet deployment

### Phase 4 — Ecosystem
- [ ] TypeScript SDK for dApp integration
- [ ] Multi-token support (beyond XLM)
- [ ] Stream NFT receipts
- [ ] Batch stream creation

---

## Contributing

We welcome contributions of all kinds — Rust contract work, React/TypeScript frontend, documentation, gas optimization, and security reviews. Issues are scoped for all difficulty levels including good first issues.

See [CONTRIBUTING.md](./CONTRIBUTING.md) for how to get started.

### Commit Convention

```text
feat:     new feature
fix:      bug fix
test:     adding or updating tests
docs:     documentation changes
chore:    tooling, CI, config
refactor: code restructuring
```

---

## License

MIT