// knhk-config v0.1.0 - Configuration Management
// TOML-based configuration with environment variable overrides

#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub mod config;
pub mod env;

pub use config::load_config;
pub use config::Config;
pub use env::apply_env_overrides;
pub use env::load_env_config;
