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
import { CharacterCard } from '../components/CharacterCard';
import { RunDetailPanel } from './RunDetailPanel';
import { useSound } from '../hooks/useSound';

export function AgentsPanel() {
  const { playSfx } = useSound();
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
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>Fleet Status</h3>
          <div className="row">
            <span className="tagBadge">🎖 {agents.length} agents</span>
            <span className="pill" style={{ borderColor: 'var(--status-victory)', color: 'var(--status-victory)' }}>
              ● {onlineCount} online
            </span>
            <span className="pill" style={{ color: 'var(--muted-2)' }}>
              ○ {agents.length - onlineCount} offline
            </span>
            {pending.length > 0 ? (
              <span className="pill" style={{ borderColor: 'var(--status-pending)', color: 'var(--status-pending)' }}>
                ⏳ {pending.length} pending
              </span>
            ) : null}
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Actions</h3>
          <div className="row">
            <button className="btn" onClick={() => { playSfx('click'); void refresh(); }} disabled={loading}>
              {loading ? 'Scanning…' : '🔄 Refresh Fleet'}
            </button>
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>
      </div>

      <div style={{ height: 14 }} />

      {pending.length > 0 ? (
        <>
          <div className="card" style={{ borderColor: 'rgba(255, 225, 86, 0.25)' }}>
            <h3 className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
              ⚠ Pending Approvals
            </h3>
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
                          style={{ borderColor: 'rgba(57,255,20,0.4)' }}
                          onClick={() => {
                            playSfx('success');
                            void approveAgent(a.id)
                              .then(refresh)
                              .catch((e) => setError(e instanceof Error ? e.message : String(e)));
                          }}
                        >
                          ✓ Approve
                        </button>
                        <button
                          className="btn"
                          style={{ borderColor: 'rgba(255,45,149,0.4)' }}
                          onClick={() => {
                            playSfx('failure');
                            void blockAgent(a.id)
                              .then(refresh)
                              .catch((e) => setError(e instanceof Error ? e.message : String(e)));
                          }}
                        >
                          ✕ Block
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          <div style={{ height: 14 }} />
        </>
      ) : null}

      <div className="gridRoster">
        {agents.map((a) => (
          <CharacterCard
            key={a.id}
            agent={a}
            selected={selectedAgentId === a.id}
            onClick={() => {
              playSfx('click');
              setSelectedAgentId(a.id);
              setSelectedRunId(null);
            }}
          />
        ))}
        {agents.length === 0 ? (
          <div className="dialogueBox">
            <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>SYSTEM</div>
            <div className="dialogueText" style={{ color: 'var(--muted)' }}>
              No agents registered yet. Deploy an agent to begin operations.
            </div>
          </div>
        ) : null}
      </div>

      {selectedAgent ? (
        <>
          <div style={{ height: 14 }} />
          <div className="grid2">
            <div className="card" style={{ borderColor: 'var(--game-border)' }}>
              <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>Agent Intel</h3>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                <span className="tagBadge">{selectedAgent.hostname}</span>
                <span className="tagBadge">{selectedAgent.ip}</span>
                <span className="tagBadge">
                  {selectedAgent.os}/{selectedAgent.arch}
                </span>
                <span className="tagBadge">{selectedAgent.approval_status}</span>
              </div>

              <div style={{ height: 12 }} />
              <div className="cardTitle" style={{ color: 'var(--neon-purple)' }}>Squads</div>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                {groups.map((g) => (
                  <span key={g.id} className="tagBadge" style={{ borderColor: 'rgba(191,90,242,0.3)', color: 'var(--neon-purple)' }}>
                    {g.name}
                  </span>
                ))}
                {groups.length === 0 ? (
                  <span style={{ color: 'var(--muted)' }}>No squads assigned.</span>
                ) : null}
              </div>

              <div style={{ height: 12 }} />
              <div className="cardTitle" style={{ color: 'var(--neon-green)' }}>Tags</div>
              <div className="row" style={{ flexWrap: 'wrap' }}>
                {tags.map((t) => (
                  <button
                    key={t.tag}
                    className="btn"
                    style={{ fontSize: 12, padding: '4px 10px' }}
                    onClick={(e) => {
                      e.stopPropagation();
                      void removeAgentTag(selectedAgent.id, t.tag)
                        .then(() => refreshSelected(selectedAgent.id))
                        .catch((err) => setError(err instanceof Error ? err.message : String(err)));
                    }}
                  >
                    ✕ {t.tag}
                  </button>
                ))}
                {tags.length === 0 ? (
                  <span style={{ color: 'var(--muted)' }}>No tags.</span>
                ) : null}
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
                  + Add
                </button>
              </div>
            </div>

            <div className="card" style={{ borderColor: 'var(--game-border)' }}>
              <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>Mission History</h3>
              <table className="table">
                <thead>
                  <tr>
                    <th className="th">Timestamp</th>
                    <th className="th">Test ID</th>
                    <th className="th">Status</th>
                    <th className="th">Run ID</th>
                  </tr>
                </thead>
                <tbody>
                  {runs.map((r) => (
                    <tr
                      key={r.id}
                      onClick={(e) => {
                        e.stopPropagation();
                        playSfx('click');
                        setSelectedRunId(r.id);
                      }}
                      style={{
                        cursor: 'pointer',
                        background:
                          selectedRunId === r.id ? 'rgba(0, 240, 255, 0.06)' : undefined,
                      }}
                    >
                      <td className="td mono">{r.created_at}</td>
                      <td className="td">{r.test_id}</td>
                      <td className="td">
                        <span
                          className="pill"
                          style={{
                            borderColor:
                              r.status === 'completed'
                                ? 'rgba(57,255,20,0.3)'
                                : 'var(--stroke)',
                            color:
                              r.status === 'completed'
                                ? 'var(--status-victory)'
                                : undefined,
                          }}
                        >
                          {r.status}
                        </span>
                      </td>
                      <td className="td mono">{r.id}</td>
                    </tr>
                  ))}
                  {runs.length === 0 ? (
                    <tr>
                      <td className="td" colSpan={4} style={{ color: 'var(--muted)' }}>
                        No missions for this agent.
                      </td>
                    </tr>
                  ) : null}
                </tbody>
              </table>

              {selectedRunId ? (
                <RunDetailPanel
                  runId={selectedRunId}
                  onClose={() => setSelectedRunId(null)}
                />
              ) : null}
            </div>
          </div>
        </>
      ) : null}
    </div>
  );
}
