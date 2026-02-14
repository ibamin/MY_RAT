# Banner_Scanner

Rust scaffold for a banner/service scanner module.

## Current State

- `Banner_Scanner/src/main.rs` is currently empty in this repo snapshot.
- There is an `out.jsonl` artifact in `Banner_Scanner/` that appears to be generated output.

## Dependencies

From `Banner_Scanner/Cargo.toml`:

- CLI / UX: `clap`, `console`, `atty`, `anyhow`
- Async: `tokio`
- TLS: `native-tls`, `tokio-native-tls`
- Parsing: `quick-xml`, `regex`
- Serialization: `serde`, `serde_json`
- Time: `chrono`

## Intended I/O (Planned)

This crate is expected to:

- Input: target endpoints (host/port) and/or raw banners
- Output: structured records (e.g., JSONL) containing observed banners and derived metadata

The BAS MVP currently performs **offline** fingerprinting in `server/` using regex rules.

## Run / Test

```bash
cd Banner_Scanner
cargo build
cargo test
```
