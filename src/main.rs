//! Ephemeral File Share - P2P file sharing with end-to-end encryption
//!
//! This application enables secure, peer-to-peer file transfers with:
//! - End-to-end encryption using ChaCha20-Poly1305
//! - Self-destructing transfer links
//! - NAT traversal via libp2p and WebRTC
//! - QR code generation for quick transfers

mod network;
mod encryption;
mod storage;
mod api;
mod qr;

use axum::Router;
use std::net::SocketAddr;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ephemeral_file_share=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    
    tracing::info!("Starting Ephemeral File Share server on {}", addr);

    // Initialize storage
    let storage = storage::Storage::new().await?;
    
    // Initialize network layer
    let network = network::Network::new().await?;
    
    // Build router
    let app = Router::new()
        .merge(api::create_router(storage, network))
        .with_state(());

    // Start server
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("Server listening on {}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}
