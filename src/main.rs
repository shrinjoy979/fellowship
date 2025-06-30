use axum::{
    routing::{post},
    Json, Router, response::IntoResponse, http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
};
use std::{net::SocketAddr, str::FromStr};
use tokio::net::TcpListener;
use base64::{engine::general_purpose, Engine as _};
use ed25519_dalek::{PublicKey as DalekPublicKey, Signature as DalekSignature, Verifier};
use bs58;

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

    let mut data = vec![0u8];
    data.push(payload.decimals);
    data.extend_from_slice(mint_authority.as_ref());
    data.extend_from_slice(&[0u8; 1]);

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

#[derive(Deserialize)]
struct VerifyMessageRequest {
    message: String,
    signature: String,
    pubkey: String,
}

#[derive(Serialize)]
struct VerifyMessageResponseData {
    valid: bool,
    message: String,
    pubkey: String,
}

async fn verify_message(Json(payload): Json<VerifyMessageRequest>) -> impl IntoResponse {
    let pubkey_bytes: Vec<u8> = match bs58::decode(&payload.pubkey).into_vec() {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid base58 pubkey" })),
            );
        }
    };

    let pubkey = match DalekPublicKey::from_bytes(&pubkey_bytes) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid public key bytes" })),
            );
        }
    };

    let signature_bytes = match general_purpose::STANDARD.decode(&payload.signature) {
        Ok(bytes) => bytes,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid base64 signature" })),
            );
        }
    };

    let signature = match DalekSignature::from_bytes(&signature_bytes) {
        Ok(sig) => sig,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid signature bytes" })),
            );
        }
    };

    let valid = pubkey.verify(payload.message.as_bytes(), &signature).is_ok();

    let response_data = VerifyMessageResponseData {
        valid,
        message: payload.message,
        pubkey: payload.pubkey,
    };

    let response = json!({
        "success": true,
        "data": response_data
    });

    (StatusCode::OK, Json(response))
}

#[derive(Deserialize)]
struct SendSolRequest {
    from: String,
    to: String,
    lamports: u64,
}

async fn send_sol(Json(payload): Json<SendSolRequest>) -> impl IntoResponse {
    let from_pubkey = match Pubkey::from_str(&payload.from) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid sender pubkey" })),
            );
        }
    };

    let to_pubkey = match Pubkey::from_str(&payload.to) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid recipient pubkey" })),
            );
        }
    };

    if payload.lamports == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Lamports must be greater than zero" })),
        );
    }

    let program_id = system_program::id();

    let accounts = vec![
        AccountMeta::new(from_pubkey, true),
        AccountMeta::new(to_pubkey, false),
    ];

    let amount_bytes = payload.lamports.to_le_bytes();

    let mut instruction_data = Vec::with_capacity(9);
    instruction_data.push(2);
    instruction_data.extend_from_slice(&amount_bytes);

    let response = json!({
        "success": true,
        "data": {
            "program_id": program_id.to_string(),
            "accounts": [
                from_pubkey.to_string(),
                to_pubkey.to_string()
            ],
            "instruction_data": general_purpose::STANDARD.encode(instruction_data)
        }
    });

    (StatusCode::OK, Json(response))
}

#[derive(Deserialize)]
struct SendTokenRequest {
    destination: String,
    mint: String,
    owner: String,
    amount: u64,
}

#[derive(Serialize)]
struct SendTokenAccountInfo {
    pubkey: String,
    isSigner: bool,
}

async fn send_token(Json(payload): Json<SendTokenRequest>) -> impl IntoResponse {
    let destination = match Pubkey::from_str(&payload.destination) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid destination pubkey" })),
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

    let owner = match Pubkey::from_str(&payload.owner) {
        Ok(pk) => pk,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({ "success": false, "error": "Invalid owner pubkey" })),
            );
        }
    };

    if payload.amount == 0 {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "success": false, "error": "Amount must be greater than zero" })),
        );
    }

    let program_id = Pubkey::from_str("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap();

    let accounts = vec![
        SendTokenAccountInfo {
            pubkey: destination.to_string(),
            isSigner: false,
        },
        SendTokenAccountInfo {
            pubkey: mint.to_string(),
            isSigner: false,
        },
        SendTokenAccountInfo {
            pubkey: owner.to_string(),
            isSigner: true,
        },
    ];

    let mut instruction_data = Vec::with_capacity(9);
    instruction_data.push(3u8);
    instruction_data.extend_from_slice(&payload.amount.to_le_bytes());

    let response = json!({
        "success": true,
        "data": {
            "program_id": program_id.to_string(),
            "accounts": accounts,
            "instruction_data": general_purpose::STANDARD.encode(instruction_data)
        }
    });

    (StatusCode::OK, Json(response))
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token))
        .route("/token/mint", post(mint_token))
        .route("/message/verify", post(verify_message))
        .route("/send/sol", post(send_sol))
        .route("/send/token", post(send_token));

    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("Server running at http://{}", address);

    let listener = TcpListener::bind(address).await.unwrap();
    axum::serve(listener, app.into_make_service()).await.unwrap();
}
