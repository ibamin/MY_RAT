# Shadow Protocol — Breach & Attack Simulation (BAS)

Breach & Attack Simulation platform with real agent execution, server-driven step control, and a game-style tactical UI for security education.

## Architecture

```
┌──────────────┐       HTTP/JSON        ┌──────────────┐
│   Agent       │ ◄──────────────────►  │   Server     │
│  (Rust)       │  register / heartbeat │  (Rust/Axum) │
│  Windows +    │  poll runs / results  │  SQLite DB   │
│  Linux        │  evidence / events    │  Scenarios   │
└──────────────┘                        └──────┬───────┘
                                               │
                                        REST API (JSON)
                                               │
                                        ┌──────┴───────┐
                                        │     UI       │
                                        │  Electron +  │
                                        │  React/Vite  │
                                        └──────────────┘
```

| Component | Stack | Description |
|-----------|-------|-------------|
| **Server** | Rust, Axum, SQLite (sqlx) | REST API, scenario engine, achievement system, AI script generator |
| **Agent** | Rust (cross-platform) | Real OS executors, scanner, evasion modules |
| **UI** | Electron, React 19, Vite, Framer Motion | Game-style tactical interface |

## Prerequisites

- **Rust** 1.85+ with Cargo
- **Node.js** 18+ with npm
- (Optional) **Electron** — included via npm

## Quickstart

### 1. Start the Server

```bash
cd server
cargo run
```

The server creates `red-sim.db` (SQLite) automatically on first run and listens on `http://127.0.0.1:3000`.

### 2. Start the Agent

```bash
cd Agent
cargo run
```

The agent registers with the server, sends heartbeats, and polls for pending runs.

### 3. Start the UI

```bash
cd ui
npm install
npm run dev
```

This starts both Vite dev server (`:5173`) and Electron. Open the Electron window or navigate to `http://localhost:5173`.

### 4. Run a Scenario

1. Open the **Operations** tab in the UI
2. Select a scenario and target agent
3. Click **DEPLOY** to create a run
4. Watch the agent execute steps in real-time on the **Missions** tab

## Configuration

### Server Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DATABASE_URL` | `sqlite:red-sim.db` | SQLite database path |
| `SCENARIOS_PATH` | `data/scenarios` | Directory containing scenario JSON files |
| `FINGERPRINT_RULES_PATH` | — | Path to fingerprint rules JSON |
| `OPERATOR_TOKEN` | — | Bearer token for operator API auth. If unset, auth is disabled (dev mode) |
| `AI_KEY_MASTER` | — | Passphrase for AES-256-GCM encryption of AI API keys. If unset, keys stored as plaintext |
| `CORS_PERMISSIVE` | `false` | Set to `true` for permissive CORS (development) |
| `LAUNCH_UI` | `false` | Set to `true` to auto-launch the UI dev server |

### Agent Environment Variables

These can be set at **build time** (embedded into the binary) or at **runtime**:

| Variable | Build-time Env | Default | Description |
|----------|---------------|---------|-------------|
| Server URL | `AGENT_SERVER_URL` | `http://127.0.0.1:3000` | C2 server address |
| Agent GUID | `AGENT_GUID` | `dev-agent-no-guid` | Unique agent identifier |
| Sleep interval | `AGENT_SLEEP_SEC` | `5` | Heartbeat/poll interval in seconds |

Build-time embedding example:

```bash
cd Agent
AGENT_GUID=prod-agent-001 AGENT_SERVER_URL=https://c2.example.com AGENT_SLEEP_SEC=10 cargo build --release
```

### UI Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `VITE_SERVER_URL` | `http://127.0.0.1:3000` | Server API endpoint |

## Project Structure

```
├── server/                    # Rust HTTP API server
│   ├── src/
│   │   ├── main.rs           # Entry point, route definitions
│   │   ├── handlers.rs       # All API handlers
│   │   ├── models.rs         # Data models (Agent, Run, Step, etc.)
│   │   ├── db.rs             # Database initialization & migrations
│   │   ├── scenarios.rs      # Scenario engine & catalog
│   │   ├── fingerprint.rs    # Offline banner fingerprint matcher
│   │   ├── ai.rs             # AI script generator (Claude/OpenAI/Gemini)
│   │   ├── auth.rs           # Operator token auth middleware
│   │   └── crypto.rs         # AES-256-GCM API key encryption
│   └── data/
│       ├── scenarios/        # Scenario JSON definitions
│       └── fingerprint_rules.sample.json
│
├── Agent/                     # Rust cross-platform agent
│   ├── build.rs              # Build-time config embedding
│   ├── src/
│   │   ├── main.rs           # Agent loop: register → poll → execute → report
│   │   ├── config.rs         # Embedded config (GUID, server URL, sleep)
│   │   ├── lib.rs            # Library root
│   │   ├── executor/         # OS execution modules
│   │   │   ├── windows/      # COM, PowerShell, Process, Registry, Fileless
│   │   │   └── linux/        # Memfd, Shell, Syscall
│   │   ├── scanner/          # Network reconnaissance
│   │   │   ├── port.rs       # TCP port scanner + banner grabber
│   │   │   ├── banner.rs     # Banner analysis
│   │   │   └── active_directory.rs  # AD/LDAP scanner
│   │   ├── transport/        # Server communication
│   │   │   ├── http.rs       # HTTP transport with auth
│   │   │   └── protocol.rs   # Wire protocol types
│   │   └── evasion/          # Evasion techniques
│   │       ├── anti_analysis.rs     # Anti-debugging, VM detection
│   │       └── string_obfuscation.rs
│   └── Cargo.toml
│
├── ui/                        # Electron + React UI
│   ├── src/
│   │   ├── App.tsx           # Main layout with game-style navigation
│   │   ├── main.tsx          # React entry point
│   │   ├── lib/
│   │   │   ├── api.ts        # Server API client
│   │   │   └── types.ts      # TypeScript type definitions
│   │   ├── panels/           # View panels
│   │   │   ├── MissionsPanel.tsx      # Scenario selection & deployment
│   │   │   ├── AgentsPanel.tsx        # Agent roster & management
│   │   │   ├── GroupsPanel.tsx        # Squad/group management
│   │   │   ├── RunsPanel.tsx          # Mission queue & run list
│   │   │   ├── RunDetailPanel.tsx     # Step-by-step run progress
│   │   │   ├── EventsPanel.tsx        # Combat log (events)
│   │   │   ├── FingerprintPanel.tsx   # Intel center (fingerprint)
│   │   │   ├── BriefingPanel.tsx      # Mission briefing
│   │   │   ├── MapPanel.tsx           # Network map
│   │   │   ├── AchievementsPanel.tsx  # Achievement tracker
│   │   │   └── AIScriptPanel.tsx      # AI scenario generator
│   │   ├── components/       # Reusable game UI components
│   │   ├── hooks/            # Sound effects & game audio
│   │   └── styles/theme.css  # Cyberpunk theme
│   └── package.json
│
└── README.md
```

## How It Works

### Step Lifecycle

```
LOCKED → READY → COMPLETED / FAILED
```

1. Server creates a **run** from a scenario with sequential **steps**
2. First step starts as `READY`, rest as `LOCKED`
3. Agent polls for pending runs, fetches steps, executes `READY` steps
4. Agent calls `complete_step` with results
5. Server unlocks the next `LOCKED` step to `READY`
6. Choice-gated steps unlock when operator selects a branch via the UI

### Agent Execution Flow

```
Register → Approve (UI) → Poll Pending Runs → Fetch Steps
    → Find READY Step → Execute → POST complete_step
    → Server unlocks next step → repeat until all done
    → POST run result → Heartbeat loop continues
```

### Agent Build System

The server can build custom agent binaries with embedded configuration:

```bash
# Via API
POST /api/agents/build
{
  "target_platform": "windows-x86_64",
  "server_url": "https://c2.example.com",
  "sleep_sec": 10
}
```

The build injects a unique GUID, server URL, and sleep interval into the binary at compile time via `build.rs`.

## Agent Executors

### Windows
| Executor | Description |
|----------|-------------|
| **COM** | COM automation via `WScript.Shell`, `MMC20.Application` |
| **PowerShell** | PowerShell command execution with pipe capture |
| **Process** | Direct Win32 `CreateProcessW` |
| **Registry** | Windows registry read/write/delete |
| **Fileless** | In-memory shellcode execution via `VirtualAlloc` + `CreateThread` |

### Linux
| Executor | Description |
|----------|-------------|
| **Memfd** | Memory-only ELF execution via `memfd_create` + `fexecve` |
| **Syscall** | Direct `SYS_execveat` from anonymous fd |
| **Shell** | Standard `/bin/sh -c` execution |

### Cross-Platform
| Module | Description |
|--------|-------------|
| **Port Scanner** | TCP connect scan with banner grabbing (top 1000 ports) |
| **Banner Analyzer** | Service/version identification from banners |
| **AD/LDAP** | Active Directory reconnaissance |

## API Reference

### Authentication

If `OPERATOR_TOKEN` is set, operator-facing endpoints require:
```
Authorization: Bearer <token>
```

Agent-facing endpoints (register, heartbeat, poll, result) do not require operator auth.

### Scenarios
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/scenarios` | List all scenarios |
| `GET` | `/api/scenarios/:id` | Get scenario detail |
| `POST` | `/api/scenarios/validate` | Validate scenario JSON |

### Agents
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/api/agents/register` | Agent | Register new agent |
| `POST` | `/api/agents/:id/heartbeat` | Agent | Send heartbeat |
| `GET` | `/api/agents/list` | Operator | List agents (max 200) |
| `GET` | `/api/agents/pending` | Operator | List pending approval |
| `POST` | `/api/agents/:id/approve` | Operator | Approve agent |
| `POST` | `/api/agents/:id/block` | Operator | Block agent |
| `POST` | `/api/agents/build` | Operator | Build agent binary |
| `GET` | `/api/agents/builds` | Operator | List builds |

### Runs & Steps
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/api/runs` | Operator | Create a run |
| `GET` | `/api/runs` | Operator | List runs |
| `GET` | `/api/runs/:id` | Operator | Get run detail |
| `GET` | `/api/runs/:id/steps` | Operator | Get run steps |
| `GET` | `/api/runs/:id/verdict` | Operator | Get step verdicts |
| `POST` | `/api/runs/:id/replay` | Operator | Replay a completed run |
| `GET` | `/api/runs/pending/:agent_id` | Agent | Poll pending runs |
| `POST` | `/api/runs/:run_id/result` | Agent | Submit run result |
| `POST` | `/api/runs/:run_id/steps/:step_id/complete` | Agent | Complete a step |

### Groups
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/groups` | List groups (max 200) |
| `POST` | `/api/groups` | Create group |
| `POST` | `/api/groups/:id/assign` | Assign agent to group |
| `POST` | `/api/groups/:id/runs` | Create runs for all group agents |

### Evidence & Events
| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `POST` | `/api/evidence` | Agent | Submit evidence |
| `POST` | `/api/events` | Agent | Submit event |
| `GET` | `/api/events` | Operator | List events |
| `GET` | `/api/runs/:id/events` | Operator | List run events |

### AI Script Generator
| Method | Endpoint | Description |
|--------|----------|-------------|
| `GET` | `/api/ai/accounts` | List AI accounts |
| `POST` | `/api/ai/accounts` | Add AI account (Claude/OpenAI/Gemini) |
| `POST` | `/api/ai/conversations` | Start conversation |
| `POST` | `/api/ai/conversations/:id/chat` | Send message |
| `POST` | `/api/ai/conversations/:id/save-scenario` | Save generated scenario to disk |

### Other
| Method | Endpoint | Description |
|--------|----------|-------------|
| `POST` | `/api/fingerprint/match` | Offline banner fingerprint matching |
| `GET` | `/api/achievements` | List achievements |
| `POST` | `/api/achievements/check` | Check/update achievement progress |

## Security Features

- **Operator Auth**: Bearer token middleware separates agent vs operator API access
- **API Key Encryption**: AI provider keys encrypted with AES-256-GCM (requires `AI_KEY_MASTER`)
- **Agent Approval**: New agents require manual approval before receiving runs
- **Build Concurrency**: Semaphore prevents concurrent agent builds (returns 429)
- **Input Validation**: Status allowlists, path traversal prevention, scenario ID validation
- **DB Transactions**: Run + step creation wrapped in transactions for atomicity
- **HTTPS Warning**: Agent logs a warning when connecting over plaintext HTTP to non-localhost

## Tests

```bash
# Server
cd server && cargo test

# Agent
cd Agent && cargo test

# UI
cd ui && npm run lint && npm run build
```

## Scenario Format

Scenarios are JSON files in `data/scenarios/`:

```json
{
  "scenario_id": "example-scenario",
  "test_id": "BAS-001",
  "title": "Example Scenario",
  "description": "Demonstrates step execution",
  "category": "discovery",
  "mitre_ids": ["T1057"],
  "steps": [
    {
      "step_id": "step-1",
      "name": "Process Discovery",
      "executor": "powershell",
      "command": "Get-Process | Select-Object -First 10",
      "assertions": [
        {
          "description": "Command should succeed",
          "type": "exit_code",
          "kind": "equals",
          "contains": "0",
          "required": true
        }
      ]
    }
  ]
}
```

## Safety Notice

This platform contains real OS execution capabilities including process creation, COM automation, registry manipulation, memory injection, and direct syscalls. It is designed for **authorized security testing and education only**.

- Do not deploy agents on production systems without explicit authorization
- Do not distribute compiled agent binaries without strict access controls
- Always use HTTPS and operator authentication in non-local deployments
- Review scenario definitions before execution
