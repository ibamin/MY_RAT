import { useEffect, useMemo, useState } from 'react';
import { createRun, listAgents, listRuns, listScenarios } from '../lib/api';
import type { Agent, Run, ScenarioMeta } from '../lib/types';
import { RunDetailPanel } from './RunDetailPanel';

export function RunsPanel(props: {
  selectedRunId: string | null;
  onSelectRun: (runId: string | null) => void;
}) {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [scenarios, setScenarios] = useState<ScenarioMeta[]>([]);
  const [runs, setRuns] = useState<Run[]>([]);
  const [agentId, setAgentId] = useState('');
  const [scenarioId, setScenarioId] = useState('');
  const [paramsJson, setParamsJson] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const hasAgents = agents.length > 0;
  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === agentId) || null,
    [agents, agentId],
  );

  const selectedScenario = useMemo(
    () => scenarios.find((s) => s.scenario_id === scenarioId) || null,
    [scenarios, scenarioId],
  );

  useEffect(() => {
    (async () => {
      try {
        const [a, s, r] = await Promise.all([listAgents(), listScenarios(), listRuns()]);
        setAgents(a);
        setScenarios(s);
        setRuns(r);
        if (!agentId && a[0]?.id) setAgentId(a[0].id);
        if (!scenarioId && s[0]?.scenario_id) setScenarioId(s[0].scenario_id);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [agentId, scenarioId]);

  async function refreshRuns() {
    try {
      const r = await listRuns();
      setRuns(r);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function submit() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const payload = {
        agent_id: agentId,
        scenario_id: scenarioId,
        params_json: paramsJson.trim() ? paramsJson.trim() : null,
      };
      if (!payload.agent_id) {
        throw new Error('Select an agent');
      }
      if (!payload.scenario_id) throw new Error('Select a scenario');
      const run = await createRun(payload);
      setStatus(`queued run ${run.id}`);
      setParamsJson('');
      props.onSelectRun(run.id);
      void refreshRuns();
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
          <h3 className="cardTitle">Queue A BAS Run</h3>
          <div className="field">
            <div className="label">Agent</div>
            <select
              className="input"
              value={agentId}
              onChange={(e) => setAgentId(e.target.value)}
              disabled={!hasAgents || busy}
            >
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
              disabled={busy || scenarios.length === 0}
            >
              {scenarios.map((s) => (
                <option key={s.scenario_id} value={s.scenario_id}>
                  {s.test_id} · {s.title}
                </option>
              ))}
            </select>
            {selectedScenario ? (
              <div style={{ color: 'var(--muted)', fontSize: 12 }}>
                test_id: <span className="mono">{selectedScenario.test_id}</span>
              </div>
            ) : null}
          </div>

          <div className="field">
            <div className="label">params_json (optional)</div>
            <textarea
              className="input"
              placeholder='{"target":"host-a"}'
              value={paramsJson}
              onChange={(e) => setParamsJson(e.target.value)}
              rows={4}
              disabled={busy}
            />
          </div>

          <div className="row">
            <button className="btn" onClick={() => void submit()} disabled={busy}>
              {busy ? 'Queuing…' : 'Queue Run'}
            </button>
            {status ? <span className="mono">{status}</span> : null}
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Selected Agent</h3>
          {selectedAgent ? (
            <div style={{ display: 'grid', gap: 8 }}>
              <div className="row">
                <span className="pill">{selectedAgent.status}</span>
                <span className="pill">{selectedAgent.os}</span>
                <span className="pill">{selectedAgent.arch}</span>
              </div>
              <div className="mono">id: {selectedAgent.id}</div>
              <div className="mono">last_seen: {selectedAgent.last_seen}</div>
            </div>
          ) : (
            <div style={{ color: 'var(--muted)' }}>
              No agent selected. Start `sim_agent` to register one.
            </div>
          )}
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="card">
        <div className="row" style={{ justifyContent: 'space-between' }}>
          <h3 className="cardTitle" style={{ margin: 0 }}>
            Recent Runs
          </h3>
          <button className="btn" onClick={() => void refreshRuns()} disabled={busy}>
            Refresh
          </button>
        </div>
        <div style={{ height: 10 }} />
        <table className="table">
          <thead>
            <tr>
              <th className="th">created_at</th>
              <th className="th">test_id</th>
              <th className="th">status</th>
              <th className="th">run_id</th>
            </tr>
          </thead>
          <tbody>
            {runs.map((r) => (
              <tr
                key={r.id}
                onClick={() => props.onSelectRun(r.id)}
                style={{ cursor: 'pointer', background: props.selectedRunId === r.id ? 'rgba(101, 214, 255, 0.06)' : undefined }}
              >
                <td className="td mono">{r.created_at}</td>
                <td className="td">{r.test_id}</td>
                <td className="td">
                  <span className="pill">{r.status}</span>
                </td>
                <td className="td mono">{r.id}</td>
              </tr>
            ))}
            {runs.length === 0 ? (
              <tr>
                <td className="td" colSpan={4} style={{ color: 'var(--muted)' }}>
                  No runs yet.
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>

      {props.selectedRunId ? (
        <RunDetailPanel runId={props.selectedRunId} onClose={() => props.onSelectRun(null)} />
      ) : null}
    </div>
  );
}
