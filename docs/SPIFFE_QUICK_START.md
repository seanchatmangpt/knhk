# SPIFFE/SPIRE Quick Start Guide

## Installation

The SPIFFE/SPIRE integration is built into `knhk-sidecar`. No additional installation needed.

## Basic Setup

### 1. Configure SPIRE Agent

```bash
# Ensure SPIRE agent is running with workload API enabled
# Default socket: /tmp/spire-agent/public/api.sock

# Check SPIRE status
ps aux | grep spire-agent
ls -la /tmp/spire-agent/public/api.sock
```

### 2. Configure KNHK

Set environment variables:

```bash
export KGC_SPIFFE_ENABLED=true
export KGC_SPIFFE_SOCKET_PATH=/tmp/spire-agent/public/api.sock
export KGC_SPIFFE_TRUST_DOMAIN=example.com
```

### 3. Initialize in Your Code

```rust
use knhk_sidecar::spiffe::{SpiffeConfig, SpiffeCertManager};

async fn setup_spiffe() -> Result<SpiffeCertManager, Box<dyn std::error::Error>> {
    // Create config
    let config = SpiffeConfig::new("example.com".to_string());

    // Initialize manager
    let mut manager = SpiffeCertManager::new(config)?;

    // Load initial certificate
    manager.load_certificate().await?;

    // Start automatic refresh (every 1 hour)
    manager.start_refresh_task()?;

    Ok(manager)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let spiffe_manager = setup_spiffe().await?;

    // Get certificate for TLS
    let cert = spiffe_manager.get_certificate()?;
    let key = spiffe_manager.get_private_key()?;
    let spiffe_id = spiffe_manager.get_spiffe_id()?;

    println!("Loaded SPIFFE ID: {}", spiffe_id);
    println!("Certificate size: {} bytes", cert.len());

    Ok(())
}
```

## Common Usage Patterns

### Using with Tonic gRPC

```rust
use tonic::transport::Server;
use tonic_prost::ProstCodec;
use knhk_sidecar::spiffe::{SpiffeConfig, SpiffeCertManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Setup SPIFFE
    let mut spiffe_manager = SpiffeCertManager::new(
        SpiffeConfig::new("example.com".to_string())
    )?;
    spiffe_manager.load_certificate().await?;
    spiffe_manager.start_refresh_task()?;

    // Get TLS credentials
    let cert = spiffe_manager.get_certificate()?;
    let key = spiffe_manager.get_private_key()?;

    // Configure Tonic with certificate
    let identity = tonic::transport::Identity::from_pem(cert, key);
    let tls = tonic::transport::ServerTlsConfig::new()
        .identity(identity);

    // Start server
    let addr = "0.0.0.0:50051".parse()?;
    Server::builder()
        .tls_config(tls)?
        .add_service(/* your service */)
        .serve(addr)
        .await?;

    Ok(())
}
```

### Peer Verification

```rust
// When receiving a peer certificate, verify its SPIFFE ID
fn verify_peer_certificate(
    spiffe_manager: &SpiffeCertManager,
    peer_cert_pem: &[u8],
) -> bool {
    // In real implementation, parse certificate to extract peer's SPIFFE ID
    // Then verify it matches expected trust domain

    let peer_spiffe_id = "spiffe://example.com/api-service";
    spiffe_manager.verify_peer_id(peer_spiffe_id)
}
```

### Manual Refresh

```rust
// Force immediate refresh (useful for testing)
async fn refresh_now(spiffe_manager: &SpiffeCertManager) -> Result<(), Box<dyn std::error::Error>> {
    spiffe_manager.load_certificate().await?;
    println!("Certificate refreshed!");
    Ok(())
}
```

## Configuration Options

### SpiffeConfig Fields

```rust
pub struct SpiffeConfig {
    // SPIRE agent socket path
    pub socket_path: String,

    // Trust domain (e.g., "example.com")
    pub trust_domain: String,

    // Optional explicit SPIFFE ID
    // If None, defaults to spiffe://{trust_domain}/sidecar
    pub spiffe_id: Option<String>,

    // Refresh interval (default: 1 hour)
    pub refresh_interval: Duration,
}
```

### Advanced Configuration

```rust
// Custom refresh interval (30 minutes)
let mut config = SpiffeConfig::new("example.com".to_string());
config.refresh_interval = std::time::Duration::from_secs(1800);

// Custom SPIFFE ID
config.spiffe_id = Some("spiffe://example.com/my-service".to_string());

// Custom socket path
config.socket_path = "/var/run/spire/agent.sock".to_string();
```

## Error Handling

### Handling Connection Errors

```rust
async fn load_with_retry(
    manager: &SpiffeCertManager,
    max_retries: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    for attempt in 1..=max_retries {
        match manager.load_certificate().await {
            Ok(_) => {
                println!("Certificate loaded successfully");
                return Ok(());
            }
            Err(e) => {
                eprintln!("Attempt {}: Failed to load certificate: {}", attempt, e);
                if attempt < max_retries {
                    tokio::time::sleep(
                        tokio::time::Duration::from_secs(2u64.pow(attempt - 1))
                    ).await;
                }
            }
        }
    }
    Err("Failed to load certificate after retries".into())
}
```

### Checking Certificate Status

```rust
// Check if certificate needs refresh
if manager.needs_refresh() {
    println!("Certificate refresh needed");
    manager.load_certificate().await?;
}

// Check TTL remaining
if let Some(ttl) = manager.get_certificate_ttl() {
    println!("Certificate expires in: {:?}", ttl);
    if ttl.as_secs() < 3600 {
        println!("Certificate expiring soon!");
    }
}
```

## Logging

Enable debug logging to see what's happening:

```bash
# In your Rust code, initialize tracing
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // Initialize tracing with debug level
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Now SPIFFE debug messages will appear
    // Example output:
    // DEBUG: Connecting to SPIRE agent at: /tmp/spire-agent/public/api.sock
    // INFO: Successfully connected to SPIRE workload API
    // DEBUG: Received 1024 bytes from SPIRE workload API
    // INFO: Successfully parsed SPIRE response: SPIFFE ID = spiffe://example.com/service
}
```

## Testing

### Mock SPIRE for Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spiffe_id_extraction() {
        let config = SpiffeConfig::new("example.com".to_string());
        assert_eq!(
            config.extract_spiffe_id(),
            "spiffe://example.com/sidecar"
        );
    }

    #[test]
    fn test_peer_verification() {
        let config = SpiffeConfig::new("example.com".to_string());
        let manager = SpiffeCertManager::new(config)
            .expect("Failed to create manager");

        // Same trust domain - should succeed
        assert!(manager.verify_peer_id("spiffe://example.com/other-service"));

        // Different trust domain - should fail
        assert!(!manager.verify_peer_id("spiffe://other.com/service"));
    }
}
```

### Integration Testing with Real SPIRE

```bash
# 1. Start SPIRE agent
docker run -d --name spire-agent \
  -v /tmp/spire-agent:/tmp/spire-agent \
  ghcr.io/spiffe/spire-agent:latest

# 2. Run your application
cargo run

# 3. Check logs for successful certificate load
# Should see: "Successfully loaded X.509-SVID from SPIRE"
```

## Performance Tips

1. **Refresh Interval**: Use 1 hour for production (default)
2. **Connection**: Local Unix socket is fast (~5ms)
3. **Caching**: Don't cache certificate indefinitely (let SPIRE manage rotation)
4. **Async**: Always use async/await with SPIFFE operations
5. **Error Handling**: Gracefully degrade if SPIRE unavailable

## Troubleshooting

### Problem: "SPIRE agent socket not found"

```bash
# Check if socket exists
ls -la /tmp/spire-agent/public/api.sock

# Check if SPIRE is running
ps aux | grep spire-agent

# If not running, start SPIRE agent
docker run -d --name spire-agent ghcr.io/spiffe/spire-agent:latest
```

### Problem: "Connection timeout"

```bash
# Increase timeout in code or check SPIRE health
# Default timeouts: 5s connection, 10s read

# Check SPIRE logs
docker logs spire-agent

# Test connectivity
nc -U /tmp/spire-agent/public/api.sock
```

### Problem: "Cannot extract SPIFFE ID"

```bash
# Certificate might not have SPIFFE ID in SAN
# Check certificate details
openssl x509 -text -noout < /path/to/cert.pem
# Look for: "URI: spiffe://..."

# Contact SPIRE admin to verify certificate generation
```

## Security Checklist

- [ ] SPIRE agent running on localhost only
- [ ] Socket permissions set correctly (0600)
- [ ] Trust domain configured correctly
- [ ] Peer SPIFFE IDs verified before communication
- [ ] Certificate refresh enabled (automatic)
- [ ] Error handling doesn't expose sensitive info
- [ ] Logging doesn't capture private keys
- [ ] TLS used for all peer communication

## Next Steps

1. Review: `/home/user/knhk/docs/SPIFFE_SPIRE_INTEGRATION.md` for detailed documentation
2. Check: `/home/user/knhk/rust/knhk-sidecar/src/spiffe.rs` for API documentation
3. Run: `cargo test --lib spiffe` to run unit tests
4. Deploy: Use integration with your Tonic/gRPC services

## Support

For issues or questions, refer to:
- SPIFFE: https://spiffe.io/
- SPIRE: https://github.com/spiffe/spire
- KNHK Documentation: `/home/user/knhk/docs/`

---

**Version**: 1.0.0
**Last Updated**: 2025-11-16
