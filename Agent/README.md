# Agent

Rust prototype crate (Windows-focused) that contains:

- Windows API / COM automation helpers
- LDAP (Active Directory) query helpers

This crate is **not used** by the current BAS MVP pipeline (`server/` + `sim_agent/` + `ui/`).

## Dependencies

From `Agent/Cargo.toml`:

- `windows` (Win32 APIs, COM, Shell, Threading, etc.)
- `ldap3` (LDAP client)
- `tokio` (async runtime)
- `regex`

## Code Layout

- `Agent/src/main.rs`
  - Local test harness / demo code invoking modules.
  - Contains placeholder credential strings for demonstration.
- `Agent/src/Module/`
  - `Executor/`
    - `COM.rs`: COM automation wrapper.
    - `SYSCALL.rs`: direct Win32 process creation utilities.
  - `Scanner/`
    - `Active_Directory.rs`: LDAP/AD query functions.

## Run / Test

Build:
```bash
cd Agent
cargo build
```

Run (local demo harness):
```bash
cargo run
```

Tests:
```bash
cargo test
```

Notes:
- Most functionality is Windows-specific.
- AD-related tests require a correctly configured AD/LDAP environment.
- Do not commit real credentials. Replace placeholders with environment-based configuration before any real use.

## Safety Note

This repository also contains a BAS MVP. The `Agent/` crate includes powerful OS/LDAP automation prototypes.
Keep it isolated from production systems and avoid distributing binaries without strict controls.
