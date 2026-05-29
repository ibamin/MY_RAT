import { useEffect, useState } from 'react';
import { listEvents } from '../lib/api';
import type { Event } from '../lib/types';
import { motion, AnimatePresence } from 'framer-motion';

function levelColor(level: string) {
  const l = level.toLowerCase();
  if (l === 'error') return 'var(--neon-pink)';
  if (l === 'warn') return 'var(--neon-yellow)';
  if (l === 'info') return 'var(--neon-cyan)';
  return 'var(--muted)';
}

function levelIcon(level: string) {
  const l = level.toLowerCase();
  if (l === 'error') return '🔴';
  if (l === 'warn') return '🟡';
  if (l === 'info') return '🔵';
  return '⚪';
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

  const errorCount = events.filter((e) => e.level.toLowerCase() === 'error').length;
  const warnCount = events.filter((e) => e.level.toLowerCase() === 'warn').length;
  const infoCount = events.filter((e) => e.level.toLowerCase() === 'info').length;

  return (
    <div className="content">
      <div className="grid2">
        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
            📡 COMMS INTERCEPT
          </h3>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            <button
              className="btn"
              style={{
                background: 'linear-gradient(180deg, rgba(0,240,255,0.15), rgba(0,240,255,0.04))',
                borderColor: 'rgba(0,240,255,0.35)',
              }}
              onClick={() => void refresh()}
              disabled={loading}
            >
              {loading ? '⏳ Scanning…' : '🔄 Refresh Feed'}
            </button>
            <span className="tagBadge">{events.length} intercepted</span>
            {errorCount > 0 ? (
              <span className="tagBadge" style={{ borderColor: 'rgba(255,45,149,0.4)', color: 'var(--neon-pink)' }}>
                🔴 {errorCount} critical
              </span>
            ) : null}
            {warnCount > 0 ? (
              <span className="tagBadge" style={{ borderColor: 'rgba(255,225,86,0.4)', color: 'var(--neon-yellow)' }}>
                🟡 {warnCount} warnings
              </span>
            ) : null}
            {infoCount > 0 ? (
              <span className="tagBadge" style={{ borderColor: 'rgba(0,240,255,0.3)', color: 'var(--neon-cyan)' }}>
                🔵 {infoCount} intel
              </span>
            ) : null}
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-purple)' }}>
            📋 Operator Guide
          </h3>
          <div className="dialogueBox">
            <div className="dialogueSpeaker" style={{ color: 'var(--neon-purple)' }}>COMMAND</div>
            <div className="dialogueText" style={{ color: 'var(--muted)' }}>
              Deploy an agent and queue operations. Intercepted comms will appear here in real-time.
            </div>
          </div>
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="card combatLog" style={{ borderColor: 'var(--game-border)', maxHeight: 600, overflowY: 'auto' }}>
        <AnimatePresence>
          {events.map((e) => (
            <motion.div
              key={e.id}
              initial={{ opacity: 0, x: -12 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 12 }}
              transition={{ duration: 0.2 }}
              style={{
                display: 'grid',
                gridTemplateColumns: '30px 110px 70px 100px 100px 1fr',
                gap: 8,
                alignItems: 'center',
                padding: '8px 12px',
                borderBottom: '1px solid rgba(255,255,255,0.04)',
                fontSize: 13,
              }}
            >
              <span>{levelIcon(e.level)}</span>
              <span className="mono" style={{ color: 'var(--muted-2)', fontSize: 11 }}>
                {e.ts}
              </span>
              <span
                className="tagBadge"
                style={{
                  borderColor: levelColor(e.level),
                  color: levelColor(e.level),
                  fontSize: 11,
                  padding: '2px 8px',
                  textTransform: 'uppercase',
                }}
              >
                {e.level}
              </span>
              <span className="mono" style={{ color: 'var(--neon-green)', fontSize: 11 }}>
                {e.agent_id ? e.agent_id.slice(0, 8) : '—'}
              </span>
              <span className="mono" style={{ color: 'var(--neon-purple)', fontSize: 11 }}>
                {e.run_id ? e.run_id.slice(0, 8) : '—'}
              </span>
              <span style={{ color: 'var(--text)' }}>{e.message}</span>
            </motion.div>
          ))}
        </AnimatePresence>
        {events.length === 0 ? (
          <div style={{ padding: 24, textAlign: 'center' }}>
            <div className="dialogueBox">
              <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>SYSTEM</div>
              <div className="dialogueText" style={{ color: 'var(--muted)' }}>
                No intercepted signals. Comms channel is silent.
              </div>
            </div>
          </div>
        ) : null}
      </div>
    </div>
  );
}
