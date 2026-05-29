import { useEffect, useMemo, useState } from 'react';
import {
  assignAgentToGroup,
  createGroupRuns,
  createGroup,
  listAgents,
  listGroupAgents,
  listGroups,
  listScenarios,
  unassignAgentFromGroup,
} from '../lib/api';
import type { Agent, Group, ScenarioMeta } from '../lib/types';
import { motion, AnimatePresence } from 'framer-motion';

function approvalColor(s: string) {
  const lo = s.toLowerCase();
  if (lo === 'approved') return 'var(--status-victory)';
  if (lo === 'blocked') return 'var(--status-defeat)';
  if (lo === 'pending') return 'var(--status-pending)';
  return 'var(--muted)';
}

export function GroupsPanel() {
  const [groups, setGroups] = useState<Group[]>([]);
  const [agents, setAgents] = useState<Agent[]>([]);
  const [scenarios, setScenarios] = useState<ScenarioMeta[]>([]);
  const [selectedGroupId, setSelectedGroupId] = useState<string>('');
  const [selectedAgentId, setSelectedAgentId] = useState<string>('');
  const [selectedScenarioId, setSelectedScenarioId] = useState<string>('');
  const [groupAgents, setGroupAgents] = useState<Agent[]>([]);
  const [newGroupName, setNewGroupName] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [status, setStatus] = useState<string | null>(null);

  const selectedGroup = useMemo(
    () => groups.find((g) => g.id === selectedGroupId) || null,
    [groups, selectedGroupId],
  );

  async function refresh() {
    setError(null);
    setStatus(null);
    try {
      const [g, a, s] = await Promise.all([listGroups(), listAgents(), listScenarios()]);
      setGroups(g);
      setAgents(a);
      setScenarios(s);
      if (!selectedGroupId && g[0]?.id) setSelectedGroupId(g[0].id);
      if (!selectedAgentId && a[0]?.id) setSelectedAgentId(a[0].id);
      if (!selectedScenarioId && s[0]?.scenario_id) setSelectedScenarioId(s[0].scenario_id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function refreshGroupAgents(groupId: string) {
    try {
      const ga = await listGroupAgents(groupId);
      setGroupAgents(ga);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  useEffect(() => {
    if (!selectedGroupId) return;
    void refreshGroupAgents(selectedGroupId);
  }, [selectedGroupId]);

  async function onCreateGroup() {
    const name = newGroupName.trim();
    if (!name) return;
    setBusy(true);
    setError(null);
    try {
      const g = await createGroup(name);
      setNewGroupName('');
      await refresh();
      setSelectedGroupId(g.id);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onAssign() {
    if (!selectedGroupId || !selectedAgentId) return;
    setBusy(true);
    setError(null);
    try {
      await assignAgentToGroup(selectedGroupId, selectedAgentId);
      await refreshGroupAgents(selectedGroupId);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onUnassign(agentId: string) {
    if (!selectedGroupId) return;
    setBusy(true);
    setError(null);
    try {
      await unassignAgentFromGroup(selectedGroupId, agentId);
      await refreshGroupAgents(selectedGroupId);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function onGroupRun() {
    if (!selectedGroupId || !selectedScenarioId) return;
    setBusy(true);
    setError(null);
    setStatus(null);
    try {
      const res = await createGroupRuns({ group_id: selectedGroupId, scenario_id: selectedScenarioId, params_json: null });
      setStatus(`🚀 ${res.runs.length} operations deployed to squad`);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="content">
      <div className="grid2">
        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
            🔗 Squads
          </h3>
          <div className="field">
            <div className="label" style={{ color: 'var(--neon-cyan)' }}>Active Squad</div>
            <select
              className="input"
              value={selectedGroupId}
              onChange={(e) => setSelectedGroupId(e.target.value)}
            >
              {groups.map((g) => (
                <option key={g.id} value={g.id}>
                  {g.name}
                </option>
              ))}
            </select>
          </div>

          {selectedGroup ? (
            <div className="row" style={{ marginBottom: 10 }}>
              <span className="tagBadge" style={{ borderColor: 'rgba(0,240,255,0.3)', color: 'var(--neon-cyan)' }}>
                {selectedGroup.name}
              </span>
              <span className="tagBadge">
                🎖 {groupAgents.length} members
              </span>
              <span className="tagBadge" style={{ color: 'var(--muted-2)', fontSize: 11 }}>
                created {selectedGroup.created_at}
              </span>
            </div>
          ) : null}

          <div style={{ borderTop: '1px solid rgba(255,255,255,0.06)', paddingTop: 12 }}>
            <div className="label" style={{ color: 'var(--neon-green)' }}>Form New Squad</div>
            <div className="row">
              <input
                className="input"
                placeholder="squad name"
                value={newGroupName}
                onChange={(e) => setNewGroupName(e.target.value)}
                disabled={busy}
                style={{ borderColor: 'var(--game-border)' }}
              />
              <button
                className="btn"
                style={{
                  background: 'linear-gradient(180deg, rgba(57,255,20,0.15), rgba(57,255,20,0.04))',
                  borderColor: 'rgba(57,255,20,0.35)',
                }}
                onClick={() => void onCreateGroup()}
                disabled={busy || !newGroupName.trim()}
              >
                + Create
              </button>
            </div>
          </div>

          {error ? <div style={{ color: 'var(--danger)', marginTop: 10 }}>{error}</div> : null}
        </div>

        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
            ⚙ Squad Actions
          </h3>

          <div className="field">
            <div className="label" style={{ color: 'var(--neon-purple)' }}>Recruit Agent</div>
            <select
              className="input"
              value={selectedAgentId}
              onChange={(e) => setSelectedAgentId(e.target.value)}
            >
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.hostname} ({a.ip})
                </option>
              ))}
            </select>
          </div>
          <button
            className="btn"
            style={{
              background: 'linear-gradient(180deg, rgba(191,90,242,0.15), rgba(191,90,242,0.04))',
              borderColor: 'rgba(191,90,242,0.35)',
            }}
            onClick={() => void onAssign()}
            disabled={busy || !selectedGroupId || !selectedAgentId}
          >
            🔗 Assign to Squad
          </button>

          <div style={{ height: 18, borderBottom: '1px solid rgba(255,255,255,0.06)' }} />

          <div style={{ paddingTop: 12 }}>
            <div className="label" style={{ color: 'var(--neon-yellow)' }}>Deploy to Entire Squad</div>
            <div className="field">
              <select
                className="input"
                value={selectedScenarioId}
                onChange={(e) => setSelectedScenarioId(e.target.value)}
                disabled={busy}
              >
                {scenarios.map((s) => (
                  <option key={s.scenario_id} value={s.scenario_id}>
                    {s.test_id} · {s.title}
                  </option>
                ))}
              </select>
            </div>
            <button
              className="btn"
              style={{
                background: 'linear-gradient(180deg, rgba(0,240,255,0.2), rgba(0,240,255,0.06))',
                borderColor: 'rgba(0,240,255,0.4)',
              }}
              onClick={() => void onGroupRun()}
              disabled={busy || !selectedGroupId || !selectedScenarioId}
            >
              🚀 Deploy to Squad
            </button>
            {status ? (
              <div className="mono" style={{ marginTop: 10, color: 'var(--status-victory)' }}>{status}</div>
            ) : null}
          </div>
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="card" style={{ borderColor: 'var(--game-border)' }}>
        <div className="row" style={{ justifyContent: 'space-between' }}>
          <h3 className="cardTitle" style={{ margin: 0, color: 'var(--neon-purple)' }}>
            👥 Squad Roster
            {selectedGroup ? (
              <span style={{ color: 'var(--neon-cyan)', fontSize: 14, marginLeft: 10 }}>
                — {selectedGroup.name}
              </span>
            ) : null}
          </h3>
          <button
            className="btn"
            onClick={() => (selectedGroupId ? void refreshGroupAgents(selectedGroupId) : undefined)}
            disabled={busy || !selectedGroupId}
          >
            🔄 Refresh
          </button>
        </div>

        <div style={{ height: 10 }} />

        <table className="table">
          <thead>
            <tr>
              <th className="th">Hostname</th>
              <th className="th">IP</th>
              <th className="th">Clearance</th>
              <th className="th">Agent ID</th>
              <th className="th">Actions</th>
            </tr>
          </thead>
          <tbody>
            <AnimatePresence>
              {groupAgents.map((a) => (
                <motion.tr
                  key={a.id}
                  initial={{ opacity: 0, x: -10 }}
                  animate={{ opacity: 1, x: 0 }}
                  exit={{ opacity: 0, x: 10 }}
                  transition={{ duration: 0.15 }}
                >
                  <td className="td" style={{ color: 'var(--neon-cyan)' }}>{a.hostname}</td>
                  <td className="td mono">{a.ip}</td>
                  <td className="td">
                    <span
                      className="tagBadge"
                      style={{
                        borderColor: approvalColor(a.approval_status),
                        color: approvalColor(a.approval_status),
                      }}
                    >
                      {a.approval_status}
                    </span>
                  </td>
                  <td className="td mono" style={{ fontSize: 11, color: 'var(--muted)' }}>{a.id}</td>
                  <td className="td">
                    <button
                      className="btn"
                      style={{ borderColor: 'rgba(255,45,149,0.35)', fontSize: 12, padding: '4px 10px' }}
                      onClick={() => void onUnassign(a.id)}
                      disabled={busy}
                    >
                      ✕ Remove
                    </button>
                  </td>
                </motion.tr>
              ))}
            </AnimatePresence>
            {groupAgents.length === 0 ? (
              <tr>
                <td className="td" colSpan={5} style={{ textAlign: 'center', padding: 20 }}>
                  <div className="dialogueBox" style={{ display: 'inline-block' }}>
                    <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>COMMAND</div>
                    <div className="dialogueText" style={{ color: 'var(--muted)' }}>
                      No agents assigned to this squad. Recruit agents above.
                    </div>
                  </div>
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </div>
  );
}
