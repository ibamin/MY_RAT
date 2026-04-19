import { useCallback, useEffect, useMemo, useState } from 'react';
import { AnimatePresence, motion } from 'framer-motion';
import { listScenarios, getScenario } from '../lib/api';
import type { ScenarioMeta, ScenarioDef, CharacterClass } from '../lib/types';
import { CharacterPortrait } from '../components/CharacterPortrait';
import { useSound } from '../hooks/useSound';

type BriefingLine = {
  speaker: string;
  speakerColor: string;
  text: string;
  portrait: string;
  characterClass: CharacterClass;
};

type BriefingChoice = {
  id: string;
  label: string;
};

type BriefingScene = {
  lines: BriefingLine[];
  choices: BriefingChoice[];
};

const OPERATOR_COLOR = 'var(--neon-cyan)';
const ANALYST_COLOR = 'var(--neon-purple)';
const COMMANDER_COLOR = 'var(--neon-yellow)';

function scenarioToBriefing(scenario: ScenarioDef): BriefingScene {
  const lines: BriefingLine[] = [];

  lines.push({
    speaker: 'COMMANDER',
    speakerColor: COMMANDER_COLOR,
    text: `Attention, Operator. New mission briefing incoming: "${scenario.title}".`,
    portrait: '🎖️',
    characterClass: 'commander',
  });

  lines.push({
    speaker: 'ANALYST',
    speakerColor: ANALYST_COLOR,
    text: `Mission ID: ${scenario.test_id}. Difficulty rating: ${'★'.repeat(scenario.difficulty)}${'☆'.repeat(Math.max(0, 5 - scenario.difficulty))}. Estimated completion: ${scenario.estimated_time_sec}s.`,
    portrait: '📊',
    characterClass: 'analyst',
  });

  if (scenario.steps.length > 0) {
    lines.push({
      speaker: 'COMMANDER',
      speakerColor: COMMANDER_COLOR,
      text: `This operation consists of ${scenario.steps.length} phase${scenario.steps.length > 1 ? 's' : ''}. Pay close attention.`,
      portrait: '🎖️',
      characterClass: 'commander',
    });

    scenario.steps.forEach((step, i) => {
      const actionCount = step.actions.length;
      const assertionCount = step.assertions.length;
      const requiredAssertions = step.assertions.filter((a) => a.required).length;

      lines.push({
        speaker: 'ANALYST',
        speakerColor: ANALYST_COLOR,
        text: `Phase ${i + 1}: "${step.name}" — ${actionCount} action${actionCount !== 1 ? 's' : ''}, ${assertionCount} assertion${assertionCount !== 1 ? 's' : ''} (${requiredAssertions} required).`,
        portrait: '📊',
        characterClass: 'analyst',
      });

      if (step.choices.length > 0) {
        lines.push({
          speaker: 'OPERATOR',
          speakerColor: OPERATOR_COLOR,
          text: `Decision point detected. ${step.choices.length} branch${step.choices.length > 1 ? 'es' : ''} available: ${step.choices.map((c) => `"${c.title}"`).join(', ')}.`,
          portrait: '🎯',
          characterClass: 'operator',
        });
      }
    });
  }

  lines.push({
    speaker: 'COMMANDER',
    speakerColor: COMMANDER_COLOR,
    text: 'Understood? Select your approach and deploy when ready. Good luck, Operator.',
    portrait: '🎖️',
    characterClass: 'commander',
  });

  const firstStepChoices = scenario.steps[0]?.choices ?? [];
  const choices: BriefingChoice[] = firstStepChoices.length > 0
    ? firstStepChoices.map((c) => ({ id: c.choice_id, label: c.title }))
    : [{ id: 'proceed', label: 'Proceed with mission' }];

  return { lines, choices };
}

function TypewriterText({ text, speed = 30, onComplete }: { text: string; speed?: number; onComplete?: () => void }) {
  const [displayed, setDisplayed] = useState('');
  const [done, setDone] = useState(false);

  useEffect(() => {
    setDisplayed('');
    setDone(false);
    let idx = 0;
    const interval = window.setInterval(() => {
      idx++;
      setDisplayed(text.slice(0, idx));
      if (idx >= text.length) {
        window.clearInterval(interval);
        setDone(true);
        onComplete?.();
      }
    }, speed);
    return () => window.clearInterval(interval);
  }, [text, speed, onComplete]);

  return (
    <span>
      {displayed}
      {!done ? <span style={{ opacity: 0.6 }}>▌</span> : null}
    </span>
  );
}

export function BriefingPanel() {
  const { playSfx } = useSound();
  const [scenarios, setScenarios] = useState<ScenarioMeta[]>([]);
  const [selectedScenarioId, setSelectedScenarioId] = useState<string | null>(null);
  const [scenarioDef, setScenarioDef] = useState<ScenarioDef | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [lineIndex, setLineIndex] = useState(0);
  const [lineComplete, setLineComplete] = useState(false);
  const [briefingStarted, setBriefingStarted] = useState(false);
  const [selectedChoice, setSelectedChoice] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    void listScenarios()
      .then((result) => {
        if (!cancelled) setScenarios(result);
      })
      .catch((e) => {
        if (!cancelled) setError(e instanceof Error ? e.message : String(e));
      })
      .finally(() => {
        if (!cancelled) setLoading(false);
      });
    return () => { cancelled = true; };
  }, []);

  const briefing = useMemo(
    () => (scenarioDef ? scenarioToBriefing(scenarioDef) : null),
    [scenarioDef],
  );

  const currentLine = briefing?.lines[lineIndex] ?? null;
  const isLastLine = briefing ? lineIndex >= briefing.lines.length - 1 : false;

  const handleScenarioSelect = useCallback(
    (id: string) => {
      setSelectedScenarioId(id);
      setBriefingStarted(false);
      setLineIndex(0);
      setLineComplete(false);
      setSelectedChoice(null);
      setLoading(true);
      void getScenario(id)
        .then((def) => {
          setScenarioDef(def);
        })
        .catch((e) => setError(e instanceof Error ? e.message : String(e)))
        .finally(() => setLoading(false));
    },
    [],
  );

  const advance = useCallback(() => {
    if (!lineComplete) {
      setLineComplete(true);
      return;
    }
    if (!isLastLine) {
      setLineIndex((prev) => prev + 1);
      setLineComplete(false);
    }
  }, [lineComplete, isLastLine]);

  const startBriefing = useCallback(() => {
    setLineIndex(0);
    setLineComplete(false);
    setSelectedChoice(null);
    setBriefingStarted(true);
  }, []);

  return (
    <div className="content">
      <div className="grid2">
        <div className="card gamePanel neonBorder">
          <h3 className="cardTitle neonGlow">Mission Briefing</h3>
          <div className="row">
            <span className="tagBadge">📋 {scenarios.length} scenarios</span>
            {selectedScenarioId ? (
              <span className="tagBadge" style={{ borderColor: 'rgba(191,90,242,0.3)', color: 'var(--neon-purple)' }}>
                Active: {selectedScenarioId.slice(0, 8)}…
              </span>
            ) : null}
          </div>
        </div>

        <div className="card gamePanel">
          <h3 className="cardTitle">Scenario Select</h3>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            {loading ? (
              <span style={{ color: 'var(--muted)' }}>Loading…</span>
            ) : (
              scenarios.map((s) => (
                <button
                  key={s.scenario_id}
                  className="btn"
                  style={{
                    borderColor:
                      selectedScenarioId === s.scenario_id
                        ? 'rgba(0, 240, 255, 0.5)'
                        : 'rgba(255, 255, 255, 0.18)',
                    fontSize: 12,
                  }}
                  onClick={() => { playSfx('click'); handleScenarioSelect(s.scenario_id); }}
                >
                  {s.test_id}
                </button>
              ))
            )}
          </div>
          {error ? <div style={{ marginTop: 8, color: 'var(--danger)', fontSize: 12 }}>{error}</div> : null}
        </div>
      </div>

      <div style={{ height: 14 }} />

      {scenarioDef && !briefingStarted ? (
        <motion.div
          initial={{ opacity: 0, y: 12 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.3 }}
        >
          <div className="card gamePanel neonBorder" style={{ textAlign: 'center', padding: 40 }}>
            <div style={{ fontSize: 40, marginBottom: 16 }}>📡</div>
            <div style={{ fontSize: 18, fontWeight: 700, color: 'var(--neon-cyan)', marginBottom: 8 }}>
              {scenarioDef.title}
            </div>
            <div style={{ color: 'var(--muted)', marginBottom: 6 }}>
              {scenarioDef.test_id} · v{scenarioDef.version}
            </div>
            <div className="difficultyStars" style={{ marginBottom: 20 }}>
              {'★'.repeat(scenarioDef.difficulty)}
              {'☆'.repeat(Math.max(0, 5 - scenarioDef.difficulty))}
            </div>
            <button className="btn" style={{ padding: '12px 32px', fontSize: 16 }} onClick={() => { playSfx('deploy'); startBriefing(); }}>
              ▶ Start Briefing
            </button>
          </div>
        </motion.div>
      ) : null}

      {briefingStarted && briefing && currentLine ? (
        <motion.div
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          transition={{ duration: 0.4 }}
        >
          <div
            className="card gamePanel"
            style={{
              minHeight: 300,
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'flex-end',
              padding: 0,
              overflow: 'hidden',
              position: 'relative',
            }}
          >
            <div
              style={{
                flex: 1,
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                padding: 40,
                background: 'radial-gradient(ellipse at center, rgba(0,240,255,0.03), transparent 70%)',
              }}
            >
              <AnimatePresence mode="wait">
                <motion.div
                  key={`portrait-${lineIndex}`}
                  initial={{ opacity: 0, scale: 0.8 }}
                  animate={{ opacity: 1, scale: 1 }}
                  exit={{ opacity: 0, scale: 0.8 }}
                  transition={{ duration: 0.25 }}
                >
                  <CharacterPortrait
                    characterClass={currentLine.characterClass}
                    seed={currentLine.speaker}
                    size="xl"
                    animated={false}
                  />
                </motion.div>
              </AnimatePresence>
            </div>

            <div
              className="dialogueBox"
              onClick={() => { playSfx('transition'); advance(); }}
              style={{
                cursor: 'pointer',
                borderTop: `1px solid ${currentLine.speakerColor}44`,
                borderRadius: 0,
                minHeight: 100,
              }}
            >
              <div className="dialogueSpeaker" style={{ color: currentLine.speakerColor }}>
                {currentLine.speaker}
              </div>
              <div className="dialogueText">
                {lineComplete ? (
                  currentLine.text
                ) : (
                  <TypewriterText
                    key={lineIndex}
                    text={currentLine.text}
                    speed={25}
                    onComplete={() => setLineComplete(true)}
                  />
                )}
              </div>

              <div
                style={{
                  marginTop: 10,
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                }}
              >
                <span className="mono" style={{ fontSize: 11, color: 'var(--muted-2)' }}>
                  {lineIndex + 1} / {briefing.lines.length}
                </span>
                {!isLastLine ? (
                  <span style={{ fontSize: 12, color: 'var(--muted)', animation: 'pulse 1.5s infinite' }}>
                    Click to continue ▶
                  </span>
                ) : null}
              </div>
            </div>

            {isLastLine && lineComplete ? (
              <motion.div
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.3, delay: 0.2 }}
                style={{ padding: '12px 20px', borderTop: '1px solid var(--game-border)' }}
              >
                <div style={{ fontSize: 12, color: 'var(--muted)', marginBottom: 8 }}>
                  SELECT APPROACH:
                </div>
                <div className="row" style={{ flexWrap: 'wrap' }}>
                  {briefing.choices.map((choice) => (
                    <button
                      key={choice.id}
                      className="btn"
                      style={{
                        borderColor:
                          selectedChoice === choice.id
                            ? 'rgba(57,255,20,0.5)'
                            : 'rgba(0,240,255,0.35)',
                        background:
                          selectedChoice === choice.id
                            ? 'linear-gradient(180deg, rgba(57,255,20,0.16), rgba(255,255,255,0.04))'
                            : undefined,
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        playSfx('click');
                        setSelectedChoice(choice.id);
                      }}
                    >
                      {choice.label}
                    </button>
                  ))}
                </div>
                {selectedChoice ? (
                  <motion.div
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    style={{ marginTop: 10, fontSize: 12, color: 'var(--neon-green)' }}
                  >
                    Approach locked: {briefing.choices.find((c) => c.id === selectedChoice)?.label ?? selectedChoice}. Ready to deploy.
                  </motion.div>
                ) : null}
              </motion.div>
            ) : null}
          </div>

          <div style={{ height: 10 }} />

          <div className="row" style={{ justifyContent: 'center' }}>
            <button
              className="btn"
              style={{ fontSize: 12 }}
              onClick={() => {
                setLineIndex(0);
                setLineComplete(false);
                setSelectedChoice(null);
              }}
            >
              ⏮ Restart
            </button>
            {lineIndex > 0 ? (
              <button
                className="btn"
                style={{ fontSize: 12 }}
                onClick={() => {
                  setLineIndex((prev) => Math.max(0, prev - 1));
                  setLineComplete(false);
                }}
              >
                ◀ Back
              </button>
            ) : null}
            {!isLastLine ? (
              <button className="btn" style={{ fontSize: 12 }} onClick={advance}>
                Skip ▶▶
              </button>
            ) : null}
          </div>
        </motion.div>
      ) : null}

      {!selectedScenarioId && !loading ? (
        <div className="dialogueBox" style={{ textAlign: 'center', padding: 40 }}>
          <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>SYSTEM</div>
          <div className="dialogueText" style={{ color: 'var(--muted)' }}>
            Select a scenario above to begin the mission briefing.
          </div>
        </div>
      ) : null}
    </div>
  );
}
