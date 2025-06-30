use axum::{routing::post, Json, Router, response::IntoResponse};
use serde_json::json;
use solana_sdk::signature::{Keypair, Signer};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use base64::{engine::general_purpose, Engine as _};

async fn generate_keypair() -> impl IntoResponse {
    // Generate Solana-compatible keypair
    let keypair = Keypair::new();

    // Public key in base58 (Solana standard)
    let pubkey = keypair.pubkey().to_string();

    // Secret key (64 bytes = 32 bytes secret + 32 bytes public)
    let secret_bytes = keypair.to_bytes();
    let secret_base64 = general_purpose::STANDARD.encode(secret_bytes);

    let response = json!({
        "success": true,
        "data": {
            "pubkey": pubkey,
            "secret": secret_base64
        }
    });

    (axum::http::StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/keypair", post(generate_keypair));

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://{}", address);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
    
}
