import { motion } from 'framer-motion';
import type { Agent } from '../lib/types';
import { osToClass } from '../lib/types';
import { CharacterPortrait } from './CharacterPortrait';

function getStatusColor(status: string): string {
  return status.toLowerCase() === 'online' ? 'var(--status-victory)' : 'var(--muted-2)';
}

function agentStatus(status: string): 'online' | 'offline' {
  return status.toLowerCase() === 'online' ? 'online' : 'offline';
}

export function CharacterCard(props: {
  agent: Agent;
  selected?: boolean;
  onClick?: () => void;
}) {
  const { agent, selected, onClick } = props;
  return (
    <motion.div
      className={`characterCard${selected ? ' characterCardSelected' : ''}`}
      onClick={onClick}
      whileHover={{ scale: 1.03 }}
      transition={{ type: 'spring', stiffness: 400, damping: 25 }}
      style={{ cursor: onClick ? 'pointer' : undefined }}
    >
      <div style={{ display: 'flex', gap: 12, alignItems: 'center', marginBottom: 12 }}>
        <CharacterPortrait
          characterClass={osToClass(agent.os)}
          seed={agent.id || agent.hostname}
          size="md"
          status={agentStatus(agent.status)}
          animated={false}
        />
        <div>
          <div style={{ fontWeight: 700, fontSize: 16 }}>{agent.hostname}</div>
          <div className="mono" style={{ fontSize: 12, color: 'var(--muted)' }}>
            {agent.user}
          </div>
        </div>
      </div>
      <div style={{ display: 'grid', gap: 6, marginBottom: 12 }}>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13 }}>
          <span style={{ color: 'var(--muted)' }}>IP</span>
          <span className="mono">{agent.ip}</span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13 }}>
          <span style={{ color: 'var(--muted)' }}>Arch</span>
          <span className="mono">
            {agent.os}/{agent.arch}
          </span>
        </div>
        <div style={{ display: 'flex', justifyContent: 'space-between', fontSize: 13 }}>
          <span style={{ color: 'var(--muted)' }}>Last Seen</span>
          <span className="mono" style={{ fontSize: 12 }}>
            {agent.last_seen}
          </span>
        </div>
      </div>
      <div style={{ display: 'flex', gap: 8 }}>
        <span
          className="pill"
          style={{
            borderColor: getStatusColor(agent.status),
            color: getStatusColor(agent.status),
          }}
        >
          {agent.status}
        </span>
        <span className="tagBadge">{agent.approval_status}</span>
      </div>
    </motion.div>
  );
}
