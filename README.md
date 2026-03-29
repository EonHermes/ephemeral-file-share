# Ephemeral File Share 🔐

A peer-to-peer file sharing system with end-to-end encryption and self-destructing links. Built with Rust and React.

## Features ✨

- **End-to-End Encryption**: All files are encrypted using ChaCha20-Poly1305 before storage
- **Self-Destructing Links**: Set expiration times for your file transfers
- **P2P Networking**: libp2p-based peer-to-peer connectivity with NAT traversal
- **QR Code Support**: Generate QR codes for quick mobile transfers
- **Privacy-First**: No central servers store your data long-term
- **Clean Architecture**: Well-tested Rust backend with React frontend

## Quick Start 🚀

### Backend (Rust)

```bash
# Install dependencies
cargo build --release

# Run the server
cargo run --release

# Server starts on http://localhost:3000
```

### Frontend (React)

```bash
cd frontend
npm install
npm run dev
```

## API Reference 📚

### Create Transfer

```http
POST /api/transfer
Content-Type: application/json

{
  "filename": "secret.txt",
  "data": "base64-encoded-file-content",
  "expires_in_minutes": 60
}
```

**Response:**
```json
{
  "id": "uuid-1234-5678",
  "token": "secure-token",
  "qr_url": "/api/qr/token",
  "expires_at": "2024-01-01T12:00:00Z"
}
```

### Get Transfer Info

```http
GET /api/transfer/:id
```

### Delete Transfer

```http
DELETE /api/transfer/:id
```

### Service Status

```http
GET /api/status
```

## Architecture 🏗️

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   React     │────▶│    Rust      │────▶│  libp2p     │
│   Frontend  │     │   Backend    │     │  P2P Layer  │
└─────────────┘     └──────────────┘     └─────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   Storage    │
                    │ (Ephemeral)  │
                    └──────────────┘
```

### Components

- **`src/api.rs`**: HTTP API handlers using Axum
- **`src/encryption.rs`**: ChaCha20-Poly1305 encryption module
- **`src/storage.rs`**: In-memory storage with auto-cleanup
- **`src/network.rs`**: libp2p P2P networking layer
- **`src/qr.rs`**: QR code generation utilities

## Security 🔒

- All file data is encrypted before storage
- Encryption keys are generated per-file and never stored
- Files automatically expire and are deleted
- No persistent storage of sensitive data
- End-to-end encryption ensures only sender/receiver can decrypt

## Testing 🧪

```bash
# Run all tests
cargo test

# Run with coverage
cargo tarpaulin --out Html

# Test specific module
cargo test encryption
```

### Test Coverage

The project includes comprehensive unit tests for:
- Encryption/decryption operations
- Storage management and expiration
- Network token generation/parsing
- API endpoints

## Development 💻

### Prerequisites

- Rust 1.70+
- Node.js 18+ (for frontend)
- Cargo (Rust package manager)

### Project Structure

```
ephemeral-file-share/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Library exports
│   ├── api.rs           # HTTP API handlers
│   ├── encryption.rs    # Encryption module
│   ├── storage.rs       # Storage management
│   ├── network.rs       # P2P networking
│   └── qr.rs            # QR code utilities
├── frontend/            # React frontend (separate repo)
├── Cargo.toml          # Rust dependencies
├── README.md           # This file
└── tests/              # Integration tests
```

## Use Cases 💡

1. **Secure Document Sharing**: Send sensitive documents that self-destruct after viewing
2. **Temporary File Transfer**: Share large files without permanent storage
3. **Privacy-First Collaboration**: Collaborate without leaving digital traces
4. **Mobile-to-Desktop Transfer**: Use QR codes for quick file transfers
5. **Ephemeral Backups**: Temporary backup with automatic cleanup

## Future Enhancements 🚀

- [ ] WebRTC integration for browser-based P2P
- [ ] Relay servers for NAT traversal
- [ ] Mobile app (React Native)
- [ ] File chunking for large transfers
- [ ] Resume interrupted transfers
- [ ] Password protection option
- [ ] Browser extension

## License 📄

MIT License - See LICENSE file for details.

## Contributing 🤝

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

---

Built with ❤️ using Rust and React by [EonHermes](https://github.com/EonHermes)
