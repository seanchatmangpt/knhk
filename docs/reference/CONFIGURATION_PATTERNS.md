# Configuration Patterns

How to manage configuration safely in KNHK applications.

---

## 1. Environment-Based Configuration

### Use Environment Variables, Not Code

```rust
// ✅ GOOD: Configuration from environment
fn from_env() -> Result<Config, ConfigError> {
    let db_url = std::env::var("DATABASE_URL")
        .map_err(|_| ConfigError::MissingDatabaseUrl)?;
    let api_key = std::env::var("API_KEY")
        .map_err(|_| ConfigError::MissingApiKey)?;
    
    Ok(Config { db_url, api_key })
}

// ❌ BAD: Hardcoded configuration
fn hardcoded() -> Config {
    Config {
        db_url: "postgresql://localhost/mydb", // Never do this!
        api_key: "secret123",                  // Never do this!
    }
}
```

---

## 2. Configuration Validation

### Validate at Startup, Not at Use

```rust
pub struct Config {
    database_url: Url,      // Already validated
    port: NonZeroU16,       // Cannot be zero
    timeout_ms: u64,        // ≥ 0
    max_connections: NonZeroU32,  // Cannot be zero
}

impl TryFrom<RawConfig> for Config {
    type Error = ConfigError;
    
    fn try_from(raw: RawConfig) -> Result<Self, Self::Error> {
        // Validate all fields at construction
        let database_url = Url::parse(&raw.database_url)
            .map_err(ConfigError::InvalidDatabaseUrl)?;
        
        let port = NonZeroU16::new(raw.port)
            .ok_or(ConfigError::InvalidPort)?;
        
        let max_connections = NonZeroU32::new(raw.max_connections)
            .ok_or(ConfigError::InvalidMaxConnections)?;
        
        Ok(Config {
            database_url,
            port,
            timeout_ms: raw.timeout_ms,
            max_connections,
        })
    }
}
```

**Benefits**:
- Errors caught at startup, not at runtime
- No validation checks needed in code
- Type system enforces validity

---

## 3. Configuration Hierarchies

### Default → Environment → File → Arguments

```rust
fn load_config() -> Result<Config, ConfigError> {
    // 1. Start with defaults
    let mut config = Config::default();
    
    // 2. Override with config file (if exists)
    if let Ok(file_config) = load_from_file("config.yaml") {
        config.merge(file_config);
    }
    
    // 3. Override with environment variables
    config.merge_from_env();
    
    // 4. Override with command-line arguments
    config.merge_from_args();
    
    // 5. Validate final configuration
    config.validate()?;
    
    Ok(config)
}
```

**Priority**: Arguments > Env > File > Defaults

---

## 4. Secrets Management

### Never Hardcode Secrets

```rust
// ❌ WRONG: Secrets in code
const API_KEY: &str = "sk-1234567890";
const DATABASE_PASSWORD: &str = "MyPassword123";

// ✅ CORRECT: Secrets from environment
pub fn get_api_key() -> Result<String, ConfigError> {
    std::env::var("API_KEY")
        .map_err(|_| ConfigError::MissingApiKey)
}

pub fn get_database_password() -> Result<String, ConfigError> {
    std::env::var("DATABASE_PASSWORD")
        .map_err(|_| ConfigError::MissingDatabasePassword)
}
```

**For Production**:
- Use Vault (HashiCorp)
- Use Secrets Manager (AWS/Azure/GCP)
- Use Key Management Service (KMS)
- Rotate secrets regularly

---

## 5. Feature Flags

### Runtime Feature Control

```rust
pub struct FeatureFlags {
    enable_new_algorithm: bool,
    enable_beta_features: bool,
    enable_debug_logging: bool,
}

impl FeatureFlags {
    pub fn from_env() -> Self {
        FeatureFlags {
            enable_new_algorithm: std::env::var("FEATURE_NEW_ALGORITHM")
                .map(|v| v == "true")
                .unwrap_or(false),
            enable_beta_features: std::env::var("FEATURE_BETA")
                .map(|v| v == "true")
                .unwrap_or(false),
            enable_debug_logging: std::env::var("DEBUG_LOGGING")
                .map(|v| v == "true")
                .unwrap_or(false),
        }
    }
}

// Use in code
fn process_workflow(flags: &FeatureFlags) {
    if flags.enable_new_algorithm {
        use_new_algorithm()
    } else {
        use_legacy_algorithm()
    }
}
```

---

## 6. Environment-Specific Configuration

### Different Settings Per Environment

```
config/
├── default.yaml      # Base defaults
├── development.yaml  # Dev overrides
├── staging.yaml      # Staging overrides
└── production.yaml   # Prod overrides
```

**Load by environment**:

```rust
fn load_config_for_env(env: &str) -> Result<Config, ConfigError> {
    // Load defaults
    let mut config = load_file("config/default.yaml")?;
    
    // Override with environment-specific
    let env_config = load_file(&format!("config/{}.yaml", env))?;
    config.merge(env_config);
    
    // Override with env vars
    config.merge_from_env();
    
    Ok(config)
}
```

---

## 7. Configuration Audit Trail

### Log Configuration Changes

```rust
pub struct ConfigAudit {
    timestamp: DateTime<Utc>,
    changed_by: String,
    changes: Vec<ConfigChange>,
}

pub enum ConfigChange {
    Set { key: String, old_value: String, new_value: String },
    Removed { key: String },
    Added { key: String, value: String },
}

impl Config {
    pub fn with_audit(&mut self, change: ConfigChange) {
        // Log audit entry
        log::info!("Configuration changed: {:?}", change);
        // Trigger notification
        notify_admins(&change);
    }
}
```

---

## 8. Configuration Versioning

### Handle Config Format Changes

```rust
pub struct ConfigV2 {
    database_url: String,
    port: u16,
    timeout_ms: u64,
}

pub struct ConfigV3 {
    database: DatabaseConfig,  // Nested
    server: ServerConfig,      // Reorganized
    timeouts: TimeoutConfig,   // New section
}

impl From<ConfigV2> for ConfigV3 {
    fn from(v2: ConfigV2) -> Self {
        ConfigV3 {
            database: DatabaseConfig {
                url: v2.database_url,
            },
            server: ServerConfig {
                port: v2.port,
            },
            timeouts: TimeoutConfig {
                default_ms: v2.timeout_ms,
            },
        }
    }
}
```

---

## Checklist

- [ ] No hardcoded secrets in code
- [ ] Configuration validated at startup
- [ ] Environment variables used for secrets
- [ ] Invalid configs fail fast at startup
- [ ] Type system prevents invalid values
- [ ] Feature flags for gradual rollout
- [ ] Configuration audit logging enabled
- [ ] Different configs per environment
- [ ] Documentation of all config options
- [ ] Configuration version management

---

**Last Updated**: 2025-11-15
**Version**: v1.1.0
**Framework**: Configuration Patterns
