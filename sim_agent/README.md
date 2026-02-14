# sim_agent

Rust **simulated agent** for the BAS MVP.

It does not run OS commands. It emulates an agent lifecycle:

- register to server
- send heartbeat periodically
- poll for pending runs
- post a simulated result
- emit events for the UI timeline

## Dependencies

From `sim_agent/Cargo.toml`:

- HTTP: `reqwest` (JSON)
- Runtime: `tokio`
- Serialization: `serde`, `serde_json`
- IDs: `uuid`

## Configuration (Environment Variables)

- `SERVER_URL`
  - Default: `http://127.0.0.1:3000`

Agent metadata sent during registration:
- `AGENT_HOSTNAME` (default: `COMPUTERNAME`/`HOSTNAME` or `sim-agent`)
- `AGENT_IP` (default: `127.0.0.1`)
- `AGENT_OS` (default: `std::env::consts::OS`)
- `AGENT_ARCH` (default: `std::env::consts::ARCH`)
- `AGENT_USER` (default: `USERNAME`/`USER` or `unknown`)

## Inputs / Outputs

### Input (server -> agent)

The agent polls:

- `GET /api/runs/pending/:agent_id`

The server responds with a JSON array of Runs. The agent uses:

- `id` (run id)
- `test_id`
- `params_json` (optional string)

### Output (agent -> server)

The agent calls:

- `POST /api/agents/:id/heartbeat` (no body)
- `POST /api/runs/:run_id/result`
  - body: `{ "status": "completed", "result_json": "..." }`
- `POST /api/events`
  - emits `run_start ...` and `run_done`

The simulated result payload is a JSON string.

## Run / Test

```bash
cd sim_agent
set SERVER_URL=http://127.0.0.1:3000
cargo run
```

```bash
cargo test
cargo clippy -- -D warnings
```
