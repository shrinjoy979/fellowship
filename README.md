# Fellowship

An HTTP API, built with Axum and Tokio, that exposes a set of Solana-related utility endpoints: generating keypairs, signing and verifying messages, and constructing (but not submitting) SPL Token and native SOL transfer instructions.

The server does not connect to any Solana RPC endpoint or submit transactions itself — endpoints that build instructions return the program id, account metadata, and base64-encoded instruction data for the caller to assemble and sign into a transaction elsewhere.

## Tech Stack

- Rust
- Axum 0.7 (HTTP routing and handlers)
- Tokio (async runtime)
- `solana-sdk` (keys, pubkeys, account metas)
- `ed25519-dalek` (signature verification)
- `bs58` (base58 encoding/decoding)
- `serde` / `serde_json` (JSON request and response bodies)
- `base64` (encoding secrets, signatures, and instruction data)

## Project Structure

```
.
├── src/
│   └── main.rs       Server setup and all route handlers
├── Cargo.toml
├── Cargo.lock
├── Dockerfile         Multi-stage build using cargo-chef
└── fly.toml           Fly.io deployment configuration
```

All application logic currently lives in a single file, `src/main.rs`.

## Prerequisites

- Rust and Cargo

## Getting Started

1. Clone the repository

   ```
   git clone https://github.com/shrinjoy979/fellowship.git
   cd fellowship
   ```

2. Run the server

   ```
   cargo run
   ```

The server listens on `0.0.0.0:8080` and prints its address on startup.

## Deployment

A `Dockerfile` (using `cargo-chef` for dependency-layer caching) and a `fly.toml` are included for deploying to [Fly.io](https://fly.io). The container listens on port `8080`, matching the app's `PORT` environment variable in `fly.toml`.

## API Endpoints

All endpoints accept and return JSON, and respond in the shape `{ "success": true, "data": { ... } }` on success or `{ "success": false, "error": "..." }` on failure (typically with a `400` status for invalid input).

### `POST /keypair`

Generates a new random Solana keypair. No request body required.

Response `data`:
```json
{ "pubkey": "base58 public key", "secret": "base64-encoded 64-byte secret key" }
```

### `POST /token/create`

Builds an SPL Token `InitializeMint`-style instruction for creating a new token mint.

Request body:
```json
{ "mintAuthority": "base58 pubkey", "mint": "base58 pubkey", "decimals": 9 }
```

Response `data` includes `program_id` (the SPL Token program), the list of `accounts` (pubkey, is_signer, is_writable), and base64-encoded `instruction_data`.

### `POST /token/mint`

Builds an SPL Token `MintTo`-style instruction.

Request body:
```json
{ "mint": "base58 pubkey", "destination": "base58 pubkey", "authority": "base58 pubkey", "amount": 1000000 }
```

Response shape matches `/token/create`.

### `POST /message/sign`

Signs an arbitrary message with a provided base58-encoded secret key.

Request body:
```json
{ "message": "string to sign", "secret": "base58-encoded secret key" }
```

Response `data`:
```json
{ "signature": "base64-encoded signature", "public_key": "base58 pubkey", "message": "string that was signed" }
```

### `POST /message/verify`

Verifies an ed25519 signature against a message and public key.

Request body:
```json
{ "message": "string", "signature": "base64-encoded signature", "pubkey": "base58 pubkey" }
```

Response `data`:
```json
{ "valid": true, "message": "string", "pubkey": "base58 pubkey" }
```

### `POST /send/sol`

Builds a native SOL transfer instruction (System Program).

Request body:
```json
{ "from": "base58 pubkey", "to": "base58 pubkey", "lamports": 1000000 }
```
`lamports` must be greater than zero. Response `data` includes `program_id` (the System Program), the `accounts` involved, and base64-encoded `instruction_data`.

### `POST /send/token`

Builds an SPL Token transfer-style instruction.

Request body:
```json
{ "destination": "base58 pubkey", "mint": "base58 pubkey", "owner": "base58 pubkey", "amount": 1000000 }
```
`amount` must be greater than zero. Response `data` includes `program_id` (the SPL Token program), the `accounts` involved (each with `pubkey` and `isSigner`), and base64-encoded `instruction_data`.

## Notes

- This server builds instructions and signs/verifies messages, but never holds funds, submits transactions, or talks to a Solana cluster — all blockchain submission is left to the caller.
- `/message/sign` accepts a raw secret key in the request body; treat this endpoint as a development/testing convenience rather than something to expose with real funds behind it in a production or public deployment.
- Pubkey and secret key inputs are validated for correct encoding, but the SPL Token instruction builders do not verify that the accounts you pass in actually correspond to existing mints, token accounts, or authorities on-chain — that validation happens when the resulting transaction is submitted.
