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
      setStatus(`queued ${res.runs.length} runs`);
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
          <h3 className="cardTitle">Groups</h3>
          <div className="field">
            <div className="label">Select group</div>
            <select className="input" value={selectedGroupId} onChange={(e) => setSelectedGroupId(e.target.value)}>
              {groups.map((g) => (
                <option key={g.id} value={g.id}>
                  {g.name}
                </option>
              ))}
            </select>
          </div>

          <div className="row">
            <input
              className="input"
              placeholder="new group name"
              value={newGroupName}
              onChange={(e) => setNewGroupName(e.target.value)}
              disabled={busy}
            />
            <button className="btn" onClick={() => void onCreateGroup()} disabled={busy}>
              Create
            </button>
          </div>

          {error ? <div style={{ color: 'var(--danger)', marginTop: 10 }}>{error}</div> : null}
        </div>

        <div className="card">
          <h3 className="cardTitle">Assign agent</h3>
          <div className="field">
            <div className="label">Agent</div>
            <select className="input" value={selectedAgentId} onChange={(e) => setSelectedAgentId(e.target.value)}>
              {agents.map((a) => (
                <option key={a.id} value={a.id}>
                  {a.hostname} ({a.ip})
                </option>
              ))}
            </select>
          </div>
          <button className="btn" onClick={() => void onAssign()} disabled={busy || !selectedGroupId || !selectedAgentId}>
            Assign to group
          </button>

          {selectedGroup ? (
            <div style={{ marginTop: 10, color: 'var(--muted)' }}>
              Selected: <span className="mono">{selectedGroup.name}</span>
            </div>
          ) : null}

          <div style={{ height: 14 }} />
          <h3 className="cardTitle">Run on group</h3>
          <div className="field">
            <div className="label">Scenario</div>
            <select className="input" value={selectedScenarioId} onChange={(e) => setSelectedScenarioId(e.target.value)} disabled={busy}>
              {scenarios.map((s) => (
                <option key={s.scenario_id} value={s.scenario_id}>
                  {s.test_id} · {s.title}
                </option>
              ))}
            </select>
          </div>
          <button className="btn" onClick={() => void onGroupRun()} disabled={busy || !selectedGroupId || !selectedScenarioId}>
            Queue scenario for group
          </button>
          {status ? <div className="mono" style={{ marginTop: 10 }}>{status}</div> : null}
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="card">
        <div className="row" style={{ justifyContent: 'space-between' }}>
          <h3 className="cardTitle" style={{ margin: 0 }}>
            Group Members
          </h3>
          <button className="btn" onClick={() => (selectedGroupId ? void refreshGroupAgents(selectedGroupId) : undefined)} disabled={busy || !selectedGroupId}>
            Refresh
          </button>
        </div>

        <div style={{ height: 10 }} />

        <table className="table">
          <thead>
            <tr>
              <th className="th">Hostname</th>
              <th className="th">IP</th>
              <th className="th">Approval</th>
              <th className="th">Agent ID</th>
              <th className="th">Actions</th>
            </tr>
          </thead>
          <tbody>
            {groupAgents.map((a) => (
              <tr key={a.id}>
                <td className="td">{a.hostname}</td>
                <td className="td mono">{a.ip}</td>
                <td className="td">
                  <span className="pill">{a.approval_status}</span>
                </td>
                <td className="td mono">{a.id}</td>
                <td className="td">
                  <button className="btn" onClick={() => void onUnassign(a.id)} disabled={busy}>
                    Remove
                  </button>
                </td>
              </tr>
            ))}
            {groupAgents.length === 0 ? (
              <tr>
                <td className="td" colSpan={5} style={{ color: 'var(--muted)' }}>
                  No agents assigned.
                </td>
              </tr>
            ) : null}
          </tbody>
        </table>
      </div>
    </div>
  );
}
