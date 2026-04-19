import type { ScenarioMeta } from '../lib/types';

function renderStars(difficulty: number): string {
  const max = 5;
  const filled = Math.min(Math.max(Math.round(difficulty), 0), max);
  return '★'.repeat(filled) + '☆'.repeat(max - filled);
}

export function MissionBriefing(props: {
  scenario: ScenarioMeta;
  onStart?: () => void;
  disabled?: boolean;
}) {
  const { scenario, onStart, disabled } = props;
  return (
    <div className="missionCard">
      <div style={{ fontWeight: 700, fontSize: 16, marginBottom: 8 }}>{scenario.title}</div>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap', marginBottom: 10 }}>
        <span className="pill">{scenario.test_id}</span>
        <span className="difficultyStars">{renderStars(scenario.difficulty)}</span>
        <span className="tagBadge">v{scenario.version}</span>
      </div>
      <div style={{ color: 'var(--muted)', fontSize: 13, marginBottom: 12 }}>
        Estimated: <span className="mono">{scenario.estimated_time_sec}s</span>
      </div>
      {onStart ? (
        <button
          className="btn"
          onClick={onStart}
          disabled={disabled}
          style={{
            background: 'linear-gradient(180deg, rgba(0,240,255,0.2), rgba(0,240,255,0.06))',
            borderColor: 'rgba(0,240,255,0.4)',
          }}
        >
          {disabled ? 'Deploying…' : 'Deploy'}
        </button>
      ) : null}
    </div>
  );
}
