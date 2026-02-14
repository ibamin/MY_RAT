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
      await postOperatorAction(props.runId, { type: 'approve_action', action_id: actionId, note: null });
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  async function selectChoice(choiceId: string) {
    try {
      await postOperatorAction(props.runId, { type: 'select_choice', choice_id: choiceId, note: null });
      await refresh();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  }

  useEffect(() => {
    void refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [props.runId]);

  return (
    <div className="card" style={{ marginTop: 14 }}>
      <div className="row" style={{ justifyContent: 'space-between' }}>
        <h3 className="cardTitle" style={{ margin: 0 }}>
          Run Detail
        </h3>
        <div className="row">
          <button className="btn" onClick={() => void refresh()} disabled={busy}>
            {busy ? 'Refreshing…' : 'Refresh'}
          </button>
          <button className="btn" onClick={() => props.onClose()}>
            Close
          </button>
        </div>
      </div>

      {error ? <div style={{ color: 'var(--danger)', marginTop: 10 }}>{error}</div> : null}

      {run ? (
        <div style={{ display: 'grid', gap: 12, marginTop: 12 }}>
          <div className="row" style={{ flexWrap: 'wrap' }}>
            <span className="pill">test_id {run.test_id}</span>
            {run.scenario_id ? <span className="pill">scenario_id {run.scenario_id}</span> : null}
            <span className="pill">status {run.status}</span>
            <span className="pill">agent {run.agent_id}</span>
          </div>

          <div className="grid2">
            <div className="card" style={{ padding: 12 }}>
              <div className="cardTitle">Steps</div>
              <div style={{ display: 'grid', gap: 8 }}>
                {steps.map((s) => (
                  <div key={s.id} className="row" style={{ justifyContent: 'space-between' }}>
                    <div>
                      <span className="mono">{s.idx}</span> · {s.name}
                    </div>
                    <span className="pill">{s.status}</span>
                  </div>
                ))}
                {steps.length === 0 ? <div style={{ color: 'var(--muted)' }}>No steps.</div> : null}
              </div>

              {scenario ? (
                <div style={{ marginTop: 14 }}>
                  <div className="cardTitle">Allowlisted Actions</div>
                  <div style={{ display: 'grid', gap: 10 }}>
                    {scenario.steps.map((st) => (
                      <div key={st.step_id} className="card" style={{ padding: 10, background: 'rgba(255,255,255,0.02)' }}>
                        <div style={{ fontWeight: 650 }}>{st.name}</div>
                        <div style={{ height: 8 }} />
                        <div className="row" style={{ flexWrap: 'wrap' }}>
                          {st.actions.map((a) => (
                            <button key={a.action_id} className="btn" onClick={() => void approveAction(a.action_id)} disabled={busy}>
                              Approve: {a.title}
                            </button>
                          ))}
                          {st.actions.length === 0 ? <span style={{ color: 'var(--muted)' }}>No actions.</span> : null}
                        </div>
                        {st.choices.length ? (
                          <>
                            <div style={{ height: 10 }} />
                            <div style={{ color: 'var(--muted)' }}>Choices</div>
                            <div className="row" style={{ flexWrap: 'wrap' }}>
                              {st.choices.map((c) => (
                                <button
                                  key={c.choice_id}
                                  className="btn"
                                  onClick={() => void selectChoice(c.choice_id)}
                                  disabled={busy}
                                >
                                  Select: {c.title}
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

            <div className="card" style={{ padding: 12 }}>
              <div className="cardTitle">Verdict</div>
              {verdict ? (
                <div style={{ display: 'grid', gap: 10 }}>
                  {verdict.steps.map((s) => (
                    <div key={s.step.id} className="card" style={{ padding: 10, background: 'rgba(255,255,255,0.02)' }}>
                      <div className="row" style={{ justifyContent: 'space-between' }}>
                        <div style={{ fontWeight: 650 }}>
                          {s.step.name}
                        </div>
                        <span className="pill">{s.verdict.verdict}</span>
                      </div>
                      {s.verdict.reason_code ? (
                        <div className="mono" style={{ color: 'var(--warn)', marginTop: 6 }}>
                          {s.verdict.reason_code}
                        </div>
                      ) : null}
                      {s.verdict.summary ? (
                        <div style={{ color: 'var(--muted)', marginTop: 6 }}>{s.verdict.summary}</div>
                      ) : null}
                      <div style={{ marginTop: 8, display: 'grid', gap: 6 }}>
                        {s.assertions.map((a) => (
                          <div key={a.id} className="row" style={{ justifyContent: 'space-between' }}>
                            <div style={{ color: 'var(--muted)' }}>{a.description}</div>
                            <span className="pill">{a.status}</span>
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
                <div style={{ color: 'var(--muted)' }}>No verdict.</div>
              )}
            </div>
          </div>

          <div className="grid2">
            <div className="card" style={{ padding: 12 }}>
              <div className="cardTitle">Evidence</div>
              <div style={{ display: 'grid', gap: 10 }}>
                {evidence.map((e) => (
                  <div key={e.id} className="card" style={{ padding: 10, background: 'rgba(255,255,255,0.02)' }}>
                    <div className="row" style={{ justifyContent: 'space-between' }}>
                      <div style={{ fontWeight: 650 }}>{e.kind}</div>
                      <span className="pill">{e.created_at}</span>
                    </div>
                    <div className="mono" style={{ marginTop: 6 }}>
                      locator: {e.locator}
                    </div>
                    <div className="mono" style={{ marginTop: 6 }}>
                      sha256: {e.sha256}
                    </div>
                  </div>
                ))}
                {evidence.length === 0 ? <div style={{ color: 'var(--muted)' }}>No evidence.</div> : null}
              </div>
            </div>

            <div className="card" style={{ padding: 12 }}>
              <div className="cardTitle">Timeline</div>
              <div style={{ display: 'grid', gap: 8, maxHeight: 340, overflow: 'auto' }}>
                {events.map((e) => (
                  <div key={e.id} className="row" style={{ justifyContent: 'space-between', gap: 14 }}>
                    <div className="mono" style={{ minWidth: 200, color: 'var(--muted)' }}>
                      {e.ts}
                    </div>
                    <div style={{ flex: 1 }}>{e.message}</div>
                    <span className="pill">{e.level}</span>
                  </div>
                ))}
                {events.length === 0 ? <div style={{ color: 'var(--muted)' }}>No events.</div> : null}
              </div>
            </div>
          </div>

          <div className="card" style={{ padding: 12 }}>
            <div className="cardTitle">Operator Log</div>
            <div style={{ display: 'grid', gap: 8, maxHeight: 260, overflow: 'auto' }}>
              {operatorLog.map((o) => (
                <div key={o.id} className="row" style={{ justifyContent: 'space-between', gap: 14 }}>
                  <div className="mono" style={{ minWidth: 200, color: 'var(--muted)' }}>
                    {o.ts}
                  </div>
                  <div style={{ flex: 1 }}>
                    <span className="mono">{o.type}</span>
                    {o.action_id ? <span className="mono"> action_id={o.action_id}</span> : null}
                    {o.choice_id ? <span className="mono"> choice_id={o.choice_id}</span> : null}
                    {o.note ? <span> · {o.note}</span> : null}
                  </div>
                </div>
              ))}
              {operatorLog.length === 0 ? <div style={{ color: 'var(--muted)' }}>No operator actions.</div> : null}
            </div>
          </div>
        </div>
      ) : (
        <div style={{ color: 'var(--muted)', marginTop: 12 }}>Loading…</div>
      )}
    </div>
  );
}
