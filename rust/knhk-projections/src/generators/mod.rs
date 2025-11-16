//! Projection generators - Transform Σ snapshots into various artifacts

pub mod hooks;
pub mod markdown;
pub mod openapi;
pub mod otel;
pub mod rust_models;

pub use hooks::{HooksGenerator, HooksOutput};
pub use markdown::{MarkdownGenerator, MarkdownOutput};
pub use openapi::{OpenApiGenerator, OpenApiOutput};
pub use otel::{OtelGenerator, OtelOutput};
pub use rust_models::{RustModelsGenerator, RustModelsOutput};
