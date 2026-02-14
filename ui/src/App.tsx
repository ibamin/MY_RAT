import { useMemo, useState } from 'react';
import { getServerUrl } from './lib/api';
import { AgentsPanel } from './panels/AgentsPanel';
import { EventsPanel } from './panels/EventsPanel';
import { FingerprintPanel } from './panels/FingerprintPanel';
import { MissionsPanel } from './panels/MissionsPanel';
import { RunsPanel } from './panels/RunsPanel';

type View = 'missions' | 'agents' | 'runs' | 'events' | 'fingerprint';

export default function App() {
  const [view, setView] = useState<View>('missions');
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const title = useMemo(() => {
    if (view === 'missions') return 'Missions';
    if (view === 'agents') return 'Agents';
    if (view === 'runs') return 'Runs';
    if (view === 'events') return 'Events';
    return 'Fingerprint';
  }, [view]);

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <div className="brandTitle">Red Team Simulation</div>
          <div className="brandBadge">BAS</div>
        </div>

        <nav className="nav">
          <button
            className={`navBtn ${view === 'missions' ? 'navBtnActive' : ''}`}
            onClick={() => setView('missions')}
          >
            Missions
          </button>
          <button
            className={`navBtn ${view === 'agents' ? 'navBtnActive' : ''}`}
            onClick={() => setView('agents')}
          >
            Agents
          </button>
          <button
            className={`navBtn ${view === 'runs' ? 'navBtnActive' : ''}`}
            onClick={() => setView('runs')}
          >
            Queue Runs
          </button>
          <button
            className={`navBtn ${view === 'events' ? 'navBtnActive' : ''}`}
            onClick={() => setView('events')}
          >
            Events
          </button>
          <button
            className={`navBtn ${view === 'fingerprint' ? 'navBtnActive' : ''}`}
            onClick={() => setView('fingerprint')}
          >
            Fingerprint
          </button>
        </nav>

        <div className="sidebarFooter">
          <div className="mono">server: {getServerUrl()}</div>
          <div style={{ marginTop: 8 }}>
            Start order: `server` → `sim_agent` → UI. Queue a run to see events.
          </div>
        </div>
      </aside>

      <main className="main">
        <div className="topbar">
          <div className="topbarTitle">{title}</div>
          <div className="topbarMeta">v0.1 · local dashboard</div>
        </div>

        {view === 'missions' ? (
          <MissionsPanel
            onQueuedRun={(runId) => {
              setSelectedRunId(runId);
              setView('runs');
            }}
          />
        ) : null}
        {view === 'agents' ? <AgentsPanel /> : null}
        {view === 'runs' ? (
          <RunsPanel selectedRunId={selectedRunId} onSelectRun={(runId) => setSelectedRunId(runId)} />
        ) : null}
        {view === 'events' ? <EventsPanel /> : null}
        {view === 'fingerprint' ? <FingerprintPanel /> : null}
      </main>
    </div>
  );
}
