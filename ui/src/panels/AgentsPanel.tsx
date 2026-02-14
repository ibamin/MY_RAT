import { useEffect, useMemo, useState } from 'react';
import { listAgents } from '../lib/api';
import type { Agent } from '../lib/types';

export function AgentsPanel() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const onlineCount = useMemo(
    () => agents.filter((a) => a.status.toLowerCase() === 'online').length,
    [agents],
  );

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const data = await listAgents();
      setAgents(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    const t = window.setInterval(() => void refresh(), 5000);
    return () => window.clearInterval(t);
  }, []);

  return (
    <div className="content">
      <div className="grid2">
        <div className="card">
          <h3 className="cardTitle">Fleet</h3>
          <div className="row">
            <span className="pill">agents {agents.length}</span>
            <span className="pill">online {onlineCount}</span>
            <span className="pill">offline {agents.length - onlineCount}</span>
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

      <table className="table">
        <thead>
          <tr>
            <th className="th">Hostname</th>
            <th className="th">User</th>
            <th className="th">OS / Arch</th>
            <th className="th">IP</th>
            <th className="th">Status</th>
            <th className="th">Last Seen</th>
            <th className="th">Agent ID</th>
          </tr>
        </thead>
        <tbody>
          {agents.map((a) => (
            <tr key={a.id}>
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
              <td className="td" style={{ fontFamily: 'var(--font-mono)' }}>
                {a.last_seen}
              </td>
              <td className="td mono">{a.id}</td>
            </tr>
          ))}
          {agents.length === 0 ? (
            <tr>
              <td className="td" colSpan={7} style={{ color: 'var(--muted)' }}>
                No agents registered yet. Start `sim_agent` to register one.
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}
