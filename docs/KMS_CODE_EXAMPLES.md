# KMS Code Examples - Developer Guide

## Quick Start Examples

### 1. Azure Key Vault

#### Basic Setup
```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Azure Key Vault
    let config = KmsConfig::azure(
        "https://my-vault.vault.azure.net".to_string(),
        "my-signing-key".to_string()
    );

    // Validate configuration
    config.validate()?;

    // Create KMS manager
    let manager = KmsManager::new(config).await?;

    // Sign data
    let data = b"Hello, Azure KMS!";
    let signature = manager.sign(data).await?;
    println!("Signature: {:?}", hex::encode(&signature));

    Ok(())
}
```

#### Environment Setup
```bash
# Set Azure authentication token
export AZURE_AUTH_TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."

# Or use Managed Identity (if running in Azure)
# No environment variable needed
```

#### Common Operations
```rust
// Get public key
let public_key = manager.get_public_key().await?;
println!("Public Key: {}", hex::encode(&public_key));

// Rotate key
let new_version = manager.rotate_key().await?;
println!("New key version: {}", new_version);

// Get key metadata
let metadata = manager.get_key_metadata().await?;
println!("Key ID: {}", metadata.key_id);
println!("Created: {:?}", metadata.created_at);
println!("Algorithm: {}", metadata.algorithm);
```

### 2. HashiCorp Vault

#### Basic Setup
```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure Vault
    let config = KmsConfig::vault(
        "https://vault.example.com".to_string(),
        "transit".to_string(),
        "my-app-key".to_string()
    );

    // Validate configuration
    config.validate()?;

    // Create KMS manager
    let manager = KmsManager::new(config).await?;

    // Sign data
    let data = b"Message to sign";
    let signature = manager.sign(data).await?;
    println!("Signature: {:?}", hex::encode(&signature));

    Ok(())
}
```

#### Environment Setup
```bash
# Set Vault token (required)
export VAULT_TOKEN="hvs.CAESIFake..."

# Optional: Set Vault address if not using default
export VAULT_ADDR="https://vault.example.com"
```

#### Common Operations
```rust
// Get public key
let public_key = manager.get_public_key().await?;
println!("Public Key: {}", hex::encode(&public_key));

// Rotate key (creates new version)
let rotated = manager.rotate_key().await?;
println!("Rotated key: {}", rotated);

// Get key metadata
let metadata = manager.get_key_metadata().await?;
println!("Key type: {}", metadata.algorithm);
println!("Key ID: {}", metadata.key_id);
```

### 3. AWS KMS

#### Basic Setup
```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configure AWS KMS
    let config = KmsConfig::aws(
        "us-west-2".to_string(),
        "arn:aws:kms:us-west-2:123456789012:key/12345678-1234-1234-1234-123456789012".to_string()
    );

    // Validate configuration
    config.validate()?;

    // Create KMS manager
    let manager = KmsManager::new(config).await?;

    // Sign data
    let data = b"Secure message";
    let signature = manager.sign(data).await?;
    println!("Signature: {:?}", hex::encode(&signature));

    Ok(())
}
```

#### Environment Setup
```bash
# Use AWS credential chain (no environment setup needed)
# Either:
# - IAM role (if running on EC2, ECS, Lambda, etc.)
# - AWS credentials file (~/.aws/credentials)
# - Environment variables: AWS_ACCESS_KEY_ID, AWS_SECRET_ACCESS_KEY
# - SSO session

# Optional: Specify region
export AWS_REGION="us-west-2"
```

#### Common Operations
```rust
// Get public key
let public_key = manager.get_public_key().await?;
println!("Public Key (DER): {}", hex::encode(&public_key));

// Rotate key
let new_key_id = manager.rotate_key().await?;
println!("Rotated key ID: {}", new_key_id);

// Get key metadata
let metadata = manager.get_key_metadata().await?;
println!("Key ID: {}", metadata.key_id);
println!("Algorithm: {}", metadata.algorithm);
println!("Created: {:?}", metadata.created_at);
```

## Advanced Examples

### 1. Multi-Provider Support

```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};

async fn create_manager(provider: &str) -> Result<KmsManager, Box<dyn std::error::Error>> {
    let config = match provider {
        "azure" => KmsConfig::azure(
            std::env::var("AZURE_VAULT_URL")?,
            std::env::var("AZURE_KEY_NAME")?,
        ),
        "vault" => KmsConfig::vault(
            std::env::var("VAULT_ADDR")?,
            std::env::var("VAULT_MOUNT_PATH")?,
            std::env::var("VAULT_KEY_NAME")?,
        ),
        "aws" => KmsConfig::aws(
            std::env::var("AWS_REGION")?,
            std::env::var("AWS_KMS_KEY_ID")?,
        ),
        _ => panic!("Unknown provider: {}", provider),
    };

    config.validate()?;
    Ok(KmsManager::new(config).await?)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let provider = std::env::var("KMS_PROVIDER").unwrap_or_else(|_| "aws".to_string());
    let manager = create_manager(&provider).await?;

    let signature = manager.sign(b"data").await?;
    println!("Signed with {}: {}", provider, hex::encode(&signature));

    Ok(())
}
```

### 2. Key Rotation with Auto-Rotation

```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create manager with auto-rotation enabled
    let mut config = KmsConfig::azure(
        "https://vault.azure.net".to_string(),
        "signing-key".to_string()
    );

    // Set rotation interval to 12 hours
    config.rotation_interval = Duration::from_secs(12 * 60 * 60);
    config.auto_rotation_enabled = true;

    let mut manager = KmsManager::new(config).await?;

    // Check if rotation is needed
    if manager.needs_rotation() {
        match manager.rotate_if_needed().await {
            Ok(()) => println!("Key rotated successfully"),
            Err(e) => eprintln!("Rotation failed: {}", e),
        }
    }

    Ok(())
}
```

### 3. Signature Verification Pattern

```rust
use knhk_sidecar::kms::KmsManager;

// Helper function to verify a signature with public key
// Note: This is pseudo-code; actual implementation depends on your crypto library
async fn verify_signature(
    manager: &KmsManager,
    data: &[u8],
    signature: &[u8]
) -> Result<bool, Box<dyn std::error::Error>> {
    // Get the public key from KMS
    let public_key = manager.get_public_key().await?;

    // Verify signature (using your chosen crypto library)
    // This is a conceptual example
    // let valid = verify_rsa_signature(&public_key, data, signature)?;
    // Ok(valid)

    // For now, just return the capability
    Ok(!signature.is_empty())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup manager ...

    let data = b"Message to verify";
    let signature = manager.sign(data).await?;

    // Verify signature
    match verify_signature(&manager, data, &signature).await {
        Ok(true) => println!("Signature verified!"),
        Ok(false) => println!("Signature invalid!"),
        Err(e) => println!("Verification error: {}", e),
    }

    Ok(())
}
```

### 4. Error Handling Patterns

```rust
use knhk_sidecar::kms::{KmsConfig, KmsManager};
use knhk_sidecar::error::SidecarError;

async fn robust_sign(
    manager: &KmsManager,
    data: &[u8],
    max_retries: usize,
) -> Result<Vec<u8>, SidecarError> {
    let mut attempts = 0;

    loop {
        match manager.sign(data).await {
            Ok(sig) => return Ok(sig),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                eprintln!("Sign attempt {} failed: {}, retrying...", attempts, e);
                tokio::time::sleep(std::time::Duration::from_millis(100 * attempts as u64)).await;
            }
            Err(e) => return Err(e),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup manager ...

    match robust_sign(&manager, b"data", 3).await {
        Ok(sig) => println!("Signed: {}", hex::encode(&sig)),
        Err(e) => {
            match e {
                SidecarError::NetworkError { .. } => println!("Network error occurred"),
                SidecarError::ConfigError { .. } => println!("Configuration error"),
                _ => println!("Other error: {}", e),
            }
        }
    }

    Ok(())
}
```

### 5. Batch Signing

```rust
use knhk_sidecar::kms::KmsManager;

async fn batch_sign(
    manager: &KmsManager,
    messages: Vec<Vec<u8>>,
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut signatures = Vec::new();

    for message in messages {
        let signature = manager.sign(&message).await?;
        signatures.push(signature);
    }

    Ok(signatures)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup manager ...

    let messages = vec![
        b"Message 1".to_vec(),
        b"Message 2".to_vec(),
        b"Message 3".to_vec(),
    ];

    let signatures = batch_sign(&manager, messages).await?;

    for (i, sig) in signatures.iter().enumerate() {
        println!("Signature {}: {}", i + 1, hex::encode(sig));
    }

    Ok(())
}
```

### 6. Concurrent Operations

```rust
use knhk_sidecar::kms::KmsManager;
use tokio::task::JoinHandle;

async fn concurrent_sign(
    manager: std::sync::Arc<KmsManager>,
    data: Vec<u8>,
    count: usize,
) -> Result<Vec<Vec<u8>>, Box<dyn std::error::Error>> {
    let mut handles: Vec<JoinHandle<Result<Vec<u8>, Box<dyn std::error::Error>>>> = Vec::new();

    for _ in 0..count {
        let mgr = manager.clone();
        let d = data.clone();

        let handle = tokio::spawn(async move {
            Ok(mgr.sign(&d).await?.to_vec())
        });

        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        results.push(handle.await??);
    }

    Ok(results)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ... setup manager ...
    let manager = std::sync::Arc::new(manager);

    let signatures = concurrent_sign(
        manager,
        b"Concurrent signing test".to_vec(),
        10
    ).await?;

    println!("Generated {} signatures", signatures.len());

    Ok(())
}
```

## Configuration Examples

### Production Azure Setup

```rust
let config = KmsConfig::azure(
    "https://company-prod.vault.azure.net".to_string(),
    "prod-signing-key".to_string()
);

// With validation
if let Err(e) = config.validate() {
    eprintln!("Invalid configuration: {}", e);
    std::process::exit(1);
}
```

### Development Vault Setup

```rust
let config = KmsConfig::vault(
    "https://vault-dev.internal".to_string(),
    "transit".to_string(),
    "dev-app-key".to_string()
);

// Check rotation settings
println!("Auto rotation: {}", config.auto_rotation_enabled);
println!("Rotation interval: {} seconds", config.rotation_interval.as_secs());
```

### Testing AWS Setup

```rust
let config = KmsConfig::aws(
    "us-east-1".to_string(),
    "alias/test-signing-key".to_string() // Can use alias instead of ARN
);

// Validate before use
config.validate()?;
```

## Logging Examples

```rust
use tracing::info;

async fn sign_with_logging(
    manager: &KmsManager,
    data: &[u8],
) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    info!("Starting signature operation");
    let signature = manager.sign(data).await?;
    info!("Signature created: {} bytes", signature.len());
    Ok(signature)
}
```

## Key Takeaways

1. **Always validate configuration** before creating a manager
2. **Use environment variables** for sensitive data (tokens, credentials)
3. **Handle errors gracefully** - KMS operations can fail
4. **Implement retry logic** for transient failures
5. **Log important operations** with appropriate levels
6. **Test with all three providers** if supporting multiple KMS systems

## References

- [Azure Key Vault API Documentation](https://learn.microsoft.com/en-us/rest/api/keyvault/)
- [HashiCorp Vault Transit Engine](https://www.vaultproject.io/api-docs/secret/transit)
- [AWS KMS API Reference](https://docs.aws.amazon.com/kms/latest/APIReference/)
- [Rust async/await](https://doc.rust-lang.org/book/ch19-05-advanced-features.html)
- [tokio Runtime](https://tokio.rs/)
