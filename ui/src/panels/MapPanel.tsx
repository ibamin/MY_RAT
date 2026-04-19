import { useEffect, useMemo, useRef, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { listAgents } from '../lib/api';
import type { Agent } from '../lib/types';

type NodePosition = { x: number; y: number };

function hashToPosition(id: string, width: number, height: number, padding: number): NodePosition {
  let h = 0;
  for (let i = 0; i < id.length; i++) {
    h = ((h << 5) - h + id.charCodeAt(i)) | 0;
  }
  const x = padding + (Math.abs(h % 1000) / 1000) * (width - padding * 2);
  const y = padding + (Math.abs((h >> 10) % 1000) / 1000) * (height - padding * 2);
  return { x, y };
}

function statusColor(status: string): string {
  switch (status.toLowerCase()) {
    case 'online':
      return 'var(--neon-green)';
    case 'offline':
      return 'var(--muted-2)';
    default:
      return 'var(--neon-yellow)';
  }
}

function osEmoji(os: string): string {
  const lower = os.toLowerCase();
  if (lower.includes('windows')) return '🪟';
  if (lower.includes('linux')) return '🐧';
  if (lower.includes('mac') || lower.includes('darwin')) return '🍎';
  return '💻';
}

const CANVAS_WIDTH = 900;
const CANVAS_HEIGHT = 500;
const NODE_RADIUS = 24;
const PADDING = 60;

const SERVER_POS: NodePosition = { x: CANVAS_WIDTH / 2, y: 50 };

export function MapPanel() {
  const [agents, setAgents] = useState<Agent[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  async function refresh() {
    setLoading(true);
    setError(null);
    try {
      setAgents(await listAgents());
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
    const t = window.setInterval(() => void refresh(), 8000);
    return () => window.clearInterval(t);
  }, []);

  const nodePositions = useMemo(() => {
    const map = new Map<string, NodePosition>();
    agents.forEach((a) => {
      map.set(a.id, hashToPosition(a.id, CANVAS_WIDTH, CANVAS_HEIGHT - 80, PADDING));
    });
    return map;
  }, [agents]);

  const selectedAgent = useMemo(
    () => agents.find((a) => a.id === selectedId) ?? null,
    [agents, selectedId],
  );

  const onlineCount = useMemo(
    () => agents.filter((a) => a.status.toLowerCase() === 'online').length,
    [agents],
  );

  return (
    <div className="content">
      <div className="grid2">
        <div className="card gamePanel neonBorder">
          <h3 className="cardTitle neonGlow">Network Topology</h3>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            <span className="tagBadge">🌐 {agents.length} nodes</span>
            <span className="tagBadge" style={{ borderColor: 'rgba(57,255,20,0.3)', color: 'var(--neon-green)' }}>
              ● {onlineCount} online
            </span>
            <span className="tagBadge">○ {agents.length - onlineCount} offline</span>
          </div>
        </div>
        <div className="card gamePanel">
          <h3 className="cardTitle">Map Controls</h3>
          <div className="row">
            <button className="btn" onClick={() => void refresh()} disabled={loading}>
              {loading ? 'Scanning...' : '🔄 Refresh Map'}
            </button>
            {selectedId ? (
              <button className="btn" onClick={() => setSelectedId(null)}>
                ✕ Deselect
              </button>
            ) : null}
          </div>
          {error ? <div style={{ marginTop: 8, color: 'var(--danger)', fontSize: 12 }}>{error}</div> : null}
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div
        className="card gamePanel neonBorder"
        style={{ padding: 0, overflow: 'hidden', position: 'relative' }}
      >
        <svg
          ref={svgRef}
          viewBox={`0 0 ${CANVAS_WIDTH} ${CANVAS_HEIGHT}`}
          width="100%"
          height={CANVAS_HEIGHT}
          style={{ display: 'block', background: 'rgba(0,0,0,0.3)' }}
        >
          <defs>
            <radialGradient id="serverGlow">
              <stop offset="0%" stopColor="rgba(0,240,255,0.3)" />
              <stop offset="100%" stopColor="rgba(0,240,255,0)" />
            </radialGradient>
            <radialGradient id="nodeGlowOnline">
              <stop offset="0%" stopColor="rgba(57,255,20,0.25)" />
              <stop offset="100%" stopColor="rgba(57,255,20,0)" />
            </radialGradient>
            <filter id="neonFilter">
              <feGaussianBlur in="SourceGraphic" stdDeviation="2" result="blur" />
              <feMerge>
                <feMergeNode in="blur" />
                <feMergeNode in="SourceGraphic" />
              </feMerge>
            </filter>
          </defs>

          <circle cx={SERVER_POS.x} cy={SERVER_POS.y} r={40} fill="url(#serverGlow)" />
          <circle
            cx={SERVER_POS.x}
            cy={SERVER_POS.y}
            r={18}
            fill="rgba(0,240,255,0.15)"
            stroke="var(--neon-cyan)"
            strokeWidth={2}
            filter="url(#neonFilter)"
          />
          <text
            x={SERVER_POS.x}
            y={SERVER_POS.y + 5}
            textAnchor="middle"
            fill="var(--neon-cyan)"
            fontSize={14}
          >
            C2
          </text>

          {agents.map((agent) => {
            const pos = nodePositions.get(agent.id);
            if (!pos) return null;
            const isOnline = agent.status.toLowerCase() === 'online';

            return (
              <line
                key={`line-${agent.id}`}
                x1={SERVER_POS.x}
                y1={SERVER_POS.y + 18}
                x2={pos.x}
                y2={pos.y - NODE_RADIUS}
                stroke={isOnline ? 'rgba(0,240,255,0.2)' : 'rgba(255,255,255,0.06)'}
                strokeWidth={isOnline ? 1.5 : 1}
                strokeDasharray={isOnline ? 'none' : '4 4'}
              />
            );
          })}

          {agents.map((agent) => {
            const pos = nodePositions.get(agent.id);
            if (!pos) return null;
            const isOnline = agent.status.toLowerCase() === 'online';
            const isSelected = selectedId === agent.id;

            return (
              <g
                key={agent.id}
                style={{ cursor: 'pointer' }}
                onClick={() => setSelectedId(isSelected ? null : agent.id)}
              >
                {isOnline ? (
                  <circle cx={pos.x} cy={pos.y} r={NODE_RADIUS + 12} fill="url(#nodeGlowOnline)" />
                ) : null}

                <circle
                  cx={pos.x}
                  cy={pos.y}
                  r={NODE_RADIUS}
                  fill={isSelected ? 'rgba(0,240,255,0.2)' : 'rgba(18,22,33,0.9)'}
                  stroke={isSelected ? 'var(--neon-cyan)' : statusColor(agent.status)}
                  strokeWidth={isSelected ? 2.5 : 1.5}
                  filter={isOnline ? 'url(#neonFilter)' : undefined}
                />

                <text
                  x={pos.x}
                  y={pos.y + 2}
                  textAnchor="middle"
                  fontSize={16}
                  dominantBaseline="middle"
                >
                  {osEmoji(agent.os)}
                </text>

                <text
                  x={pos.x}
                  y={pos.y + NODE_RADIUS + 14}
                  textAnchor="middle"
                  fill={isOnline ? 'var(--text)' : 'var(--muted-2)'}
                  fontSize={10}
                  fontFamily="var(--font-mono)"
                >
                  {agent.hostname.length > 14 ? agent.hostname.slice(0, 12) + '…' : agent.hostname}
                </text>

                <circle
                  cx={pos.x + NODE_RADIUS - 4}
                  cy={pos.y - NODE_RADIUS + 4}
                  r={4}
                  fill={statusColor(agent.status)}
                />
              </g>
            );
          })}

          {agents.length === 0 ? (
            <text
              x={CANVAS_WIDTH / 2}
              y={CANVAS_HEIGHT / 2 + 30}
              textAnchor="middle"
              fill="var(--muted)"
              fontSize={14}
            >
              No agents deployed. Deploy agents to populate the network map.
            </text>
          ) : null}
        </svg>
      </div>

      <AnimatePresence>
        {selectedAgent ? (
          <motion.div
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -8 }}
            transition={{ duration: 0.2 }}
          >
            <div style={{ height: 14 }} />
            <div className="card gamePanel neonBorder">
              <h3 className="cardTitle neonGlow">Node Intel: {selectedAgent.hostname}</h3>
              <div className="grid2">
                <div>
                  <div className="row" style={{ flexWrap: 'wrap', marginBottom: 10 }}>
                    <span className="tagBadge">{selectedAgent.os} / {selectedAgent.arch}</span>
                    <span className="tagBadge">{selectedAgent.ip}</span>
                    <span
                      className="tagBadge"
                      style={{
                        borderColor: selectedAgent.status.toLowerCase() === 'online'
                          ? 'rgba(57,255,20,0.3)'
                          : 'var(--game-border)',
                        color: statusColor(selectedAgent.status),
                      }}
                    >
                      ● {selectedAgent.status}
                    </span>
                    <span className="tagBadge">{selectedAgent.approval_status}</span>
                  </div>
                  <div style={{ fontSize: 12, color: 'var(--muted)' }}>
                    <span className="mono">User: {selectedAgent.user}</span>
                  </div>
                  <div style={{ fontSize: 12, color: 'var(--muted)', marginTop: 4 }}>
                    <span className="mono">Last seen: {selectedAgent.last_seen}</span>
                  </div>
                </div>
                <div>
                  <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 6 }}>Agent ID</div>
                  <div className="mono" style={{ fontSize: 11, wordBreak: 'break-all', color: 'var(--neon-cyan)' }}>
                    {selectedAgent.id}
                  </div>
                </div>
              </div>
            </div>
          </motion.div>
        ) : null}
      </AnimatePresence>
    </div>
  );
}
