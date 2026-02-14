#+#+#+#+
# Red Team Simulation (BAS MVP)

This repo is a work-in-progress **Breach & Attack Simulation (BAS)** dashboard and backend.

It is based on `Simulation 개발 문서.md` (the original plan uses Go + Gin), but the current MVP implementation is:

- Backend: Rust (`axum` + `sqlx` + SQLite)
- Agent: Rust simulated agent (no OS command execution)
- UI: Electron + React (Vite)

## Repository Layout

- `server/` Rust HTTP API + SQLite (agents, runs, events) + offline banner fingerprint matcher
- `sim_agent/` Rust simulated agent (registers, heartbeats, polls pending runs, posts results/events)
- `ui/` Electron + React dashboard (Agents / Runs / Events / Fingerprint)
- `Agent/` Rust Windows/LDAP prototype code (NOT used by the MVP; see safety note)
- `Banner_Scanner/` Rust scaffold for a scanner (currently incomplete)
- `Simulation 개발 문서.md` Original architecture/roadmap document

## Quickstart (Local MVP)

Prereqs:
- Rust + Cargo
- Node.js + npm

1) Start server

```bash
cd server
set FINGERPRINT_RULES_PATH=data\fingerprint_rules.sample.json
set SCENARIOS_PATH=data\scenarios
set LAUNCH_UI=1
cargo run
```

2) Start simulated agent

```bash
cd sim_agent
set SERVER_URL=http://127.0.0.1:3000
cargo run
```

3) Start UI

```bash
cd ui
npm install
set VITE_SERVER_URL=http://127.0.0.1:3000
npm run dev
```

Then start a Mission in the UI and open Run Detail to see PASS/FAIL + evidence.

## User Guide (UI)

1) Start a mission
- Open the UI and go to `Missions`
- Select:
  - Agent (Simulated Endpoint)
  - Scenario (shows `test_id` like `BAS-DEMO-001`)
- Click `Start Mission`

2) Pick a branch (if the mission has choices)
- Open `Runs` and click the newest run (or it will auto-select after starting)
- In `Run Detail`:
  - Use `Select: ...` buttons under `Allowlisted Actions` to choose a branch (choice_id)
  - Click `Refresh`
- Notes:
  - Some steps are "locked" until you pick a choice.
  - The simulated runner waits briefly for a branch choice before proceeding.

3) Read results
- `Steps`: per-step status
- `Verdict`: step PASS/FAIL + assertion statuses
- `Evidence`: evidence items (kind, locator, sha256)
- `Timeline`: append-only event timeline
- `Operator Log`: records action approvals and branch selections

## Data Model (MVP)

- Agent: registers to server; server stores metadata + `last_seen`.
- Run: unit of work; queued by UI; polled by agent; marked dispatched/completed.
- Event: append-only telemetry entries for UI timeline.
- Fingerprint: offline banner matching using local regex rules.

## API Overview

- `GET /api/scenarios` -> `[ScenarioMeta]`
- `GET /api/scenarios/:scenario_id` -> `ScenarioDef`
- `POST /api/agents/register` -> Agent JSON
- `POST /api/agents/:id/heartbeat` -> 200
- `GET /api/agents/list` -> `[Agent]`
- `GET /api/runs` -> `[Run]`
- `POST /api/runs` -> Run JSON (accepts `scenario_id` or `test_id`)
- `GET /api/runs/:run_id` -> Run JSON
- `GET /api/runs/:run_id/steps` -> `[Step]`
- `GET /api/runs/:run_id/events` -> `[Event]` (filtered)
- `GET /api/runs/:run_id/evidence` -> `[Evidence]`
- `GET /api/runs/:run_id/verdict` -> Run verdict view (step PASS/FAIL)
- `GET /api/runs/:run_id/operator-actions` -> `[OperatorAction]`
- `POST /api/runs/:run_id/operator-actions` -> 200
- `POST /api/evidence` -> Evidence JSON
- `GET /api/runs/pending/:agent_id` -> `[Run]` (server marks pending -> dispatched)
- `POST /api/runs/:run_id/result` -> 200
- `POST /api/events` -> Event JSON
- `GET /api/events` -> `[Event]`
- `POST /api/fingerprint/match` -> `{ candidates: [...] }`

See module READMEs for full request/response shapes.

## Tests

```bash
cd server && cargo test
cd ..\sim_agent && cargo test
cd ..\ui && npm run lint && npm run build
```

## Safety Note

This MVP intentionally does **not** implement remote OS command execution.

The `Agent/` crate contains Windows API + LDAP automation prototype code and includes placeholder credentials.
It is **not** used by the MVP pipeline (`server/` + `sim_agent/` + `ui/`).

## Merge Checklist

See `MERGE_CHECKLIST.md`.
