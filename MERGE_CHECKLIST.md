# Merge Checklist (dev/bas-v1-game-ui -> main)

## Pre-merge verification

1) Rust
```bash
cd server && cargo test
cd ..\sim_agent && cargo test
```

2) UI
```bash
cd ui && npm run lint && npm run build
```

## Manual smoke test

1) Start server
```bash
cd server
set SCENARIOS_PATH=data\scenarios
set FINGERPRINT_RULES_PATH=data\fingerprint_rules.sample.json
cargo run
```

2) Start runner
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

4) Verify flow
- Missions: start `BAS-DEMO-001`
- Runs: open newest run -> Run Detail
- Pick a branch choice (e.g. Stealth/Loud) and click Refresh
- Confirm additional steps unlock after choosing
- Confirm Verdict shows PASS/FAIL with assertions and Evidence exists
- Confirm Operator Log shows the choice selection

## Hygiene

- No build artifacts committed (`**/target/`, `ui/dist/`, `ui/node_modules/`)
- No secrets committed (.env, keys, creds)
- Docs aligned: `README.md`, `Simulation 개발 문서.md`
