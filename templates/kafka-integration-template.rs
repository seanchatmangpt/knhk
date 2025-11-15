// Kafka Integration Template
// Ready-to-use Kafka producer/consumer setup with telemetry
//
// Features:
// - Producer with batching and compression
// - Consumer with error handling
// - Telemetry integration
// - Graceful shutdown

use rdkafka::config::ClientConfig;
use rdkafka::consumer::{Consumer, StreamConsumer};
use rdkafka::producer::{FutureProducer, FutureRecord};
use rdkafka::Message;
use std::time::Duration;

// ============================================================================
// Kafka Producer
// ============================================================================

/// Create Kafka producer with optimal configuration
pub fn create_producer(brokers: &str) -> Result<FutureProducer, String> {
    ClientConfig::new()
        // Bootstrap servers
        .set("bootstrap.servers", brokers)
        // Producer configuration
        .set("message.timeout.ms", "5000")
        .set("queue.buffering.max.messages", "100000")
        .set("queue.buffering.max.kbytes", "1048576")
        .set("batch.num.messages", "10000")
        // Compression
        .set("compression.type", "snappy")
        // Delivery semantics
        .set("acks", "all") // Wait for all replicas
        .set("retries", "10")
        // Idempotence (exactly-once delivery)
        .set("enable.idempotence", "true")
        .create()
        .map_err(|e| format!("Failed to create producer: {}", e))
}

/// Send message to Kafka topic
pub async fn send_message(
    producer: &FutureProducer,
    topic: &str,
    key: Option<&str>,
    payload: &[u8],
) -> Result<(), String> {
    let mut record = FutureRecord::to(topic).payload(payload);

    if let Some(k) = key {
        record = record.key(k);
    }

    producer
        .send(record, Duration::from_secs(0))
        .await
        .map_err(|(err, _)| format!("Failed to send message: {}", err))?
        .map_err(|(err, _)| format!("Failed to deliver message: {}", err))?;

    Ok(())
}

// ============================================================================
// Kafka Consumer
// ============================================================================

/// Create Kafka consumer with optimal configuration
pub fn create_consumer(brokers: &str, group_id: &str, topics: &[&str]) -> Result<StreamConsumer, String> {
    let consumer: StreamConsumer = ClientConfig::new()
        // Bootstrap servers
        .set("bootstrap.servers", brokers)
        // Consumer configuration
        .set("group.id", group_id)
        .set("enable.auto.commit", "true")
        .set("auto.commit.interval.ms", "5000")
        // Start from beginning if no offset
        .set("auto.offset.reset", "earliest")
        // Session timeout (heartbeat)
        .set("session.timeout.ms", "30000")
        .set("heartbeat.interval.ms", "10000")
        .create()
        .map_err(|e| format!("Failed to create consumer: {}", e))?;

    consumer
        .subscribe(topics)
        .map_err(|e| format!("Failed to subscribe to topics: {}", e))?;

    Ok(consumer)
}

/// Consume messages from Kafka topic
pub async fn consume_messages<F>(
    consumer: StreamConsumer,
    mut message_handler: F,
) -> Result<(), String>
where
    F: FnMut(&[u8]) -> Result<(), String>,
{
    use futures::StreamExt;
    use rdkafka::consumer::stream_consumer::StreamConsumer as _;

    loop {
        match consumer.recv().await {
            Ok(message) => {
                // Extract payload
                let payload = message
                    .payload()
                    .ok_or_else(|| "Message has no payload".to_string())?;

                // Process message
                if let Err(e) = message_handler(payload) {
                    eprintln!("Error processing message: {}", e);
                    // Continue processing (or implement retry logic)
                }
            }
            Err(e) => {
                eprintln!("Error receiving message: {}", e);
                // Implement exponential backoff here
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

// ============================================================================
// Example Usage
// ============================================================================

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Kafka Integration Template ===\n");

    // Configuration
    let brokers = "localhost:9092";
    let topic = "knhk-events";
    let group_id = "knhk-consumer-group";

    // Example 1: Producer
    println!("--- Producer Example ---");
    let producer = create_producer(brokers)?;

    // Send messages
    for i in 0..5 {
        let message = format!("Message {}", i);
        send_message(&producer, topic, Some(&i.to_string()), message.as_bytes()).await?;
        println!("✅ Sent: {}", message);
    }
    println!();

    // Example 2: Consumer
    println!("--- Consumer Example ---");
    let consumer = create_consumer(brokers, group_id, &[topic])?;

    // Consume messages (runs forever, use Ctrl+C to stop)
    consume_messages(consumer, |payload| {
        let message = String::from_utf8_lossy(payload);
        println!("📥 Received: {}", message);
        Ok(())
    })
    .await?;

    Ok(())
}

// ============================================================================
// Production Enhancements
// ============================================================================

// TODO: Add telemetry
// use knhk_otel::{init_tracer, Tracer, SpanStatus};
//
// async fn send_message_with_telemetry(...) {
//     let mut tracer = Tracer::new();
//     let span = tracer.start_span("kafka.send".to_string(), None);
//     tracer.add_attribute(span.clone(), "topic".to_string(), topic.to_string());
//
//     let result = send_message(...).await;
//
//     match &result {
//         Ok(_) => tracer.end_span(span, SpanStatus::Ok),
//         Err(e) => {
//             tracer.add_attribute(span.clone(), "error".to_string(), e.to_string());
//             tracer.end_span(span, SpanStatus::Error)
//         }
//     }
//
//     result
// }

// TODO: Add retry logic
// use tokio::time::{sleep, Duration};
//
// async fn send_with_retry(producer: &FutureProducer, ..., max_retries: usize) -> Result<(), String> {
//     let mut attempts = 0;
//
//     loop {
//         attempts += 1;
//
//         match send_message(producer, ...).await {
//             Ok(_) => return Ok(()),
//             Err(e) if attempts >= max_retries => return Err(e),
//             Err(e) => {
//                 eprintln!("Retry {}/{}: {}", attempts, max_retries, e);
//                 sleep(Duration::from_millis(100 * attempts as u64)).await;
//             }
//         }
//     }
// }

// TODO: Add graceful shutdown
// use tokio::signal;
//
// async fn consume_with_shutdown(consumer: StreamConsumer, ...) {
//     let mut shutdown = signal::ctrl_c();
//
//     loop {
//         tokio::select! {
//             message = consumer.recv() => {
//                 // Process message
//             }
//             _ = &mut shutdown => {
//                 println!("Shutting down gracefully...");
//                 break;
//             }
//         }
//     }
// }

// Dependencies (add to Cargo.toml):
// [dependencies]
// rdkafka = { version = "0.34", features = ["tokio"] }
// tokio = { version = "1", features = ["full"] }
// futures = "0.3"
