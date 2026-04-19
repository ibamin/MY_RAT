import { useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { checkAchievements, getAchievementProgress, listAchievements } from '../lib/api';
import type { AchievementCategory, AchievementStatus } from '../lib/types';
import { useSound } from '../hooks/useSound';

type CategoryFilter = 'all' | AchievementCategory;

const CATEGORY_OPTIONS: { id: CategoryFilter; label: string }[] = [
  { id: 'all', label: 'All' },
  { id: 'combat', label: 'Combat' },
  { id: 'recon', label: 'Recon' },
  { id: 'stealth', label: 'Stealth' },
  { id: 'mastery', label: 'Mastery' },
];

function readDependencies(requirementValue: string): string[] {
  try {
    const parsed: unknown = JSON.parse(requirementValue);
    if (!parsed || typeof parsed !== 'object') {
      return [];
    }

    const dependsOn = (parsed as { depends_on?: unknown }).depends_on;
    if (!Array.isArray(dependsOn)) {
      return [];
    }

    return dependsOn.filter((item): item is string => typeof item === 'string');
  } catch {
    return [];
  }
}

export function AchievementsPanel() {
  const { playSfx } = useSound();
  const [achievements, setAchievements] = useState<AchievementStatus[]>([]);
  const [loading, setLoading] = useState(false);
  const [checking, setChecking] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [filter, setFilter] = useState<CategoryFilter>('all');
  const [statusNote, setStatusNote] = useState<string>('');

  async function refresh() {
    setLoading(true);
    setError(null);

    try {
      const [items, progress] = await Promise.all([listAchievements(), getAchievementProgress()]);
      const progressById = new Map(progress.map((item) => [item.achievement.id, item]));

      const merged = items.map((item) => {
        const progressItem = progressById.get(item.achievement.id);
        if (!progressItem) {
          return item;
        }
        return {
          ...item,
          progress: progressItem.progress,
          unlocked: progressItem.unlocked,
          unlocked_at: progressItem.unlocked_at,
        };
      });

      setAchievements(merged);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }

  useEffect(() => {
    void refresh();
  }, []);

  const filtered = useMemo(
    () =>
      achievements.filter((item) =>
        filter === 'all' ? true : item.achievement.category === filter,
      ),
    [achievements, filter],
  );

  const unlockedCount = useMemo(() => achievements.filter((item) => item.unlocked).length, [achievements]);

  const achievementNameById = useMemo(() => {
    return new Map(achievements.map((item) => [item.achievement.id, item.achievement.name]));
  }, [achievements]);

  const treeNodes = useMemo(
    () =>
      achievements.map((item) => ({
        id: item.achievement.id,
        name: item.achievement.name,
        icon: item.achievement.icon,
        unlocked: item.unlocked,
        dependencies: readDependencies(item.achievement.requirement_value),
      })),
    [achievements],
  );

  return (
    <div className="content">
      <div className="grid2">
        <div className="card gamePanel neonBorder">
          <h3 className="cardTitle neonGlow">Achievement Matrix</h3>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            <span className="tagBadge">🏆 {unlockedCount}/{achievements.length} unlocked</span>
            <span className="tagBadge">📚 {filtered.length} shown</span>
          </div>
        </div>
        <div className="card gamePanel">
          <h3 className="cardTitle">Controls</h3>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            {CATEGORY_OPTIONS.map((option) => (
              <button
                key={option.id}
                className="btn"
                style={{
                  borderColor:
                    filter === option.id ? 'rgba(0, 240, 255, 0.5)' : 'rgba(255, 255, 255, 0.18)',
                }}
                onClick={() => { playSfx('click'); setFilter(option.id); }}
              >
                {option.label}
              </button>
            ))}
            <button
              className="btn"
              onClick={() => void refresh()}
              disabled={loading}
              style={{ marginLeft: 'auto' }}
            >
              {loading ? 'Syncing...' : 'Refresh'}
            </button>
            <button
              className="btn"
              onClick={() => {
                setChecking(true);
                setStatusNote('');
                playSfx('click');
                void checkAchievements()
                  .then((result) => {
                    if (result.unlocked_achievement_ids.length > 0) {
                      playSfx('unlock');
                      setStatusNote(
                        `Unlocked: ${result.unlocked_achievement_ids.join(', ')}`,
                      );
                    } else {
                      setStatusNote(
                        `Checked ${result.checked_runs} passed runs. No new unlocks.`,
                      );
                    }
                    return refresh();
                  })
                  .catch((e) => {
                    setError(e instanceof Error ? e.message : String(e));
                  })
                  .finally(() => setChecking(false));
              }}
              disabled={checking}
            >
              {checking ? 'Checking...' : 'Check Achievements'}
            </button>
          </div>
          {statusNote ? (
            <div style={{ marginTop: 10, color: 'var(--neon-green)', fontSize: 12 }}>{statusNote}</div>
          ) : null}
          {error ? <div style={{ marginTop: 10, color: 'var(--danger)', fontSize: 12 }}>{error}</div> : null}
        </div>
      </div>

      <div style={{ height: 14 }} />

      <div className="achievementsGrid">
        <AnimatePresence>
          {filtered.map((item, index) => {
            const deps = readDependencies(item.achievement.requirement_value);
            return (
              <motion.div
                key={item.achievement.id}
                className={`missionCard gamePanel ${item.unlocked ? 'neonBorder' : 'achievementLocked'}`}
                initial={{ opacity: 0, y: 12 }}
                animate={{ opacity: 1, y: 0 }}
                exit={{ opacity: 0, y: -8 }}
                transition={{ duration: 0.24, delay: index * 0.02 }}
              >
                <div className="row" style={{ justifyContent: 'space-between', alignItems: 'flex-start' }}>
                  <div className="row" style={{ alignItems: 'center' }}>
                    <div className="achievementIcon">{item.achievement.icon}</div>
                    <div>
                      <div style={{ fontWeight: 700, color: item.unlocked ? 'var(--neon-cyan)' : 'var(--text)' }}>
                        {item.achievement.name}
                      </div>
                      <div style={{ fontSize: 12, color: 'var(--muted)' }}>{item.achievement.description}</div>
                    </div>
                  </div>
                  <span className="tagBadge">{item.achievement.category}</span>
                </div>

                <div style={{ height: 10 }} />
                <div className="statBar">
                  <div className="statBarFill" style={{ width: `${Math.max(0, Math.min(100, item.progress))}%` }} />
                </div>
                <div className="row" style={{ justifyContent: 'space-between', marginTop: 8 }}>
                  <span style={{ fontSize: 12, color: 'var(--muted)' }}>{item.progress}%</span>
                  <span style={{ fontSize: 12, color: item.unlocked ? 'var(--status-victory)' : 'var(--status-pending)' }}>
                    {item.unlocked ? 'UNLOCKED' : 'LOCKED'}
                  </span>
                </div>

                {deps.length > 0 ? (
                  <div style={{ marginTop: 10 }}>
                    <div style={{ fontSize: 11, color: 'var(--muted-2)', marginBottom: 6 }}>Requires</div>
                    <div className="row" style={{ flexWrap: 'wrap' }}>
                      {deps.map((dep) => (
                        <span key={dep} className="tagBadge">
                          {achievementNameById.get(dep) ?? dep}
                        </span>
                      ))}
                    </div>
                  </div>
                ) : null}
              </motion.div>
            );
          })}
        </AnimatePresence>
      </div>

      <div style={{ height: 14 }} />

      <div className="card gamePanel neonBorder">
        <h3 className="cardTitle neonGlow">Skill Tree</h3>
        <div className="skillTreeGrid">
          {treeNodes.map((node) => (
            <div key={node.id} className={`skillNode ${node.unlocked ? 'skillNodeUnlocked' : ''}`}>
              <div className="row" style={{ justifyContent: 'space-between' }}>
                <strong style={{ fontSize: 13 }}>
                  {node.icon} {node.name}
                </strong>
                <span style={{ fontSize: 11, color: node.unlocked ? 'var(--status-victory)' : 'var(--muted)' }}>
                  {node.unlocked ? 'LIVE' : 'LOCKED'}
                </span>
              </div>
              <div style={{ marginTop: 6, fontSize: 12, color: 'var(--muted)' }}>
                {node.dependencies.length === 0
                  ? 'Root node'
                  : `Depends on ${node.dependencies.length} node${node.dependencies.length === 1 ? '' : 's'}`}
              </div>
              {node.dependencies.length > 0 ? (
                <div className="skillDependencyList">
                  {node.dependencies.map((dep) => (
                    <div key={`${node.id}-${dep}`} className="skillDependencyItem">
                      {achievementNameById.get(dep) ?? dep} → {node.name}
                    </div>
                  ))}
                </div>
              ) : null}
            </div>
          ))}
        </div>
      </div>
    </div>
  );
}
