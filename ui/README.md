# ui

Electron + React dashboard for the BAS MVP.

Screens:
- Agents
- Queue Runs
- Events
- Fingerprint

## Dependencies

From `ui/package.json`:

- React + Vite (TypeScript)
- Electron (dev)
- `concurrently`, `wait-on` (dev orchestration)

## Configuration

- `VITE_SERVER_URL`
  - Default: `http://127.0.0.1:3000`
  - Used by `ui/src/lib/api.ts`

## Run / Test

```bash
cd ui
npm install
```

- Dev (Vite + Electron)
```bash
npm run dev
```

- Dev (web only)
```bash
npm run dev:web
```

- Lint / Build
```bash
npm run lint
npm run build
```

## Inputs / Outputs

The UI calls these server APIs:

- `GET /api/agents/list`
- `POST /api/runs`
- `GET /api/events`
- `POST /api/fingerprint/match`
