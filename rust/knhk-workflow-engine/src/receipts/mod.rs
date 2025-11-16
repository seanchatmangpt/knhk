//! Receipt System
//!
//! Cryptographic receipt generation and storage for all hook executions.
//! Provides immutable audit trail with query capabilities.

pub mod receipt_generator;
pub mod receipt_store;

pub use receipt_generator::{Receipt, ReceiptGenerator};
pub use receipt_store::{ReceiptQuery, ReceiptStats, ReceiptStore};
