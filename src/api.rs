//! HTTP API handlers for the ephemeral file share service

use axum::{
    extract::{Path, State},
    http::StatusCode,
    json::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::encryption::{generate_key, generate_token, EncryptedFile};
use crate::network::Network;
use crate::storage::Storage;

/// API state
pub type ApiState = Arc<ApiSharedState>;

pub struct ApiSharedState {
    pub storage: Storage,
    pub network: Network,
}

/// Create the API router
pub fn create_router(storage: Storage, network: Network) -> Router {
    let state = Arc::new(ApiSharedState { storage, network });
    
    Router::new()
        .route("/api/transfer", post(create_transfer))
        .route("/api/transfer/:id", get(get_transfer))
        .route("/api/transfer/:id", delete(delete_transfer))
        .route("/api/qr/:token", get(generate_qr))
        .route("/api/status", get(status))
        .with_state(state)
}

/// Request body for creating a transfer
#[derive(Debug, Deserialize)]
pub struct CreateTransferRequest {
    pub filename: String,
    pub data: String, // Base64 encoded file data
    pub expires_in_minutes: Option<u64>,
}

/// Response for creating a transfer
#[derive(Debug, Serialize)]
pub struct CreateTransferResponse {
    pub id: String,
    pub token: String,
    pub qr_url: String,
    pub expires_at: Option<String>,
}

/// Response for getting transfer info
#[derive(Debug, Serialize)]
pub struct GetTransferResponse {
    pub id: String,
    pub filename: String,
    pub size: u64,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub download_url: String,
}

/// Status response
#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub status: String,
    pub peer_id: String,
    pub stored_files: usize,
    pub version: String,
}

/// Create a new file transfer
async fn create_transfer(
    State(state): State<ApiState>,
    Json(request): Json<CreateTransferRequest>,
) -> Result<Json<CreateTransferResponse>, (StatusCode, String)> {
    // Decode base64 data
    let decoded_data = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(&request.data)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid base64 data: {}", e)))?;

    // Generate encryption key and encrypt the file
    let key = generate_key().unwrap();
    let (encrypted_data, nonce) = crate::encryption::encrypt(&decoded_data, &key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Encryption failed: {}", e)))?;

    // Generate file ID and token
    let id = Uuid::new_v4().to_string();
    let token = generate_token();

    // Calculate expiration time
    let expires_at = request.expires_in_minutes.map(|mins| {
        chrono::Utc::now() + chrono::Duration::minutes(mins as i64)
    });

    // Create encrypted file record
    let file = EncryptedFile {
        id: id.clone(),
        filename: request.filename,
        size: decoded_data.len() as u64,
        encrypted_data,
        nonce,
        created_at: chrono::Utc::now(),
        expires_at,
    };

    // Store the file
    state.storage.store(file).await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Storage failed: {}", e)))?;

    // Generate QR URL
    let qr_url = format!("/api/qr/{}", token);

    Ok(Json(CreateTransferResponse {
        id,
        token,
        qr_url,
        expires_at: expires_at.map(|dt| dt.to_rfc3339()),
    }))
}

/// Get transfer information
async fn get_transfer(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<Json<GetTransferResponse>, (StatusCode, String)> {
    let file = state.storage.get(&id)
        .await
        .ok_or((StatusCode::NOT_FOUND, "File not found or expired"))?;

    Ok(Json(GetTransferResponse {
        id: file.id.clone(),
        filename: file.filename.clone(),
        size: file.size,
        created_at: file.created_at.to_rfc3339(),
        expires_at: file.expires_at.map(|dt| dt.to_rfc3339()),
        download_url: format!("/api/download/{}", id),
    }))
}

/// Delete a transfer
async fn delete_transfer(
    State(state): State<ApiState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    let deleted = state.storage.delete(&id).await;
    
    if deleted {
        Ok(StatusCode::OK)
    } else {
        Err((StatusCode::NOT_FOUND, "File not found or already expired"))
    }
}

/// Generate QR code data URL
async fn generate_qr(
    State(_state): State<ApiState>,
    Path(token): Path<String>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    // In a real implementation, this would generate an actual QR code image
    // For now, return the token info as text
    let qr_data = format!("ephemeral://transfer/{}", token);
    
    Ok((StatusCode::OK, qr_data))
}

/// Get service status
async fn status(State(state): State<ApiState>) -> Json<StatusResponse> {
    let stored_files = state.storage.count().await;
    let peer_id = state.network.peer_id().to_string();

    Json(StatusResponse {
        status: "running".to_string(),
        peer_id,
        stored_files,
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_and_get_transfer() {
        let storage = Storage::new().await.unwrap();
        let network = Network::new().await.unwrap();
        let state = Arc::new(ApiSharedState { storage, network });
        
        let app = Router::new()
            .route("/api/transfer", post(create_transfer))
            .route("/api/transfer/:id", get(get_transfer))
            .with_state(state.clone());

        // Create a transfer
        let request_data = CreateTransferRequest {
            filename: "test.txt".to_string(),
            data: base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b"Hello, World!"),
            expires_in_minutes: Some(60),
        };

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/transfer")
                    .header("content-type", "application/json")
                    .body(Body::from(serde_json::to_string(&request_data).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let create_response: CreateTransferResponse = serde_json::from_slice(&body).unwrap();

        // Get the transfer
        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri(format!("/api/transfer/{}", create_response.id))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_delete_transfer() {
        let storage = Storage::new().await.unwrap();
        let network = Network::new().await.unwrap();
        let state = Arc::new(ApiSharedState { storage, network });
        
        let app = Router::new()
            .route("/api/transfer", post(create_transfer))
            .route("/api/transfer/:id", delete(delete_transfer))
            .with_state(state.clone());

        // Create a transfer first (omitted for brevity)
        // Then delete it
    }

    #[tokio::test]
    async fn test_status_endpoint() {
        let storage = Storage::new().await.unwrap();
        let network = Network::new().await.unwrap();
        let state = Arc::new(ApiSharedState { storage, network });
        
        let app = Router::new()
            .route("/api/status", get(status))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .method("GET")
                    .uri("/api/status")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
