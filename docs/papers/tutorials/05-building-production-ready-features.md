# Tutorial: Building Production-Ready Features

**Level**: Advanced
**Time**: 45-60 minutes
**Learning Objectives**: End-to-end feature development for production deployment

## What You'll Learn

By the end of this tutorial, you'll understand:
- Feature lifecycle from concept to production
- Complete development workflow
- Production-readiness checklist
- Error handling and resilience
- Monitoring and observability
- Deployment considerations

## Prerequisites

- Completed: [Chicago TDD Basics](03-chicago-tdd-basics.md)
- Completed: [Optimizing Performance](04-optimizing-performance.md)
- Production experience helpful
- ~60 minutes

## Phase 1: Conception & Planning

### Define Requirements

```rust
// Feature specification
/*
FEATURE: User Authentication Service
GOAL: Provide secure user authentication with token-based sessions
INPUT: Username, password
OUTPUT: AuthToken with expiry
CONSTRAINTS:
  - Performance: ≤8 ticks hot path
  - Security: Password hashing with argon2
  - Reliability: 99.9% uptime target
  - Observability: Full telemetry coverage
*/
```

### Create Design Document

```yaml
Feature: Authentication
Components:
  - Credential validator
  - Token generator
  - Session manager
  - Rate limiter

Interfaces:
  - fn authenticate(user: &str, pass: &str) -> Result<Token>
  - fn validate_token(token: &str) -> Result<Session>
  - fn refresh_token(token: &str) -> Result<Token>

Error Handling:
  - InvalidCredentials
  - TokenExpired
  - RateLimitExceeded
  - InternalError
```

## Phase 2: Test-Driven Development

### Write Comprehensive Tests

```rust
#[cfg(test)]
mod auth_tests {
    use super::*;

    // Happy path
    #[test]
    fn test_authenticate_valid_credentials() {
        let result = authenticate("user", "pass");
        assert!(result.is_ok());
        let token = result.unwrap();
        assert!(!token.value.is_empty());
    }

    // Error cases
    #[test]
    fn test_authenticate_invalid_credentials() {
        let result = authenticate("user", "wrong");
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AuthError::InvalidCredentials);
    }

    #[test]
    fn test_token_expiry() {
        let token = create_test_token();
        let expired = token.is_expired();
        assert!(expired);
    }

    #[test]
    fn test_rate_limiting() {
        for _ in 0..10 {
            let _ = authenticate("user", "pass");
        }
        let result = authenticate("user", "pass");
        assert_eq!(result.unwrap_err(), AuthError::RateLimitExceeded);
    }

    #[test]
    fn test_session_refresh() {
        let token = authenticate("user", "pass").unwrap();
        let new_token = refresh_token(&token).unwrap();
        assert_ne!(token.value, new_token.value);
    }

    // Performance test
    #[test]
    fn test_performance_meets_chatman_constant() {
        let start = std::time::Instant::now();
        let _ = authenticate("user", "pass");
        let elapsed = start.elapsed().as_nanos();
        assert!(elapsed <= 8, "Exceeded Chatman Constant: {}", elapsed);
    }
}
```

## Phase 3: Implementation

### Implement Core Logic

```rust
use tracing::instrument;
use std::time::{SystemTime, Duration};

#[instrument(skip(password))]
pub fn authenticate(username: &str, password: &str) -> Result<AuthToken> {
    info!("Authentication attempt for user: {}", username);

    // Validate input
    if username.is_empty() {
        warn!("Empty username");
        return Err(AuthError::InvalidCredentials);
    }

    // Check rate limit
    if is_rate_limited(username) {
        warn!("Rate limit exceeded for user: {}", username);
        return Err(AuthError::RateLimitExceeded);
    }

    // Verify credentials
    match verify_credentials(username, password) {
        Ok(user_id) => {
            info!(user_id = user_id, "Credentials verified");

            // Create token
            let token = AuthToken {
                value: generate_secure_token(),
                user_id,
                issued_at: SystemTime::now(),
                expires_at: SystemTime::now() + Duration::from_secs(3600),
            };

            info!(user_id = user_id, "Token created successfully");
            Ok(token)
        }
        Err(e) => {
            warn!("Authentication failed: {}", e);
            Err(AuthError::InvalidCredentials)
        }
    }
}

#[instrument]
pub fn validate_token(token: &str) -> Result<Session> {
    // Decode token
    let claims = decode_token(token)?;

    // Check expiry
    if claims.expires_at < SystemTime::now() {
        warn!("Token expired");
        return Err(AuthError::TokenExpired);
    }

    info!(user_id = claims.user_id, "Token valid");
    Ok(Session {
        user_id: claims.user_id,
        issued_at: claims.issued_at,
        expires_at: claims.expires_at,
    })
}
```

## Phase 4: Telemetry & Observability

### Add Complete Instrumentation

```rust
#[instrument(skip(password))]
pub fn authenticate(username: &str, password: &str) -> Result<AuthToken> {
    let start = std::time::Instant::now();

    info!("Authentication attempt", username = username);

    // Metrics
    ATTEMPT_COUNTER.add(1, &[]);

    match verify_credentials(username, password) {
        Ok(user_id) => {
            let elapsed = start.elapsed();

            info!(
                user_id = user_id,
                duration_ms = elapsed.as_millis(),
                "Authentication successful"
            );

            DURATION_HISTOGRAM.record(elapsed.as_millis() as u64, &[]);
            SUCCESS_COUNTER.add(1, &[]);

            create_token(user_id)
        }
        Err(e) => {
            let elapsed = start.elapsed();

            warn!(
                error = %e,
                duration_ms = elapsed.as_millis(),
                "Authentication failed"
            );

            FAILURE_COUNTER.add(1, &[]);
            Err(AuthError::InvalidCredentials)
        }
    }
}
```

### Create OTel Schema

```yaml
instrumentation:
  name: authentication_service
  version: 1.0.0
  description: "User authentication service"

spans:
  - name: authenticate
    description: "User authentication"
    attributes:
      - username:
          type: string
          description: "Username attempting to authenticate"
      - user_id:
          type: int
          description: "User ID after successful auth"
      - duration_ms:
          type: int
          description: "Duration in milliseconds"
    events:
      - name: "Authentication successful"
      - name: "Authentication failed"
        attributes:
          - error:
              type: string

  - name: validate_token
    description: "Token validation"
    attributes:
      - user_id:
          type: int
      - valid:
          type: bool

metrics:
  - name: auth_attempts
    type: counter
    unit: "1"
    description: "Total authentication attempts"

  - name: auth_success
    type: counter
    unit: "1"
    description: "Successful authentications"

  - name: auth_failure
    type: counter
    unit: "1"
    description: "Failed authentications"
    attributes:
      - error_type:
          type: string

  - name: auth_duration
    type: histogram
    unit: "ms"
    description: "Authentication duration"

logs:
  - name: auth_attempt
    level: INFO
    description: "Authentication attempt started"
    attributes:
      - username:
          type: string

  - name: auth_failure
    level: WARN
    description: "Authentication failed"
    attributes:
      - reason:
          type: string
```

## Phase 5: Error Handling

### Comprehensive Error Strategy

```rust
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum AuthError {
    InvalidCredentials,
    TokenExpired,
    RateLimitExceeded,
    InternalError(String),
    InvalidToken,
    UserNotFound,
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidCredentials => write!(f, "Invalid credentials"),
            Self::TokenExpired => write!(f, "Token has expired"),
            Self::RateLimitExceeded => write!(f, "Rate limit exceeded"),
            Self::InternalError(e) => write!(f, "Internal error: {}", e),
            Self::InvalidToken => write!(f, "Invalid token format"),
            Self::UserNotFound => write!(f, "User not found"),
        }
    }
}

impl std::error::Error for AuthError {}

#[instrument]
pub fn authenticate_safe(username: &str, password: &str) -> Result<AuthToken> {
    authenticate(username, password)
        .map_err(|e| {
            match e {
                AuthError::InvalidCredentials => {
                    warn!("Invalid credentials for user: {}", username);
                    e
                }
                AuthError::RateLimitExceeded => {
                    error!("Rate limit exceeded for user: {}", username);
                    e
                }
                AuthError::InternalError(ref msg) => {
                    error!("Internal error: {}", msg);
                    e
                }
                _ => e,
            }
        })
}
```

## Phase 6: Security Hardening

### Add Security Controls

```rust
use std::collections::HashMap;
use std::time::{SystemTime, Duration};

struct RateLimiter {
    attempts: HashMap<String, Vec<SystemTime>>,
    max_attempts: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn is_limited(&mut self, username: &str) -> bool {
        let now = SystemTime::now();
        let cutoff = now - self.window;

        let attempts = self.attempts.entry(username.to_string())
            .or_insert_with(Vec::new);

        // Remove old attempts
        attempts.retain(|&time| time > cutoff);

        if attempts.len() >= self.max_attempts {
            warn!(
                username = username,
                count = attempts.len(),
                "Rate limit exceeded"
            );
            return true;
        }

        attempts.push(now);
        false
    }
}

// Password hashing
#[instrument(skip(password))]
fn hash_password(password: &str) -> String {
    use argon2::{Argon2, PasswordHasher};
    use argon2::password_hash::SaltString;

    let salt = SaltString::generate(rand::thread_rng());
    let argon2 = Argon2::default();

    argon2.hash_password(password.as_bytes(), &salt)
        .expect("Failed to hash password")
        .to_string()
}

#[instrument(skip(password))]
fn verify_password(password: &str, hash: &str) -> bool {
    use argon2::{Argon2, PasswordHash, PasswordVerifier};

    let parsed_hash = PasswordHash::new(hash)
        .expect("Invalid hash");

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok()
}
```

## Phase 7: Monitoring & Metrics

### Add Production Monitoring

```rust
use prometheus::{Counter, Histogram, Registry};

lazy_static::lazy_static! {
    pub static ref AUTH_ATTEMPTS: Counter =
        Counter::new("auth_attempts_total", "Total auth attempts").unwrap();

    pub static ref AUTH_SUCCESS: Counter =
        Counter::new("auth_success_total", "Successful authentications").unwrap();

    pub static ref AUTH_FAILURES: Counter =
        Counter::new("auth_failures_total", "Failed authentications").unwrap();

    pub static ref AUTH_DURATION: Histogram =
        Histogram::new("auth_duration_ms", "Auth duration in ms")
            .unwrap()
            .linear_buckets(0.0, 10.0, 10)
            .unwrap();
}

#[instrument]
pub fn authenticate_monitored(username: &str, password: &str) -> Result<AuthToken> {
    let start = std::time::Instant::now();
    AUTH_ATTEMPTS.inc();

    match authenticate(username, password) {
        Ok(token) => {
            AUTH_SUCCESS.inc();
            let elapsed_ms = start.elapsed().as_millis() as f64;
            AUTH_DURATION.observe(elapsed_ms);
            info!("Auth successful");
            Ok(token)
        }
        Err(e) => {
            AUTH_FAILURES.inc();
            warn!("Auth failed: {}", e);
            Err(e)
        }
    }
}
```

## Phase 8: Testing for Production

### Add Integration Tests

```rust
#[tokio::test]
async fn test_full_authentication_flow() {
    // Setup
    let db = setup_test_db().await;
    let user = create_test_user(&db).await;

    // Authenticate
    let token = authenticate(user.username, user.password)
        .expect("Auth failed");

    // Validate token
    let session = validate_token(&token.value)
        .expect("Token validation failed");

    assert_eq!(session.user_id, user.id);

    // Refresh token
    let new_token = refresh_token(&token.value)
        .expect("Token refresh failed");

    assert_ne!(token.value, new_token.value);

    // Verify old token still works
    let session2 = validate_token(&new_token.value)
        .expect("New token validation failed");

    assert_eq!(session2.user_id, user.id);
}

#[tokio::test]
async fn test_concurrent_authentication() {
    let tasks: Vec<_> = (0..100)
        .map(|i| {
            tokio::spawn(async move {
                authenticate(&format!("user{}", i), "password")
            })
        })
        .collect();

    let results = futures::future::join_all(tasks).await;
    assert!(results.iter().all(|r| r.is_ok()));
}
```

## Phase 9: Documentation & Deployment

### Create API Documentation

```rust
/// Authenticates a user with credentials
///
/// # Arguments
/// * `username` - The username
/// * `password` - The password (not stored)
///
/// # Returns
/// * `Ok(AuthToken)` - Valid authentication token
/// * `Err(AuthError)` - Authentication failed
///
/// # Performance
/// Meeting Chatman Constant: ≤8 ticks hot path
///
/// # Security
/// - Password verified with argon2
/// - Rate limiting applied per user
/// - Token signed with HMAC-SHA256
///
/// # Example
/// ```
/// let token = authenticate("alice", "secret")?;
/// println!("Token: {}", token.value);
/// ```
#[instrument(skip(password))]
pub fn authenticate(username: &str, password: &str) -> Result<AuthToken> {
    // Implementation
}
```

### Production Checklist

```markdown
## Pre-Production Checklist

### Code Quality
- [ ] All tests passing (`cargo test --workspace`)
- [ ] No clippy warnings (`cargo clippy --workspace -- -D warnings`)
- [ ] Code formatted (`cargo fmt --all`)
- [ ] No unsafe blocks (or justified)
- [ ] Error handling complete

### Performance
- [ ] Performance tests pass (`make test-performance-v04`)
- [ ] Hot paths meet ≤8 tick requirement
- [ ] No memory leaks (profiled)
- [ ] Concurrent load tested (100+ concurrent)

### Security
- [ ] No hardcoded secrets
- [ ] Dependencies audited (`cargo audit`)
- [ ] Input validation complete
- [ ] OWASP top 10 addressed

### Observability
- [ ] All spans documented (`weaver registry check`)
- [ ] Metrics exposed (`prometheus` format)
- [ ] Error logging comprehensive
- [ ] Performance metrics tracked

### Documentation
- [ ] API documented (rustdoc)
- [ ] Example usage provided
- [ ] Deployment guide written
- [ ] Troubleshooting guide created

### Deployment
- [ ] Docker image builds (`docker build`)
- [ ] Health checks implemented
- [ ] Rollback plan documented
- [ ] Monitoring alerts configured
```

## What You've Learned

Congratulations! You now understand:

1. **Feature Lifecycle** - Conception to production
2. **Complete Development Workflow** - TDD through deployment
3. **Production-Ready Code** - Security, reliability, observability
4. **Error Handling** - Comprehensive error strategies
5. **Monitoring & Observability** - Metrics and telemetry
6. **Testing for Production** - Integration and load testing

## Next Steps

- **Deploy to production**: Consult deployment guides
- **Monitor in production**: Set up alerting
- **Iterate based on metrics**: Use telemetry for improvements
- **Advanced features**: Implement feedback

## Key Takeaways

✅ **Requirements → Tests → Implementation → Telemetry → Deployment**
✅ **Security and performance from day one**
✅ **Comprehensive testing at every phase**
✅ **Full observability for production monitoring**
✅ **Documentation drives understanding**

---

**You are here**: Tutorial (Learning-oriented)
**Framework**: Diátaxis
**Tutorial Duration**: ~60 minutes
**Difficulty**: Advanced
**Prerequisites**: Chicago TDD, Performance Optimization
