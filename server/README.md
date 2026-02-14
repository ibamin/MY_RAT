# server

Rust backend for the BAS MVP.

## What It Does

- Stores `agents`, `runs`, and `events` in SQLite.
- Exposes HTTP APIs used by `ui/` (dashboard) and `sim_agent/` (simulated agent).
- Provides an **offline** banner fingerprint matcher (regex rules loaded from JSON).

## Dependencies

From `server/Cargo.toml`:

- Web: `axum`, `tokio`, `tower-http` (CORS, trace)
- DB: `sqlx` (SQLite)
- Observability: `tracing`, `tracing-subscriber`
- Fingerprint matching: `regex`, `serde`, `serde_json`

## Configuration (Environment Variables)

- `DATABASE_URL`
  - Default: `sqlite:red-sim.db`
  - Side-effect: creates a SQLite file `red-sim.db` in the `server/` working directory.
- `FINGERPRINT_RULES_PATH`
  - Optional
  - Path to a JSON file containing fingerprint rules.
  - If not set (or load fails), the matcher is empty (no candidates).

Sample rules file: `server/data/fingerprint_rules.sample.json`

## Database Schema

Tables are created automatically at startup (see `server/src/db.rs`).

All IDs are stored as `TEXT` and are UUID strings.
All timestamps are stored as `TEXT` in RFC3339 format.

### agents

Columns:
- `id` (TEXT, PK)
- `hostname` (TEXT)
- `ip` (TEXT)
- `os` (TEXT)
- `arch` (TEXT)
- `user` (TEXT)
- `last_seen` (TEXT RFC3339)
- `status` (TEXT)

### runs

Columns:
- `id` (TEXT, PK)
- `agent_id` (TEXT)
- `test_id` (TEXT)
- `params_json` (TEXT nullable)
- `status` (TEXT)
- `result_json` (TEXT nullable)
- `created_at` (TEXT RFC3339)
- `updated_at` (TEXT RFC3339)

### events

Columns:
- `id` (TEXT, PK)
- `run_id` (TEXT nullable)
- `agent_id` (TEXT nullable)
- `level` (TEXT)
- `message` (TEXT)
- `ts` (TEXT RFC3339)

## HTTP API

Routes are wired in `server/src/main.rs` and implemented in `server/src/handlers.rs`.

### POST /api/agents/register

Input JSON:
```json
{
  "hostname": "host-a",
  "ip": "127.0.0.1",
  "os": "windows",
  "arch": "x86_64",
  "user": "alice"
}
```

Output JSON (Agent):
```json
{
  "id": "<uuid>",
  "hostname": "host-a",
  "ip": "127.0.0.1",
  "os": "windows",
  "arch": "x86_64",
  "user": "alice",
  "last_seen": "2026-01-01T00:00:00Z",
  "status": "online"
}
```

Side-effects:
- Inserts a row into `agents`.

### POST /api/agents/:id/heartbeat

Path params:
- `id`: agent id (UUID string)

Output:
- `200 OK`

Side-effects:
- Updates `agents.last_seen` and sets status to `online`.

### GET /api/agents/list

Output:
- JSON array of Agents.

### POST /api/runs

Input JSON:
```json
{
  "agent_id": "<agent uuid>",
  "test_id": "BAS-DEMO-001",
  "params_json": "{\"target\":\"host-a\"}"
}
```

Output JSON (Run):
```json
{
  "id": "<uuid>",
  "agent_id": "<agent uuid>",
  "test_id": "BAS-DEMO-001",
  "params_json": "{\"target\":\"host-a\"}",
  "status": "pending",
  "result_json": null,
  "created_at": "2026-01-01T00:00:00Z",
  "updated_at": "2026-01-01T00:00:00Z"
}
```

Side-effects:
- Inserts a row into `runs`.

### GET /api/runs/pending/:agent_id

Path params:
- `agent_id`: agent id (UUID string)

Output:
- JSON array of Runs with `status == pending`.

Side-effects:
- If any pending runs exist for the agent, server marks them as `dispatched`.

### POST /api/runs/:run_id/result

Input JSON:
```json
{
  "status": "completed",
  "result_json": "{\"ok\":true}"
}
```

Output:
- `200 OK`

Side-effects:
- Updates `runs.status`, `runs.result_json`, `runs.updated_at`.

### POST /api/events

Input JSON:
```json
{
  "run_id": "<uuid>",
  "agent_id": "<uuid>",
  "level": "info",
  "message": "run_start test_id=BAS-DEMO-001"
}
```

Output JSON (Event):
```json
{
  "id": "<uuid>",
  "run_id": "<uuid>",
  "agent_id": "<uuid>",
  "level": "info",
  "message": "...",
  "ts": "2026-01-01T00:00:00Z"
}
```

Side-effects:
- Inserts a row into `events`.

### GET /api/events

Output:
- JSON array of up to 200 events ordered by newest first.

### POST /api/fingerprint/match

Input JSON:
```json
{
  "banner": "Server: nginx/1.24.0\r\n",
  "limit": 10
}
```

Output JSON:
```json
{
  "candidates": [
    {
      "service": "http",
      "product": "nginx",
      "version": "1.24.0",
      "confidence": 0.85
    }
  ]
}
```

Side-effects:
- None (offline match only).

## Run / Test

```bash
cd server
cargo run
cargo test
cargo clippy -- -D warnings
```
