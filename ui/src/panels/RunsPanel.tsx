import { useEffect, useMemo, useState } from 'react';
import { createRun, listAgents, listRuns, listScenarios } from '../lib/api';
import type { Agent, Run, ScenarioMeta } from '../lib/types';
import { RunDetailPanel } from './RunDetailPanel';
import { MissionBriefing } from '../components/MissionBriefing';
import { motion, AnimatePresence } from 'framer-motion';
import { useSound } from '../hooks/useSound';

function statusColor(s: string) {
  const lo = s.toLowerCase();
  if (lo === 'completed') return 'var(--status-victory)';
  if (lo === 'dispatched' || lo === 'running') return 'var(--status-active)';
  if (lo === 'pending') return 'var(--status-pending)';
  if (lo === 'failed') return 'var(--status-defeat)';
  return 'var(--muted)';
}

function statusIcon(s: string) {
  const lo = s.toLowerCase();
  if (lo === 'completed') return '✓';
  if (lo === 'dispatched' || lo === 'running') return '▶';
  if (lo === 'pending') return '⏳';
  if (lo === 'failed') return '✕';
  return '○';
}

export function RunsPanel(props: {
  selectedRunId: string | null;
  onSelectRun: (runId: string | null) => void;
}) {
  const { playSfx } = useSound();
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
      playSfx('deploy');
      const run = await createRun(payload);
      setStatus(`Operation deployed → ${run.id.slice(0, 8)}`);
      playSfx('success');
      setParamsJson('');
      props.onSelectRun(run.id);
      void refreshRuns();
    } catch (e) {
      playSfx('failure');
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  const completedCount = runs.filter((r) => r.status.toLowerCase() === 'completed').length;
  const pendingCount = runs.filter((r) => r.status.toLowerCase() === 'pending').length;

  return (
    <div className="content">
      <div className="grid2">
        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
            🚀 Deploy Operation
          </h3>

          <div className="field">
            <div className="label" style={{ color: 'var(--neon-cyan)' }}>
              🎯 Target Agent
            </div>
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
            <div className="label" style={{ color: 'var(--neon-cyan)' }}>
              📋 Scenario
            </div>
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
              <div style={{ color: 'var(--muted)', fontSize: 12, marginTop: 4 }}>
                test_id: <span className="mono" style={{ color: 'var(--neon-green)' }}>{selectedScenario.test_id}</span>
              </div>
            ) : null}
          </div>

          <div className="field">
            <div className="label" style={{ color: 'var(--neon-purple)' }}>
              ⚙ params_json <span style={{ fontSize: 11, color: 'var(--muted-2)' }}>(optional)</span>
            </div>
            <textarea
              className="input"
              placeholder='{"target":"host-a"}'
              value={paramsJson}
              onChange={(e) => setParamsJson(e.target.value)}
              rows={4}
              disabled={busy}
              style={{ borderColor: 'var(--game-border)' }}
            />
          </div>

          <div className="row">
            <button
              className="btn"
              style={{
                background: 'linear-gradient(180deg, rgba(0,240,255,0.2), rgba(0,240,255,0.06))',
                borderColor: 'rgba(0,240,255,0.4)',
              }}
              onClick={() => void submit()}
              disabled={busy}
            >
              {busy ? '⏳ Queuing…' : '🚀 Deploy'}
            </button>
            {status ? (
              <span className="mono" style={{ color: 'var(--status-victory)' }}>{status}</span>
            ) : null}
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
            🎖 Operation Intel
          </h3>
          {selectedAgent ? (
            <div style={{ display: 'grid', gap: 12 }}>
              <div>
                <div style={{ fontSize: 12, color: 'var(--neon-cyan)', marginBottom: 6, fontWeight: 600 }}>
                  Selected Agent
                </div>
                <div className="row" style={{ flexWrap: 'wrap' }}>
                  <span className="tagBadge" style={{ borderColor: statusColor(selectedAgent.status), color: statusColor(selectedAgent.status) }}>
                    ● {selectedAgent.status}
                  </span>
                  <span className="tagBadge">{selectedAgent.os}</span>
                  <span className="tagBadge">{selectedAgent.arch}</span>
                </div>
                <div className="mono" style={{ fontSize: 11, color: 'var(--muted)', marginTop: 6 }}>
                  {selectedAgent.id}
                </div>
                <div className="mono" style={{ fontSize: 11, color: 'var(--muted)' }}>
                  last_seen: {selectedAgent.last_seen}
                </div>
              </div>
              {selectedScenario ? (
                <div style={{ borderTop: '1px solid rgba(255,255,255,0.06)', paddingTop: 12 }}>
                  <div style={{ fontSize: 12, color: 'var(--neon-yellow)', marginBottom: 6, fontWeight: 600 }}>
                    Mission Briefing
                  </div>
                  <MissionBriefing scenario={selectedScenario} />
                </div>
              ) : null}
            </div>
          ) : (
            <div className="dialogueBox">
              <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>COMMAND</div>
              <div className="dialogueText" style={{ color: 'var(--muted)' }}>
                No agent online. Deploy an agent to begin operations.
              </div>
            </div>
          )}
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="card" style={{ borderColor: 'var(--game-border)' }}>
        <div className="row" style={{ justifyContent: 'space-between' }}>
          <h3 className="cardTitle" style={{ margin: 0, color: 'var(--neon-purple)' }}>
            📜 Operation Log
          </h3>
          <div className="row">
            <span className="tagBadge" style={{ borderColor: 'rgba(57,255,20,0.3)', color: 'var(--status-victory)' }}>
              ✓ {completedCount}
            </span>
            <span className="tagBadge" style={{ borderColor: 'rgba(255,225,86,0.3)', color: 'var(--status-pending)' }}>
              ⏳ {pendingCount}
            </span>
            <button
              className="btn"
              onClick={() => void refreshRuns()}
              disabled={busy}
            >
              🔄 Refresh
            </button>
          </div>
        </div>
        <div style={{ height: 10 }} />
        <table className="table">
          <thead>
            <tr>
              <th className="th">Timestamp</th>
              <th className="th">Test ID</th>
              <th className="th">Status</th>
              <th className="th">Operation ID</th>
            </tr>
          </thead>
          <tbody>
            <AnimatePresence>
              {runs.map((r) => (
                <motion.tr
                  key={r.id}
                  initial={{ opacity: 0 }}
                  animate={{ opacity: 1 }}
                  exit={{ opacity: 0 }}
                  transition={{ duration: 0.15 }}
                  onClick={() => { playSfx('click'); props.onSelectRun(r.id); }}
                  style={{
                    cursor: 'pointer',
                    background: props.selectedRunId === r.id ? 'rgba(0, 240, 255, 0.06)' : undefined,
                  }}
                >
                  <td className="td mono">{r.created_at}</td>
                  <td className="td">
                    <span style={{ color: 'var(--neon-cyan)' }}>{r.test_id}</span>
                  </td>
                  <td className="td">
                    <span
                      className="tagBadge"
                      style={{
                        borderColor: statusColor(r.status),
                        color: statusColor(r.status),
                      }}
                    >
                      {statusIcon(r.status)} {r.status}
                    </span>
                  </td>
                  <td className="td mono" style={{ fontSize: 11, color: 'var(--muted)' }}>{r.id}</td>
                </motion.tr>
              ))}
            </AnimatePresence>
            {runs.length === 0 ? (
              <tr>
                <td className="td" colSpan={4} style={{ color: 'var(--muted)', textAlign: 'center', padding: 20 }}>
                  No operations deployed yet. Configure and deploy above.
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
