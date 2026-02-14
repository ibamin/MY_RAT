import type {
  Agent,
  Event,
  FingerprintCandidate,
  Run,
  ScenarioMeta,
  ScenarioDef,
  Step,
  Evidence,
  RunVerdictView,
  OperatorAction,
} from './types';

const serverUrl =
  (import.meta as unknown as { env: Record<string, string | undefined> }).env
    .VITE_SERVER_URL || 'http://127.0.0.1:3000';

async function http<T>(path: string, init?: RequestInit): Promise<T> {
  const res = await fetch(`${serverUrl}${path}`, {
    ...init,
    headers: {
      'content-type': 'application/json',
      ...(init?.headers || {}),
    },
  });

  if (!res.ok) {
    const text = await res.text().catch(() => '');
    throw new Error(`${res.status} ${res.statusText}${text ? `: ${text}` : ''}`);
  }

  return (await res.json()) as T;
}

export async function listAgents(): Promise<Agent[]> {
  return http<Agent[]>('/api/agents/list');
}

export async function listScenarios(): Promise<ScenarioMeta[]> {
  return http<ScenarioMeta[]>('/api/scenarios');
}

export async function getScenario(scenarioId: string): Promise<ScenarioDef> {
  return http<ScenarioDef>(`/api/scenarios/${encodeURIComponent(scenarioId)}`);
}

export async function listRuns(): Promise<Run[]> {
  return http<Run[]>('/api/runs');
}

export async function getRun(runId: string): Promise<Run> {
  return http<Run>(`/api/runs/${encodeURIComponent(runId)}`);
}

export async function listRunSteps(runId: string): Promise<Step[]> {
  return http<Step[]>(`/api/runs/${encodeURIComponent(runId)}/steps`);
}

export async function listRunEvents(runId: string): Promise<Event[]> {
  return http<Event[]>(`/api/runs/${encodeURIComponent(runId)}/events`);
}

export async function listRunEvidence(runId: string): Promise<Evidence[]> {
  return http<Evidence[]>(`/api/runs/${encodeURIComponent(runId)}/evidence`);
}

export async function getRunVerdict(runId: string): Promise<RunVerdictView> {
  return http<RunVerdictView>(`/api/runs/${encodeURIComponent(runId)}/verdict`);
}

export async function listOperatorActions(runId: string): Promise<OperatorAction[]> {
  return http<OperatorAction[]>(`/api/runs/${encodeURIComponent(runId)}/operator-actions`);
}

export async function createRun(input: {
  agent_id: string;
  scenario_id?: string | null;
  test_id?: string | null;
  params_json?: string | null;
}): Promise<Run> {
  return http<Run>('/api/runs', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}

export async function postOperatorAction(
  runId: string,
  input: {
    type: 'approve_action' | 'select_choice';
    action_id?: string | null;
    choice_id?: string | null;
    note?: string | null;
  },
): Promise<void> {
  await http<unknown>(`/api/runs/${encodeURIComponent(runId)}/operator-actions`, {
    method: 'POST',
    body: JSON.stringify(input),
  });
}

export async function listEvents(): Promise<Event[]> {
  return http<Event[]>('/api/events');
}

export async function matchFingerprint(input: {
  banner: string;
  limit?: number;
}): Promise<{ candidates: FingerprintCandidate[] }> {
  return http<{ candidates: FingerprintCandidate[] }>('/api/fingerprint/match', {
    method: 'POST',
    body: JSON.stringify(input),
  });
}

export function getServerUrl() {
  return serverUrl;
}
