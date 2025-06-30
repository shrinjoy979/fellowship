use axum::{routing::{post}, Json, Router, response::IntoResponse, http::StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use base64::{engine::general_purpose, Engine as _};

async fn generate_keypair() -> impl IntoResponse {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret_base64 = general_purpose::STANDARD.encode(keypair.to_bytes());

    let response = json!({
        "success": true,
        "data": {
            "pubkey": pubkey,
            "secret": secret_base64
        }
    });

    (StatusCode::OK, Json(response))
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    mintAuthority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct AccountMetaInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
}

async fn create_token(Json(payload): Json<CreateTokenRequest>) -> impl IntoResponse {
    let mint_authority = match Pubkey::from_str(&payload.mintAuthority) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid mintAuthority pubkey" })),
            );
        }
    };

    let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid mint pubkey" })),
            );
        }
    };

    let program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let accounts = vec![
        AccountMeta::new(mint, true),
        AccountMeta::new_readonly(mint_authority, true),
    ];

    let mut data = vec![0u8]; // InitializeMint instruction index
    data.push(payload.decimals);
    data.extend_from_slice(mint_authority.as_ref()); // mint authority pubkey bytes
    data.extend_from_slice(&[0u8; 1]); // freeze authority: None

    let account_info: Vec<AccountMetaInfo> = accounts
        .iter()
        .map(|acc| AccountMetaInfo {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = json!({
        "success": true,
        "data": {
            "program_id": program_id.to_string(),
            "accounts": account_info,
            "instruction_data": general_purpose::STANDARD.encode(data)
        }
    });

    (StatusCode::OK, Json(response))
}

#[derive(Deserialize)]
struct MintTokenRequest {
    mint: String,
    destination: String,
    authority: String,
    amount: u64,
}

async fn mint_token(Json(payload): Json<MintTokenRequest>) -> impl IntoResponse {
    let mint = match Pubkey::from_str(&payload.mint) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid mint pubkey" })),
            );
        }
    };

    let destination = match Pubkey::from_str(&payload.destination) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid destination pubkey" })),
            );
        }
    };

    let authority = match Pubkey::from_str(&payload.authority) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid authority pubkey" })),
            );
        }
    };

    let program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let accounts = vec![
        AccountMeta::new(mint, false),
        AccountMeta::new(destination, false),
        AccountMeta::new_readonly(authority, true),
    ];

    // Instruction index for MintTo is 7 (according to SPL Token spec)
    let mut data = vec![7u8];
    data.extend_from_slice(&payload.amount.to_le_bytes());

    let account_info: Vec<AccountMetaInfo> = accounts
        .iter()
        .map(|acc| AccountMetaInfo {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
            is_writable: acc.is_writable,
        })
        .collect();

    let response = json!({
        "success": true,
        "data": {
            "program_id": program_id.to_string(),
            "accounts": account_info,
            "instruction_data": general_purpose::STANDARD.encode(data)
        }
    });

    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token));

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://{}", address);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
