import { useEffect, useState } from 'react';
import {
  getScenario,
  getRun,
  getRunVerdict,
  listRunEvidence,
  listRunEvents,
  listRunSteps,
  listOperatorActions,
  postOperatorAction,
} from '../lib/api';
import type {
  Evidence,
  Event,
  OperatorAction,
  Run,
  RunVerdictView,
  ScenarioDef,
  Step,
} from '../lib/types';
import { CombatResult } from '../components/CombatResult';

export function RunDetailPanel(props: { runId: string; onClose: () => void }) {
  const [run, setRun] = useState<Run | null>(null);
  const [steps, setSteps] = useState<Step[]>([]);
  const [events, setEvents] = useState<Event[]>([]);
  const [evidence, setEvidence] = useState<Evidence[]>([]);
  const [verdict, setVerdict] = useState<RunVerdictView | null>(null);
  const [scenario, setScenario] = useState<ScenarioDef | null>(null);
  const [operatorLog, setOperatorLog] = useState<OperatorAction[]>([]);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);

  async function refresh() {
    setBusy(true);
    setError(null);
    try {
      const r = await getRun(props.runId);
      const [st, ev, ed, vd, op] = await Promise.all([
        listRunSteps(props.runId),
        listRunEvents(props.runId),
        listRunEvidence(props.runId),
        getRunVerdict(props.runId),
        listOperatorActions(props.runId),
      ]);
      setRun(r);
      setSteps(st);
      setEvents(ev);
      setEvidence(ed);
      setVerdict(vd);
      setOperatorLog(op);

      if (r.scenario_id) {
        const s = await getScenario(r.scenario_id);
        setScenario(s);
      } else {
        setScenario(null);
      }
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setBusy(false);
    }
  }

  async function approveAction(actionId: string) {
    try {
      await postOperatorAction(props.runId, {
        type: 'approve_action',
        action_id: actionId,
        note: null,
      });
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function selectChoice(choiceId: string) {
    try {
      await postOperatorAction(props.runId, {
        type: 'select_choice',
        choice_id: choiceId,
        note: null,
      });
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.runId]);

  function stepStatusIcon(status: string): string {
    const s = status.toLowerCase();
    if (s === 'completed' || s === 'passed') return '✓';
    if (s === 'failed') return '✕';
    if (s === 'running' || s === 'in_progress') return '▶';
    return '○';
  }

  function stepStatusColor(status: string): string {
    const s = status.toLowerCase();
    if (s === 'completed' || s === 'passed') return 'var(--status-victory)';
    if (s === 'failed') return 'var(--status-defeat)';
    if (s === 'running' || s === 'in_progress') return 'var(--status-active)';
    return 'var(--muted-2)';
  }

  return (
    <div className="card" style={{ marginTop: 14, borderColor: 'var(--game-border)' }}>
      <div className="row" style={{ justifyContent: 'space-between' }}>
        <h3 className="cardTitle" style={{ margin: 0, color: 'var(--neon-cyan)' }}>
          ⚔ Battle Report
        </h3>
        <div className="row">
          <button className="btn" onClick={() => void refresh()} disabled={busy}>
            {busy ? '⏳ Loading…' : '🔄 Refresh'}
          </button>
          <button className="btn" onClick={() => props.onClose()}>
            ✕ Close
          </button>
        </div>
      </div>

      {error ? (
        <div style={{ color: 'var(--danger)', marginTop: 10 }}>{error}</div>
      ) : null}

      {run ? (
        <div style={{ display: 'grid', gap: 12, marginTop: 12 }}>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            <span className="tagBadge">{run.test_id}</span>
            {run.scenario_id ? (
              <span className="tagBadge" style={{ borderColor: 'rgba(191,90,242,0.3)', color: 'var(--neon-purple)' }}>
                {run.scenario_id}
              </span>
            ) : null}
            <span
              className="tagBadge"
              style={{
                borderColor:
                  run.status === 'completed'
                    ? 'rgba(57,255,20,0.3)'
                    : 'var(--game-border)',
                color:
                  run.status === 'completed'
                    ? 'var(--status-victory)'
                    : 'var(--neon-cyan)',
              }}
            >
              {run.status}
            </span>
            <span className="tagBadge">agent: {run.agent_id.slice(0, 8)}…</span>
          </div>

          <div className="grid2">
            <div className="card" style={{ padding: 12, borderColor: 'var(--game-border)' }}>
              <div className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
                Combat Phases
              </div>
              <div style={{ display: 'grid', gap: 8 }}>
                {steps.map((s) => (
                  <div
                    key={s.id}
                    className="row"
                    style={{ justifyContent: 'space-between' }}
                  >
                    <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
                      <span
                        style={{
                          color: stepStatusColor(s.status),
                          fontWeight: 700,
                          fontSize: 14,
                          width: 20,
                          textAlign: 'center',
                        }}
                      >
                        {stepStatusIcon(s.status)}
                      </span>
                      <span className="mono" style={{ color: 'var(--muted)' }}>
                        {s.idx}
                      </span>
                      <span>{s.name}</span>
                    </div>
                    <span
                      className="pill"
                      style={{
                        borderColor: stepStatusColor(s.status),
                        color: stepStatusColor(s.status),
                      }}
                    >
                      {s.status}
                    </span>
                  </div>
                ))}
                {steps.length === 0 ? (
                  <div style={{ color: 'var(--muted)' }}>No combat phases.</div>
                ) : null}
              </div>

              {scenario ? (
                <div style={{ marginTop: 14 }}>
                  <div className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
                    Tactical Actions
                  </div>
                  <div style={{ display: 'grid', gap: 10 }}>
                    {scenario.steps.map((st) => (
                      <div
                        key={st.step_id}
                        className="card"
                        style={{
                          padding: 10,
                          background: 'rgba(0,240,255,0.02)',
                          borderColor: 'var(--game-border)',
                        }}
                      >
                        <div style={{ fontWeight: 650 }}>{st.name}</div>
                        <div style={{ height: 8 }} />
                        <div className="row" style={{ flexWrap: 'wrap' }}>
                          {st.actions.map((a) => (
                            <button
                              key={a.action_id}
                              className="btn"
                              style={{ borderColor: 'rgba(57,255,20,0.3)' }}
                              onClick={() => void approveAction(a.action_id)}
                              disabled={busy}
                            >
                              ✓ {a.title}
                            </button>
                          ))}
                          {st.actions.length === 0 ? (
                            <span style={{ color: 'var(--muted)' }}>No actions.</span>
                          ) : null}
                        </div>
                        {st.choices.length > 0 ? (
                          <>
                            <div style={{ height: 10 }} />
                            <div style={{ color: 'var(--muted)', fontSize: 12 }}>
                              Branching Options
                            </div>
                            <div className="row" style={{ flexWrap: 'wrap' }}>
                              {st.choices.map((c) => (
                                <button
                                  key={c.choice_id}
                                  className="btn"
                                  style={{ borderColor: 'rgba(0,240,255,0.3)' }}
                                  onClick={() => void selectChoice(c.choice_id)}
                                  disabled={busy}
                                >
                                  ▶ {c.title}
                                </button>
                              ))}
                            </div>
                          </>
                        ) : null}
                      </div>
                    ))}
                  </div>
                </div>
              ) : null}
            </div>

            <div className="card" style={{ padding: 12, borderColor: 'var(--game-border)' }}>
              <div className="cardTitle" style={{ color: 'var(--neon-green)' }}>
                Verdict
              </div>
              {verdict ? (
                <div style={{ display: 'grid', gap: 10 }}>
                  {verdict.steps.map((s) => (
                    <div key={s.step.id}>
                      <CombatResult
                        verdict={s.verdict.verdict}
                        stepName={s.step.name}
                        summary={s.verdict.summary || undefined}
                      />
                      {s.verdict.reason_code ? (
                        <div
                          className="mono"
                          style={{ color: 'var(--warn)', marginTop: 6, fontSize: 12 }}
                        >
                          {s.verdict.reason_code}
                        </div>
                      ) : null}
                      <div style={{ marginTop: 8, display: 'grid', gap: 6 }}>
                        {s.assertions.map((a) => (
                          <div
                            key={a.id}
                            className="row"
                            style={{ justifyContent: 'space-between' }}
                          >
                            <div style={{ color: 'var(--muted)', fontSize: 13 }}>
                              {a.description}
                            </div>
                            <span
                              className="pill"
                              style={{
                                borderColor:
                                  a.status === 'passed'
                                    ? 'rgba(57,255,20,0.3)'
                                    : a.status === 'failed'
                                      ? 'rgba(255,45,149,0.3)'
                                      : 'var(--stroke)',
                                color:
                                  a.status === 'passed'
                                    ? 'var(--status-victory)'
                                    : a.status === 'failed'
                                      ? 'var(--status-defeat)'
                                      : undefined,
                              }}
                            >
                              {a.status}
                            </span>
                          </div>
                        ))}
                        {s.assertions.length === 0 ? (
                          <div style={{ color: 'var(--muted)' }}>No assertions.</div>
                        ) : null}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <div style={{ color: 'var(--muted)' }}>No verdict yet.</div>
              )}
            </div>
          </div>

          <div className="grid2">
            <div className="card" style={{ padding: 12, borderColor: 'var(--game-border)' }}>
              <div className="cardTitle" style={{ color: 'var(--neon-purple)' }}>
                Evidence
              </div>
              <div style={{ display: 'grid', gap: 10 }}>
                {evidence.map((e) => (
                  <div
                    key={e.id}
                    className="card"
                    style={{
                      padding: 10,
                      background: 'rgba(191,90,242,0.03)',
                      borderColor: 'rgba(191,90,242,0.15)',
                    }}
                  >
                    <div
                      className="row"
                      style={{ justifyContent: 'space-between' }}
                    >
                      <div style={{ fontWeight: 650 }}>{e.kind}</div>
                      <span className="tagBadge">{e.created_at}</span>
                    </div>
                    <div className="mono" style={{ marginTop: 6, fontSize: 12 }}>
                      locator: {e.locator}
                    </div>
                    <div
                      className="mono"
                      style={{ marginTop: 4, fontSize: 12, color: 'var(--muted)' }}
                    >
                      sha256: {e.sha256}
                    </div>
                  </div>
                ))}
                {evidence.length === 0 ? (
                  <div style={{ color: 'var(--muted)' }}>No evidence collected.</div>
                ) : null}
              </div>
            </div>

            <div className="card" style={{ padding: 12, borderColor: 'var(--game-border)' }}>
              <div className="cardTitle" style={{ color: 'var(--neon-yellow)' }}>
                Timeline
              </div>
              <div
                className="combatLog"
                style={{ display: 'grid', gap: 8 }}
              >
                {events.map((e) => (
                  <div
                    key={e.id}
                    className="row"
                    style={{ justifyContent: 'space-between', gap: 14 }}
                  >
                    <div
                      className="mono"
                      style={{ minWidth: 160, color: 'var(--muted)', fontSize: 12 }}
                    >
                      {e.ts}
                    </div>
                    <div style={{ flex: 1, fontSize: 13 }}>{e.message}</div>
                    <span
                      className="pill"
                      style={{
                        borderColor:
                          e.level === 'error'
                            ? 'rgba(255,45,149,0.3)'
                            : e.level === 'warn'
                              ? 'rgba(255,225,86,0.3)'
                              : 'var(--stroke)',
                        color:
                          e.level === 'error'
                            ? 'var(--status-defeat)'
                            : e.level === 'warn'
                              ? 'var(--status-pending)'
                              : 'var(--neon-cyan)',
                      }}
                    >
                      {e.level}
                    </span>
                  </div>
                ))}
                {events.length === 0 ? (
                  <div style={{ color: 'var(--muted)' }}>No events recorded.</div>
                ) : null}
              </div>
            </div>
          </div>

          <div className="card" style={{ padding: 12, borderColor: 'var(--game-border)' }}>
            <div className="cardTitle" style={{ color: 'var(--neon-cyan)' }}>
              Operator Log
            </div>
            <div
              className="combatLog"
              style={{ display: 'grid', gap: 8, maxHeight: 260 }}
            >
              {operatorLog.map((o) => (
                <div
                  key={o.id}
                  className="row"
                  style={{ justifyContent: 'space-between', gap: 14 }}
                >
                  <div
                    className="mono"
                    style={{ minWidth: 160, color: 'var(--muted)', fontSize: 12 }}
                  >
                    {o.ts}
                  </div>
                  <div style={{ flex: 1, fontSize: 13 }}>
                    <span className="tagBadge" style={{ marginRight: 8 }}>{o.type}</span>
                    {o.action_id ? (
                      <span className="mono"> action={o.action_id}</span>
                    ) : null}
                    {o.choice_id ? (
                      <span className="mono"> choice={o.choice_id}</span>
                    ) : null}
                    {o.note ? <span> · {o.note}</span> : null}
                  </div>
                </div>
              ))}
              {operatorLog.length === 0 ? (
                <div style={{ color: 'var(--muted)' }}>No operator actions recorded.</div>
              ) : null}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ color: 'var(--muted)', marginTop: 12 }}>Loading battle data…</div>
      )}
    </div>
  );
}
