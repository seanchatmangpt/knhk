// rust/knhk-etl/src/emit.rs
// Stage 5: Emit - Actions (A) + Receipts → Lockchain + Downstream APIs

use alloc::collections::BTreeMap;
use alloc::format;
use alloc::string::String;
use alloc::string::ToString;
use alloc::vec::Vec;

use crate::reflex::{Action, Receipt, ReflexResult};
use crate::types::PipelineError;

/// Stage 5: Emit
/// Actions (A) + Receipts → Lockchain + Downstream APIs
#[cfg(feature = "std")]
pub struct EmitStage {
    pub lockchain_enabled: bool,
    pub downstream_endpoints: Vec<String>,
    max_retries: u32,
    retry_delay_ms: u64,
    lockchain: Option<knhk_lockchain::Lockchain>,
}

#[cfg(feature = "std")]
impl EmitStage {
    pub fn new(lockchain_enabled: bool, downstream_endpoints: Vec<String>) -> Self {
        Self {
            lockchain_enabled,
            downstream_endpoints,
            max_retries: 3,
            retry_delay_ms: 1000,
            lockchain: if lockchain_enabled {
                Some(knhk_lockchain::Lockchain::new())
            } else {
                None
            },
        }
    }
    
    pub fn with_git_repo(mut self, repo_path: String) -> Self {
        if self.lockchain_enabled {
            self.lockchain = Some(knhk_lockchain::Lockchain::with_git_repo(repo_path));
        }
        self
    }

    /// Emit actions and receipts
    /// 
    /// Production implementation:
    /// 1. Write receipts to lockchain (Merkle-linked)
    /// 2. Send actions to downstream APIs (webhooks, Kafka, gRPC)
    /// 3. Update metrics
    /// 4. Return final result
    pub fn emit(&self, input: ReflexResult) -> Result<EmitResult, PipelineError> {
        let mut receipts_written = 0;
        let mut actions_sent = 0;
        let mut lockchain_hashes = Vec::new();

        // Write receipts to lockchain
        if self.lockchain_enabled {
            // Use mutable lockchain reference
            let mut lockchain_ref = if let Some(ref lockchain) = self.lockchain {
                lockchain.clone()
            } else {
                return Err(PipelineError::EmitError(
                    "Lockchain enabled but not initialized".to_string()
                ));
            };
            
            for receipt in &input.receipts {
                match self.write_receipt_to_lockchain_with_lockchain(&mut lockchain_ref, receipt) {
                    Ok(hash) => {
                        receipts_written += 1;
                        lockchain_hashes.push(hash);
                    }
                    Err(e) => {
                        return Err(PipelineError::EmitError(
                            format!("Failed to write receipt {} to lockchain: {}", receipt.id, e)
                        ));
                    }
                }
            }
        }

        // Send actions to downstream endpoints
        for action in &input.actions {
            let mut success = false;
            let mut last_error = None;

            for endpoint in &self.downstream_endpoints {
                match self.send_action_to_endpoint(action, endpoint) {
                    Ok(_) => {
                        success = true;
                        actions_sent += 1;
                        break;
                    }
                    Err(e) => {
                        last_error = Some(e);
                    }
                }
            }

            if !success {
                // All endpoints failed
                return Err(PipelineError::EmitError(
                    format!("Failed to send action {} to all endpoints: {:?}", 
                        action.id, last_error)
                ));
            }
        }

        Ok(EmitResult {
            receipts_written,
            actions_sent,
            lockchain_hashes,
        })
    }

    /// Write receipt to lockchain (Merkle-linked) - with mutable lockchain reference
    fn write_receipt_to_lockchain_with_lockchain(
        &self,
        lockchain: &mut knhk_lockchain::Lockchain,
        receipt: &Receipt,
    ) -> Result<String, String> {
        use knhk_lockchain::{LockchainEntry, LockchainError};
        
        // Create lockchain entry
        let mut metadata = BTreeMap::new();
        metadata.insert("span_id".to_string(), receipt.span_id.to_string());
        metadata.insert("ticks".to_string(), receipt.ticks.to_string());
        metadata.insert("lanes".to_string(), receipt.lanes.to_string());
        metadata.insert("a_hash".to_string(), format!("{:016x}", receipt.a_hash));
        
        let entry = LockchainEntry {
            receipt_id: receipt.id.clone(),
            receipt_hash: [0; 32], // Will be computed by append
            parent_hash: None, // Will be linked by append
            timestamp_ms: Self::get_current_timestamp_ms(),
            metadata,
        };
        
        // Append to lockchain (computes hash and links to parent)
        match lockchain.append(entry) {
            Ok(hash) => Ok(hex::encode(&hash)),
            Err(e) => Err(format!("Failed to append receipt to lockchain: {:?}", e)),
        }
    }
    
    fn get_current_timestamp_ms() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    /// Send action to downstream endpoint
    fn send_action_to_endpoint(&self, action: &Action, endpoint: &str) -> Result<(), String> {
        // Validate endpoint format
        if endpoint.is_empty() {
            return Err("Endpoint URL cannot be empty".to_string());
        }

        // Determine endpoint type and send
        if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
            self.send_http_webhook(action, endpoint)
        } else if endpoint.starts_with("kafka://") {
            self.send_kafka_action(action, endpoint)
        } else if endpoint.starts_with("grpc://") {
            self.send_grpc_action(action, endpoint)
        } else {
            Err(format!("Unknown endpoint type: {}", endpoint))
        }
    }

    fn send_http_webhook(&self, action: &Action, endpoint: &str) -> Result<(), String> {
        use reqwest::blocking::Client;
        use std::time::Duration;
        
        // Create HTTP client with timeout
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;
        
        // Serialize action payload
        #[cfg(feature = "serde_json")]
        let payload = serde_json::json!({
            "id": action.id,
            "receipt_id": action.receipt_id,
            "payload": action.payload,
        });
        
        #[cfg(not(feature = "serde_json"))]
        let payload = alloc::format!(
            r#"{{"id":"{}","receipt_id":"{}","payload":[]}}"#,
            action.id, action.receipt_id
        );
        
        // Retry logic with exponential backoff
        let mut last_error = None;
        for attempt in 0..self.max_retries {
            let request = client.post(endpoint).header("Content-Type", "application/json");
            
            #[cfg(feature = "serde_json")]
            let request = request.json(&payload);
            
            #[cfg(not(feature = "serde_json"))]
            let request = request.body(payload.clone());
            
            match request.send() {
                Ok(response) => {
                    if response.status().is_success() {
                        return Ok(());
                    } else {
                        last_error = Some(format!("HTTP {}: {}", response.status(), response.status()));
                    }
                }
                Err(e) => {
                    last_error = Some(format!("HTTP request failed: {}", e));
                }
            }
            
            // Exponential backoff: wait before retry
            if attempt < self.max_retries - 1 {
                let delay_ms = self.retry_delay_ms * (1 << attempt); // 1s, 2s, 4s
                std::thread::sleep(Duration::from_millis(delay_ms));
            }
        }
        
        Err(format!("Failed to send action after {} retries: {}", 
                    self.max_retries, 
                    last_error.unwrap_or_else(|| "Unknown error".to_string())))
    }

    fn send_kafka_action(&self, action: &Action, endpoint: &str) -> Result<(), String> {
        // Parse Kafka endpoint: kafka://broker1:9092,broker2:9092/topic
        let endpoint = endpoint.strip_prefix("kafka://")
            .ok_or_else(|| "Invalid Kafka endpoint format".to_string())?;
        
        let (brokers, topic) = endpoint.split_once('/')
            .ok_or_else(|| "Kafka endpoint must include topic: kafka://brokers/topic".to_string())?;
        
        if brokers.is_empty() {
            return Err("Bootstrap servers cannot be empty".to_string());
        }
        
        if topic.is_empty() {
            return Err("Topic name cannot be empty".to_string());
        }
        
        #[cfg(feature = "kafka")]
        {
            use rdkafka::producer::{BaseProducer, BaseRecord};
            use rdkafka::ClientConfig;
            use std::time::Duration;
            
            // Create Kafka producer (blocking)
            let mut config = ClientConfig::new();
            config.set("bootstrap.servers", brokers);
            config.set("message.timeout.ms", "5000");
            config.set("queue.buffering.max.messages", "100000");
            
            let producer: BaseProducer = config.create()
                .map_err(|e| format!("Failed to create Kafka producer: {}", e))?;
            
            // Serialize action payload
            #[cfg(feature = "serde_json")]
            let payload = serde_json::json!({
                "id": action.id,
                "receipt_id": action.receipt_id,
                "payload": action.payload,
            }).to_string();
            
            #[cfg(not(feature = "serde_json"))]
            let payload = alloc::format!(
                r#"{{"id":"{}","receipt_id":"{}","payload":[]}}"#,
                action.id, action.receipt_id
            );
            
            // Send message to Kafka topic (blocking)
            let record = BaseRecord::to(topic)
                .key(&action.id)
                .payload(&payload);
            
            // Poll for delivery
            let mut last_error = None;
            for attempt in 0..self.max_retries {
                match producer.send(record) {
                    Ok(_) => {
                        // Poll for delivery confirmation
                        for _ in 0..50 {
                            producer.poll(Duration::from_millis(100));
                        }
                        producer.flush(Duration::from_secs(5));
                        return Ok(());
                    }
                    Err((e, _)) => {
                        last_error = Some(format!("Failed to send Kafka message: {}", e));
                    }
                }
                
                // Exponential backoff
                if attempt < self.max_retries - 1 {
                    let delay_ms = self.retry_delay_ms * (1 << attempt);
                    std::thread::sleep(Duration::from_millis(delay_ms));
                }
            }
            
            Err(format!("Failed to send action to Kafka after {} retries: {}", 
                self.max_retries, 
                last_error.unwrap_or_else(|| "Unknown error".to_string())))
        }
        
        #[cfg(not(feature = "kafka"))]
        {
            Err(format!("Kafka feature not enabled. Enable with 'kafka' feature: {}", endpoint))
        }
    }

    fn send_grpc_action(&self, action: &Action, endpoint: &str) -> Result<(), String> {
        // Parse gRPC endpoint: grpc://host:port/service/method
        let endpoint = endpoint.strip_prefix("grpc://").unwrap_or(endpoint);
        
        #[cfg(feature = "grpc")]
        {
            // gRPC requires async runtime - use HTTP POST to gRPC gateway as fallback
            // For blocking operation, convert gRPC endpoint to HTTP gateway endpoint
            let http_endpoint = if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                endpoint.to_string()
            } else {
                // Convert grpc://host:port/service/method to http://host:port/service/method
                format!("http://{}", endpoint)
            };
            
            // Use HTTP POST to gRPC gateway (enables blocking operation)
            self.send_http_webhook(action, &http_endpoint)
        }

        #[cfg(not(feature = "grpc"))]
        {
            // Fallback: use HTTP POST to gRPC gateway if available
            if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
                self.send_http_webhook(action, endpoint)
            } else {
                Err(format!("gRPC feature not enabled. Use HTTP gateway or enable 'grpc' feature: {}", endpoint))
            }
        }
    }

    /// Compute receipt hash for lockchain
    fn compute_receipt_hash(receipt: &Receipt) -> u64 {
        // Use FNV-1a hash for consistency
        const FNV_OFFSET_BASIS: u64 = 1469598103934665603;
        const FNV_PRIME: u64 = 1099511628211;

        let mut hash = FNV_OFFSET_BASIS;
        
        // Hash receipt fields
        let mut value = receipt.ticks as u64;
        for _ in 0..4 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }
        
        value = receipt.lanes as u64;
        for _ in 0..4 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }
        
        value = receipt.span_id;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }
        
        value = receipt.a_hash;
        for _ in 0..8 {
            hash ^= value & 0xFF;
            hash = hash.wrapping_mul(FNV_PRIME);
            value >>= 8;
        }
        
        hash
    }
}

pub struct EmitResult {
    pub receipts_written: usize,
    pub actions_sent: usize,
    pub lockchain_hashes: Vec<String>,
}

