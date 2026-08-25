# HTTP Server in Rust

A minimal HTTP server built with the [Axum](https://github.com/tokio-rs/axum) web framework and the Tokio async runtime. Its single current endpoint generates a new Solana keypair on demand.

## Tech Stack

- Rust
- Axum 0.7 (HTTP routing and handlers)
- Tokio (async runtime)
- `solana-sdk` (keypair generation)
- `serde` / `serde_json` (JSON responses)
- `base64` (encoding the secret key)

## Project Structure

```
.
├── src/
│   └── main.rs      Server setup and route handlers
├── Cargo.toml
└── Cargo.lock
```

All logic currently lives in a single file, `src/main.rs`.

## Prerequisites

- Rust and Cargo

## Getting Started

1. Clone the repository

   ```
   git clone https://github.com/shrinjoy979/http-server-in-rust.git
   cd http-server-in-rust
   ```

2. Run the server

   ```
   cargo run
   ```

The server listens on `0.0.0.0:3000` and prints its address on startup.

## API Endpoints

### Generate Keypair

`POST /keypair`

Generates a new random Solana keypair and returns it in the response body. No request body is required.

Response:
```json
{
  "success": true,
  "data": {
    "pubkey": "base58-encoded public key",
    "secret": "base64-encoded 64-byte secret key (32-byte secret + 32-byte public key)"
  }
}
```

Example:
```
curl -X POST http://localhost:3000/keypair
```

## Notes

- The generated keypair is created fresh on every request and is not stored anywhere by the server; the caller is responsible for saving the returned secret if they need it.
- Returning a private key over an HTTP response is convenient for local development and testing, but should not be exposed on a public or untrusted network, since anyone who can reach the endpoint receives a usable secret key.
- Despite the repository name, this project builds its server on top of Axum and Tokio rather than implementing the HTTP protocol from scratch over raw TCP sockets.
