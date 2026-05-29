import { useEffect, useMemo, useState } from 'react';
import { createRun, listAgents, listScenarios } from '../lib/api';
import type { Agent, ScenarioMeta } from '../lib/types';
import { MissionBriefing } from '../components/MissionBriefing';
import { useSound } from '../hooks/useSound';

export function MissionsPanel(props: { onQueuedRun: (runId: string) => void }) {
  const { playSfx } = useSound();
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
      playSfx('deploy');
      const run = await createRun({ agent_id: agentId, scenario_id: scenarioId, params_json: null });
      setStatus(`Mission deployed → Run ${run.id}`);
      playSfx('success');
      props.onQueuedRun(run.id);
    } catch (e) {
      playSfx('failure');
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="content">
      <div className="grid2">
        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>Deploy Operation</h3>

          <div className="field">
            <div className="label" style={{ color: 'var(--neon-cyan)' }}>
              🎯 Target Agent
            </div>
            <select
              className="input"
              value={agentId}
              onChange={(e) => setAgentId(e.target.value)}
              disabled={busy}
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
            <button
              className="btn"
              style={{
                background: 'linear-gradient(180deg, rgba(0,240,255,0.2), rgba(0,240,255,0.06))',
                borderColor: 'rgba(0,240,255,0.4)',
              }}
              onClick={() => void startMission()}
              disabled={busy || !agentId || !scenarioId}
            >
              {busy ? '⏳ Deploying…' : '🚀 Deploy Mission'}
            </button>
            {status ? (
              <span className="mono" style={{ color: 'var(--status-victory)' }}>
                {status}
              </span>
            ) : null}
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>Mission Briefing</h3>
          {selectedScenario ? (
            <MissionBriefing scenario={selectedScenario} />
          ) : (
            <div className="dialogueBox">
              <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>COMMAND</div>
              <div className="dialogueText" style={{ color: 'var(--muted)' }}>
                Select a scenario to view the mission briefing.
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
