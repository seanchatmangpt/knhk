# knhk-config

Configuration management for the KNHK framework.

## Features

- TOML-based configuration parsing
- JSON configuration support
- Type-safe configuration with serde
- Extensible configuration schema

## Usage

```rust
use knhk_config::Config;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::from_file("config.toml")?;

    // Access configuration values
    println!("Database URL: {}", config.database.url);

    Ok(())
}
```

## Configuration Format

```toml
[database]
url = "postgresql://localhost/knhk"

[server]
host = "0.0.0.0"
port = 8080
```

## License

Licensed under MIT license.
