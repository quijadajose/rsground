use rsground_runner::Runner;

use super::HELLO_WORLD_RS;

pub const CARGO_TOML: &str = r#"
[package]
name = "rsground-main"
version = "0.1.0"
edition = "2021"
"#;

pub fn cargo_init(runner: &Runner) {
    runner.create_file("Cargo.toml", CARGO_TOML).unwrap();
    runner.create_file("src/main.rs", HELLO_WORLD_RS).unwrap();
}
