import { useEffect, useMemo, useState } from 'react';
import {
  addAgentTag,
  approveAgent,
  blockAgent,
  listAgentGroups,
  listAgentRuns,
  listAgentTags,
  listAgents,
  listPendingAgents,
  removeAgentTag,
} from '../lib/api';
import type { Agent, AgentTag, Group, Run } from '../lib/types';
import { RunDetailPanel } from './RunDetailPanel';

export function AgentsPanel() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [pending, setPending] = useState<Agent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [selectedAgentId, setSelectedAgentId] = useState<string | null>(null);
  const [runs, setRuns] = useState<Run[]>([]);
  const [groups, setGroups] = useState<Group[]>([]);
  const [tags, setTags] = useState<AgentTag[]>([]);
  const [tagInput, setTagInput] = useState('');
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);

  const onlineCount = useMemo(
    () => agents.filter((a) => a.status.toLowerCase() === 'online').length,
    [agents],
  );

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const [a, p] = await Promise.all([listAgents(), listPendingAgents()]);
      setAgents(a);
      setPending(p);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  async function refreshSelected(agentId: string) {
    const [r, g, t] = await Promise.all([
      listAgentRuns(agentId),
      listAgentGroups(agentId),
      listAgentTags(agentId),
    ]);
    setRuns(r);
    setGroups(g);
    setTags(t);
  }

  useEffect(() => {
    void refresh();
    const t = window.setInterval(() => void refresh(), 5000);
    return () => window.clearInterval(t);
  }, []);

  useEffect(() => {
    if (!selectedAgentId) return;
    void refreshSelected(selectedAgentId).catch((e) =>
      setError(e instanceof Error ? e.message : String(e)),
    );
  }, [selectedAgentId]);

  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === selectedAgentId) || null,
    [agents, selectedAgentId],
  );

  return (
    <div className="content">
      <div className="grid2">
        <div className="card">
          <h3 className="cardTitle">Fleet</h3>
          <div className="row">
            <span className="pill">agents {agents.length}</span>
            <span className="pill">online {onlineCount}</span>
            <span className="pill">offline {agents.length - onlineCount}</span>
            <span className="pill">pending {pending.length}</span>
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Refresh</h3>
          <div className="row">
            <button className="btn" onClick={() => void refresh()} disabled={loading}>
              {loading ? 'Loading…' : 'Reload'}
            </button>
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>
      </div>

      <div style={{ height: 14 }} />

      {pending.length ? (
        <div className="card">
          <h3 className="cardTitle">Pending Approvals</h3>
          <table className="table">
            <thead>
              <tr>
                <th className="th">Hostname</th>
                <th className="th">IP</th>
                <th className="th">OS / Arch</th>
                <th className="th">Agent ID</th>
                <th className="th">Actions</th>
              </tr>
            </thead>
            <tbody>
              {pending.map((a) => (
                <tr key={a.id}>
                  <td className="td">{a.hostname}</td>
                  <td className="td mono">{a.ip}</td>
                  <td className="td">
                    {a.os} / {a.arch}
                  </td>
                  <td className="td mono">{a.id}</td>
                  <td className="td">
                    <div className="row">
                      <button
                        className="btn"
                        onClick={() =>
                          void approveAgent(a.id)
                            .then(refresh)
                            .catch((e) => setError(e instanceof Error ? e.message : String(e)))
                        }
                      >
                        Approve
                      </button>
                      <button
                        className="btn"
                        onClick={() =>
                          void blockAgent(a.id)
                            .then(refresh)
                            .catch((e) => setError(e instanceof Error ? e.message : String(e)))
                        }
                      >
                        Block
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : null}

      <div style={{ height: 14 }} />

      <table className="table">
        <thead>
          <tr>
            <th className="th">Hostname</th>
            <th className="th">User</th>
            <th className="th">OS / Arch</th>
            <th className="th">IP</th>
            <th className="th">Status</th>
            <th className="th">Approval</th>
            <th className="th">Last Seen</th>
            <th className="th">Agent ID</th>
          </tr>
        </thead>
        <tbody>
          {agents.map((a) => (
            <tr
              key={a.id}
              onClick={() => {
                setSelectedAgentId(a.id);
                setSelectedRunId(null);
              }}
              style={{ cursor: 'pointer', background: selectedAgentId === a.id ? 'rgba(101, 214, 255, 0.06)' : undefined }}
            >
              <td className="td">{a.hostname}</td>
              <td className="td">{a.user}</td>
              <td className="td">
                {a.os} / {a.arch}
              </td>
              <td className="td" style={{ fontFamily: 'var(--font-mono)' }}>
                {a.ip}
              </td>
              <td className="td">
                <span className="pill">{a.status}</span>
              </td>
              <td className="td">
                <span className="pill">{a.approval_status}</span>
              </td>
              <td className="td" style={{ fontFamily: 'var(--font-mono)' }}>
                {a.last_seen}
              </td>
              <td className="td mono">{a.id}</td>
            </tr>
          ))}
          {agents.length === 0 ? (
            <tr>
              <td className="td" colSpan={8} style={{ color: 'var(--muted)' }}>
                No agents registered yet. Start `sim_agent` to register one.
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>

      {selectedAgent ? (
        <>
          <div style={{ height: 14 }} />
          <div className="grid2">
            <div className="card">
              <h3 className="cardTitle">Agent Detail</h3>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                <span className="pill">{selectedAgent.hostname}</span>
                <span className="pill">{selectedAgent.ip}</span>
                <span className="pill">{selectedAgent.os}/{selectedAgent.arch}</span>
                <span className="pill">{selectedAgent.approval_status}</span>
              </div>

              <div style={{ height: 12 }} />
              <div className="cardTitle">Groups</div>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                {groups.map((g) => (
                  <span key={g.id} className="pill">
                    {g.name}
                  </span>
                ))}
                {groups.length === 0 ? <span style={{ color: 'var(--muted)' }}>No groups.</span> : null}
              </div>

              <div style={{ height: 12 }} />
              <div className="cardTitle">Tags</div>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                {tags.map((t) => (
                  <button
                    key={t.tag}
                    className="btn"
                    onClick={(e) => {
                      e.stopPropagation();
                      void removeAgentTag(selectedAgent.id, t.tag)
                        .then(() => refreshSelected(selectedAgent.id))
                        .catch((err) => setError(err instanceof Error ? err.message : String(err)));
                    }}
                  >
                    remove {t.tag}
                  </button>
                ))}
                {tags.length === 0 ? <span style={{ color: 'var(--muted)' }}>No tags.</span> : null}
              </div>

              <div style={{ height: 12 }} />
              <div className="row">
                <input
                  className="input"
                  placeholder="add tag"
                  value={tagInput}
                  onChange={(e) => setTagInput(e.target.value)}
                />
                <button
                  className="btn"
                  onClick={() => {
                    const tag = tagInput.trim();
                    if (!tag) return;
                    setTagInput('');
                    void addAgentTag(selectedAgent.id, tag)
                      .then(() => refreshSelected(selectedAgent.id))
                      .catch((err) => setError(err instanceof Error ? err.message : String(err)));
                  }}
                >
                  Add
                </button>
              </div>
            </div>

            <div className="card">
              <h3 className="cardTitle">Recent Runs</h3>
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
                      onClick={(e) => {
                        e.stopPropagation();
                        setSelectedRunId(r.id);
                      }}
                      style={{ cursor: 'pointer', background: selectedRunId === r.id ? 'rgba(101, 214, 255, 0.06)' : undefined }}
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
                        No runs for this agent.
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>

              {selectedRunId ? <RunDetailPanel runId={selectedRunId} onClose={() => setSelectedRunId(null)} /> : null}
            </div>
          </div>
        </>
      ) : null}
    </div>
  );
}
