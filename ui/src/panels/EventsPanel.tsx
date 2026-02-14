import { useEffect, useState } from 'react';
import { listEvents } from '../lib/api';
import type { Event } from '../lib/types';

function levelColor(level: string) {
  const l = level.toLowerCase();
  if (l === 'error') return 'var(--danger)';
  if (l === 'warn') return 'var(--warn)';
  if (l === 'info') return 'var(--accent)';
  return 'var(--muted)';
}

export function EventsPanel() {
  const [events, setEvents] = useState<Event[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      const data = await listEvents();
      setEvents(data);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    const t = window.setInterval(() => void refresh(), 2500);
    return () => window.clearInterval(t);
  }, []);

  return (
    <div className="content">
      <div className="grid2">
        <div className="card">
          <h3 className="cardTitle">Event Stream</h3>
          <div className="row">
            <button className="btn" onClick={() => void refresh()} disabled={loading}>
              {loading ? 'Loading…' : 'Reload'}
            </button>
            <span className="pill">last 200</span>
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Hints</h3>
          <div style={{ color: 'var(--muted)', lineHeight: 1.55 }}>
            Start `sim_agent` and queue a run. You should see run_start/run_done events here.
          </div>
        </div>
      </div>

      <div style={{ height: 14 }} />

      <table className="table">
        <thead>
          <tr>
            <th className="th">ts</th>
            <th className="th">level</th>
            <th className="th">agent</th>
            <th className="th">run</th>
            <th className="th">message</th>
          </tr>
        </thead>
        <tbody>
          {events.map((e) => (
            <tr key={e.id}>
              <td className="td mono">{e.ts}</td>
              <td className="td">
                <span className="pill" style={{ borderColor: 'var(--stroke-2)', color: levelColor(e.level) }}>
                  {e.level}
                </span>
              </td>
              <td className="td mono">{e.agent_id || '-'}</td>
              <td className="td mono">{e.run_id || '-'}</td>
              <td className="td">{e.message}</td>
            </tr>
          ))}
          {events.length === 0 ? (
            <tr>
              <td className="td" colSpan={5} style={{ color: 'var(--muted)' }}>
                No events yet.
              </td>
            </tr>
          ) : null}
        </tbody>
      </table>
    </div>
  );
}
