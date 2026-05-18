# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Autonolas governance contracts on NEAR Protocol. The core contract is `WormholeMessenger` — a cross-chain governance executor that receives Wormhole VAA (Validator Action Approval) messages from a foreign chain (Ethereum/Sepolia) and executes batched calls on NEAR.

## Architecture

- **`src/lib.rs`** — Main contract (`WormholeMessenger`). Receives VAAs via `delivery()`, verifies them through the Wormhole Core contract, then sequentially executes a batch of `Call` structs (up to 10) parsed from the VAA payload. Supports contract self-upgrade via hash-gated `upgrade_contract()`.
- **`src/state.rs`** — VAA binary parsing (`ParsedVAA`). Extracts header, signatures, emitter chain/address, sequence, and payload from raw Wormhole VAA bytes.
- **`src/byte_utils.rs`** — Big-endian byte deserialization helpers (`ByteUtils` trait on `&[u8]`).
- **`artifacts/wormhole_near.wasm`** — Pre-built mock Wormhole Core contract used in sandbox tests.

## Build & Test Commands

```bash
# Install dependencies
yarn

# Build the WASM contract (outputs to artifacts/governance_near.wasm)
./scripts/build.sh

# Run sandbox tests (requires prior build)
npx ava test/WormholeMessenger.ts

# Run testnet tests
npx ava --config ava.testnet.config.cjs test/testnet_WormholeMessenger.ts

# Run sandbox tests with debug logging
NEAR_WORKSPACES_DEBUG=true npx ava test/WormholeMessenger.ts

# Rust formatting and linting
cargo fmt
cargo clippy
```

## Prerequisites

- Rust 1.81+ with `wasm32-unknown-unknown` target (added automatically by build script)
- near-cli-rs 0.16.0+
- Node.js + Yarn
- Run `setup-env.sh` to install the full toolchain

## Conventions

- Commit messages use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `perf:`
- Branch names: `feat/<topic>`, `fix/<topic>`, `docs/<topic>`
- Tests use AVA framework with `near-workspaces` for NEAR sandbox simulation
- The contract deploys from `target/wasm32-unknown-unknown/release/governance_near.wasm`; the build script copies it to `artifacts/`
