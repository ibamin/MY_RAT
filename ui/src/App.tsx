import { useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { getServerUrl } from './lib/api';
import { useSound } from './hooks/useSound';
import { AgentsPanel } from './panels/AgentsPanel';
import { AchievementsPanel } from './panels/AchievementsPanel';
import { AIScriptPanel } from './panels/AIScriptPanel';
import { BriefingPanel } from './panels/BriefingPanel';
import { EventsPanel } from './panels/EventsPanel';
import { FingerprintPanel } from './panels/FingerprintPanel';
import { GroupsPanel } from './panels/GroupsPanel';
import { MapPanel } from './panels/MapPanel';
import { MissionsPanel } from './panels/MissionsPanel';
import { RunsPanel } from './panels/RunsPanel';

type View =
  | 'operations'
  | 'roster'
  | 'squads'
  | 'missions'
  | 'briefing'
  | 'combat-log'
  | 'intel'
  | 'map'
  | 'achievements'
  | 'ai-script';

const NAV_ITEMS: { view: View; icon: string; label: string }[] = [
  { view: 'operations', icon: '🎯', label: 'Operations' },
  { view: 'roster', icon: '👥', label: 'Roster' },
  { view: 'squads', icon: '🔗', label: 'Squads' },
  { view: 'missions', icon: '📋', label: 'Missions' },
  { view: 'briefing', icon: '📖', label: 'Briefing' },
  { view: 'combat-log', icon: '⚡', label: 'Combat Log' },
  { view: 'intel', icon: '🔍', label: 'Intel' },
  { view: 'map', icon: '🗺️', label: 'Map' },
  { view: 'achievements', icon: '🏆', label: 'Achievements' },
  { view: 'ai-script', icon: '🤖', label: 'AI Script' },
];

const VIEW_TITLES: Record<View, string> = {
  operations: 'Operations',
  roster: 'Agent Roster',
  squads: 'Squad Management',
  missions: 'Mission Queue',
  briefing: 'Mission Briefing',
  'combat-log': 'Combat Log',
  intel: 'Intel Center',
  map: 'Network Map',
  achievements: 'Achievements',
  'ai-script': 'AI Script Generator',
};

export default function App() {
  const [view, setView] = useState<View>('operations');
  const [selectedRunId, setSelectedRunId] = useState<string | null>(null);
  const { playSfx, muted, toggleMute } = useSound();

  function navigate(v: View) {
    if (v !== view) {
      playSfx('transition');
      setView(v);
    }
  }

  return (
    <div className="app">
      <aside className="sidebar">
        <div className="brand">
          <div
            className="brandTitle"
            style={{
              color: 'var(--neon-cyan)',
              textShadow: '0 0 10px rgba(0,240,255,0.3)',
            }}
          >
            SHADOW PROTOCOL
          </div>
          <div className="brandBadge">v0.1</div>
        </div>

        <nav className="nav">
          {NAV_ITEMS.map((item) => (
            <button
              key={item.view}
              className={`gameNavBtn${view === item.view ? ' gameNavBtnActive' : ''}`}
              onClick={() => navigate(item.view)}
              onMouseEnter={() => playSfx('hover')}
            >
              <span style={{ fontSize: 18 }}>{item.icon}</span>
              <span>{item.label}</span>
            </button>
          ))}
        </nav>

        <div className="sidebarFooter">
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: 8 }}>
            <div className="mono" style={{ color: 'var(--neon-cyan)', fontSize: 11 }}>
              SERVER: {getServerUrl()}
            </div>
            <button
              className="muteToggle"
              onClick={() => { playSfx('click'); toggleMute(); }}
              title={muted ? 'Unmute' : 'Mute'}
              aria-label={muted ? 'Unmute' : 'Mute'}
            >
              {muted ? '🔇' : '🔊'}
            </button>
          </div>
          <div style={{ fontSize: 11 }}>TACTICAL INTERFACE ONLINE</div>
        </div>
      </aside>

      <main className="main scanline">
        <div className="topbar">
          <div className="topbarTitle">{VIEW_TITLES[view]}</div>
          <div className="topbarMeta">v0.1 · TACTICAL INTERFACE</div>
        </div>

        <AnimatePresence mode="wait">
          <motion.div
            key={view}
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -10 }}
            transition={{ duration: 0.2 }}
            style={{ height: '100%', overflow: 'auto' }}
          >
            {view === 'operations' ? (
              <MissionsPanel
                onQueuedRun={(runId) => {
                  setSelectedRunId(runId);
                  setView('missions');
                }}
              />
            ) : null}
            {view === 'roster' ? <AgentsPanel /> : null}
            {view === 'squads' ? <GroupsPanel /> : null}
            {view === 'missions' ? (
              <RunsPanel
                selectedRunId={selectedRunId}
                onSelectRun={(runId) => setSelectedRunId(runId)}
              />
            ) : null}
            {view === 'combat-log' ? <EventsPanel /> : null}
            {view === 'briefing' ? <BriefingPanel /> : null}
            {view === 'intel' ? <FingerprintPanel /> : null}
            {view === 'map' ? <MapPanel /> : null}
            {view === 'achievements' ? <AchievementsPanel /> : null}
            {view === 'ai-script' ? <AIScriptPanel /> : null}
          </motion.div>
        </AnimatePresence>
      </main>
    </div>
  );
}
