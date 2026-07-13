default:
    @just --list

connect path:
    cargo run --example cli -- {{path}}

dev log_level='trace':
    RUST_LOG={{log_level}} cargo run -- --dev
