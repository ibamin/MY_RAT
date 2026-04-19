import { useState } from 'react';
import { matchFingerprint } from '../lib/api';
import type { FingerprintCandidate } from '../lib/types';
import { motion, AnimatePresence } from 'framer-motion';

export function FingerprintPanel() {
  const [banner, setBanner] = useState('');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [candidates, setCandidates] = useState<FingerprintCandidate[]>([]);

  async function submit() {
    setBusy(true);
    setError(null);
    try {
      const res = await matchFingerprint({ banner, limit: 10 });
      setCandidates(res.candidates);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
      setCandidates([]);
    } finally {
      setBusy(false);
    }
  }

  return (
    <div className="content">
      <div className="grid2">
        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
            🔍 Signal Analysis
          </h3>
          <div className="field">
            <div className="label" style={{ color: 'var(--neon-cyan)' }}>
              📡 Captured Signal
            </div>
            <textarea
              className="input"
              placeholder="Paste a raw banner or headers (e.g. Server: nginx/1.24.0)"
              value={banner}
              onChange={(e) => setBanner(e.target.value)}
              rows={7}
              disabled={busy}
              style={{ borderColor: 'var(--game-border)' }}
            />
          </div>
          <div className="row">
            <button
              className="btn"
              style={{
                background: 'linear-gradient(180deg, rgba(0,240,255,0.2), rgba(0,240,255,0.06))',
                borderColor: 'rgba(0,240,255,0.4)',
              }}
              onClick={() => void submit()}
              disabled={busy || !banner.trim()}
            >
              {busy ? '⏳ Analyzing…' : '🔬 Decode Signal'}
            </button>
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card" style={{ borderColor: 'var(--game-border)' }}>
          <h3 className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
            🎯 Intel Report
          </h3>
          <div style={{ display: 'grid', gap: 10 }}>
            <AnimatePresence>
              {candidates.map((c, idx) => {
                const pct = Math.round(c.confidence * 100);
                const barColor = pct >= 80 ? 'var(--status-victory)' : pct >= 50 ? 'var(--neon-yellow)' : 'var(--neon-pink)';
                return (
                  <motion.div
                    key={`${c.service}-${idx}`}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -10 }}
                    transition={{ duration: 0.25, delay: idx * 0.06 }}
                    className="card"
                    style={{ padding: 14, background: 'rgba(255,255,255,0.02)', borderColor: 'var(--game-border)' }}
                  >
                    <div className="row" style={{ justifyContent: 'space-between', marginBottom: 8 }}>
                      <div style={{ fontWeight: 650, color: 'var(--neon-cyan)' }}>
                        {c.service}
                        {c.product ? ` · ${c.product}` : ''}
                      </div>
                      <span
                        className="tagBadge"
                        style={{
                          borderColor: barColor,
                          color: barColor,
                        }}
                      >
                        {pct}% match
                      </span>
                    </div>
                    <div style={{ color: 'var(--muted)', fontSize: 12, marginBottom: 8 }}>
                      version: <span className="mono" style={{ color: 'var(--neon-green)' }}>{c.version || 'unknown'}</span>
                    </div>
                    <div className="statBar">
                      <div
                        className="statBarFill"
                        style={{
                          width: `${pct}%`,
                          background: `linear-gradient(90deg, ${barColor}, transparent)`,
                        }}
                      />
                    </div>
                  </motion.div>
                );
              })}
            </AnimatePresence>
            {candidates.length === 0 ? (
              <div className="dialogueBox">
                <div className="dialogueSpeaker" style={{ color: 'var(--neon-cyan)' }}>INTEL</div>
                <div className="dialogueText" style={{ color: 'var(--muted)' }}>
                  No signals decoded yet. Paste a captured banner and run analysis.
                  Ensure <span className="mono">FINGERPRINT_RULES_PATH</span> points to a valid rules file.
                </div>
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
