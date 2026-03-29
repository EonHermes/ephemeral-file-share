//! P2P networking layer using libp2p and WebRTC

use anyhow::Result;
use libp2p::{
    identity,
    multiaddr::Multiaddr,
    noise,
    tcp,
    yamux,
    PeerId,
    SwarmBuilder,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Network manager for P2P connections
pub struct Network {
    peer_id: PeerId,
    multiaddr: Option<Multiaddr>,
    connected_peers: Arc<RwLock<Vec<PeerId>>>,
}

impl Network {
    /// Create a new network instance
    pub async fn new() -> Result<Self> {
        // Generate keypair
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        info!("Generated P2P peer ID: {}", peer_id);
        
        Ok(Self {
            peer_id,
            multiaddr: None,
            connected_peers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Get the local peer ID
    pub fn peer_id(&self) -> PeerId {
        self.peer_id
    }

    /// Generate a multiaddress for this node
    pub async fn get_multiaddr(&self) -> Option<Multiaddr> {
        // For now, return a basic TCP address
        // In production, this would include relay addresses and WebRTC endpoints
        let addr: Multiaddr = format!("/ip4/127.0.0.1/tcp/3001/p2p/{}", self.peer_id)
            .parse()
            .ok();
        
        self.multiaddr.clone().or(addr)
    }

    /// Add a peer to the connected peers list
    pub async fn add_peer(&self, peer_id: PeerId) {
        let mut peers = self.connected_peers.write().await;
        if !peers.contains(&peer_id) {
            peers.push(peer_id);
            info!("Connected to peer: {}", peer_id);
        }
    }

    /// Remove a peer from the connected peers list
    pub async fn remove_peer(&self, peer_id: &PeerId) {
        let mut peers = self.connected_peers.write().await;
        if let Some(pos) = peers.iter().position(|p| p == peer_id) {
            peers.remove(pos);
            info!("Disconnected from peer: {}", peer_id);
        }
    }

    /// Get list of connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerId> {
        let peers = self.connected_peers.read().await;
        peers.clone()
    }

    /// Generate a connection token that encodes peer information
    pub fn generate_connection_token(&self, file_id: &str) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        
        let mut data = format!("{}:{}", self.peer_id, file_id).into_bytes();
        // Add some randomness
        data.extend_from_slice(&rand::random::<[u8; 16]>());
        
        URL_SAFE_NO_PAD.encode(data)
    }

    /// Parse a connection token to extract peer and file info
    pub fn parse_connection_token(token: &str) -> Option<(PeerId, String)> {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
        
        let decoded = URL_SAFE_NO_PAD.decode(token).ok()?;
        let text = String::from_utf8(decoded).ok()?;
        
        let parts: Vec<&str> = text.split(':').collect();
        if parts.len() >= 2 {
            let peer_id = parts[0].parse().ok()?;
            let file_id = parts[1..].join(":");
            Some((peer_id, file_id))
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_generate_and_parse_token() {
        let network = Network::new().await.unwrap();
        
        let file_id = "test-file-123";
        let token = network.generate_connection_token(file_id);
        
        let (parsed_peer, parsed_file) = Network::parse_connection_token(&token).unwrap();
        
        assert_eq!(parsed_peer, network.peer_id());
        assert_eq!(parsed_file, file_id);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        let result = Network::parse_connection_token("invalid-token");
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_peer_management() {
        let network = Network::new().await.unwrap();
        
        // Create a fake peer ID for testing
        let keypair = identity::Keypair::generate_ed25519();
        let peer_id = PeerId::from(keypair.public());
        
        assert_eq!(network.get_connected_peers().await.len(), 0);
        
        network.add_peer(peer_id).await;
        assert_eq!(network.get_connected_peers().await.len(), 1);
        
        network.remove_peer(&peer_id).await;
        assert_eq!(network.get_connected_peers().await.len(), 0);
    }
}
