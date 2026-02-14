import { useEffect, useMemo, useState } from 'react';
import { createRun, listAgents, listScenarios } from '../lib/api';
import type { Agent, ScenarioMeta } from '../lib/types';

export function MissionsPanel(props: { onQueuedRun: (runId: string) => void }) {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [scenarios, setScenarios] = useState<ScenarioMeta[]>([]);
  const [agentId, setAgentId] = useState('');
  const [scenarioId, setScenarioId] = useState('');
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const selectedScenario = useMemo(
    () => scenarios.find((s) => s.scenario_id === scenarioId) || null,
    [scenarios, scenarioId],
  );

  useEffect(() => {
    (async () => {
      try {
        const [a, s] = await Promise.all([listAgents(), listScenarios()]);
        setAgents(a);
        setScenarios(s);
        if (!agentId && a[0]?.id) setAgentId(a[0].id);
        if (!scenarioId && s[0]?.scenario_id) setScenarioId(s[0].scenario_id);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [agentId, scenarioId]);

  async function startMission() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      if (!agentId) throw new Error('Select an agent');
      if (!scenarioId) throw new Error('Select a scenario');
      const run = await createRun({ agent_id: agentId, scenario_id: scenarioId, params_json: null });
      setStatus(`queued run ${run.id}`);
      props.onQueuedRun(run.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="content">
      <div className="grid2">
        <div className="card">
          <h3 className="cardTitle">Select Mission</h3>

          <div className="field">
            <div className="label">Agent (Simulated Endpoint)</div>
            <select className="input" value={agentId} onChange={(e) => setAgentId(e.target.value)} disabled={busy}>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.hostname} ({a.ip})
                </option>
              ))}
            </select>
          </div>

          <div className="field">
            <div className="label">Scenario</div>
            <select
              className="input"
              value={scenarioId}
              onChange={(e) => setScenarioId(e.target.value)}
              disabled={busy}
            >
              {scenarios.map((s) => (
                <option key={s.scenario_id} value={s.scenario_id}>
                  {s.test_id} · {s.title}
                </option>
              ))}
            </select>
          </div>

          <div className="row">
            <button className="btn" onClick={() => void startMission()} disabled={busy || !agentId || !scenarioId}>
              {busy ? 'Starting…' : 'Start Mission'}
            </button>
            {status ? <span className="mono">{status}</span> : null}
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Mission Brief</h3>
          {selectedScenario ? (
            <div style={{ display: 'grid', gap: 10 }}>
              <div style={{ fontWeight: 650 }}>{selectedScenario.title}</div>
              <div className="row">
                <span className="pill">{selectedScenario.test_id}</span>
                <span className="pill">difficulty {selectedScenario.difficulty}</span>
                <span className="pill">v{selectedScenario.version}</span>
              </div>
              <div style={{ color: 'var(--muted)' }}>
                estimated time: <span className="mono">{selectedScenario.estimated_time_sec}s</span>
              </div>
              <div style={{ color: 'var(--muted)' }}>
                This is a BAS mission. Actions are allowlisted; results are evaluated as PASS/FAIL with evidence.
              </div>
            </div>
          ) : (
            <div style={{ color: 'var(--muted)' }}>No scenario selected.</div>
          )}
        </div>
      </div>
    </div>
  );
}
