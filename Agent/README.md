# Agent

Rust agent crate with real OS execution capabilities for the BAS platform.

## Executors

### Windows
- `COM` — COM automation (WScript.Shell, MMC20)
- `Syscall` — Direct Win32 process creation (CreateProcessW)
- `PowerShell` — PowerShell command execution
- `Registry` — Windows registry manipulation
- `Fileless` — In-memory execution via CreateThread

### Linux
- `Memfd` — Memory-only execution via memfd_create
- `RawSyscall` — Direct syscall invocation
- `Shell` — /bin/sh command execution

## Scanners
- **Port Scanner** — TCP connect scan (top ports)
- **Banner Grabber** — Service banner collection
- **AD/LDAP** — Active Directory reconnaissance

## Evasion
- Anti-analysis detection (debugger, VM, sandbox)
- String obfuscation (compile-time XOR)

## Transport
- HTTP transport with server API integration
- Server-driven step polling: agent fetches READY steps, executes, calls complete_step

## Build

Agent binary supports compile-time injection via environment variables:
- `AGENT_GUID` — Unique agent identifier
- `SERVER_URL` — C2 server URL (default: http://127.0.0.1:3000)
- `SLEEP_SEC` — Poll interval in seconds (default: 5)

```bash
cd Agent
set AGENT_GUID=my-agent-01
set SERVER_URL=http://127.0.0.1:3000
cargo build --release
```

## Run / Test

```bash
cargo run
cargo test
cargo clippy -- -D warnings
```

## Safety Note

This crate contains real OS execution capabilities. Keep it isolated from production systems.
