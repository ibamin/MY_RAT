import { useEffect, useMemo, useState } from 'react';
import { createRun, listAgents } from '../lib/api';
import type { Agent } from '../lib/types';

export function RunsPanel() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [agentId, setAgentId] = useState('');
  const [testId, setTestId] = useState('');
  const [paramsJson, setParamsJson] = useState('');
  const [status, setStatus] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const hasAgents = agents.length > 0;
  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === agentId) || null,
    [agents, agentId],
  );

  useEffect(() => {
    (async () => {
      try {
        const a = await listAgents();
        setAgents(a);
        if (!agentId && a[0]?.id) {
          setAgentId(a[0].id);
        }
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, [agentId]);

  async function submit() {
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const payload = {
        agent_id: agentId,
        test_id: testId.trim(),
        params_json: paramsJson.trim() ? paramsJson.trim() : null,
      };
      if (!payload.agent_id) {
        throw new Error('Select an agent');
      }
      if (!payload.test_id) {
        throw new Error('Enter a test_id');
      }
      const run = await createRun(payload);
      setStatus(`queued run ${run.id}`);
      setTestId('');
      setParamsJson('');
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
            <div className="label">test_id</div>
            <input
              className="input"
              placeholder="e.g. BAS-DEMO-001"
              value={testId}
              onChange={(e) => setTestId(e.target.value)}
              disabled={busy}
            />
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
    </div>
  );
}
