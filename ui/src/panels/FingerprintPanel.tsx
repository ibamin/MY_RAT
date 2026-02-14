import { useState } from 'react';
import { matchFingerprint } from '../lib/api';
import type { FingerprintCandidate } from '../lib/types';

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
        <div className="card">
          <h3 className="cardTitle">Offline Banner Fingerprint</h3>
          <div className="field">
            <div className="label">banner</div>
            <textarea
              className="input"
              placeholder="Paste a raw banner or headers (e.g. Server: nginx/1.24.0)"
              value={banner}
              onChange={(e) => setBanner(e.target.value)}
              rows={7}
              disabled={busy}
            />
          </div>
          <div className="row">
            <button className="btn" onClick={() => void submit()} disabled={busy || !banner.trim()}>
              {busy ? 'Matching…' : 'Match'}
            </button>
            {error ? <span style={{ color: 'var(--danger)' }}>{error}</span> : null}
          </div>
        </div>

        <div className="card">
          <h3 className="cardTitle">Candidates</h3>
          <div style={{ display: 'grid', gap: 10 }}>
            {candidates.map((c, idx) => (
              <div key={`${c.service}-${idx}`} className="card" style={{ padding: 12, background: 'rgba(255,255,255,0.02)' }}>
                <div className="row" style={{ justifyContent: 'space-between' }}>
                  <div style={{ fontWeight: 650 }}>
                    {c.service}
                    {c.product ? ` · ${c.product}` : ''}
                  </div>
                  <span className="pill">{Math.round(c.confidence * 100)}%</span>
                </div>
                <div style={{ color: 'var(--muted)' }}>
                  version: <span className="mono">{c.version || '-'}</span>
                </div>
              </div>
            ))}
            {candidates.length === 0 ? (
              <div style={{ color: 'var(--muted)' }}>
                No matches yet. Ensure `FINGERPRINT_RULES_PATH` points to a rules JSON.
              </div>
            ) : null}
          </div>
        </div>
      </div>
    </div>
  );
}
