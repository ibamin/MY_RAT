import type {
  Agent,
  AgentTag,
  Event,
  FingerprintCandidate,
  Group,
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

export async function listPendingAgents(): Promise<Agent[]> {
  return http<Agent[]>('/api/agents/pending');
}

export async function approveAgent(agentId: string): Promise<void> {
  await http<unknown>(`/api/agents/${encodeURIComponent(agentId)}/approve`, { method: 'POST' });
}

export async function blockAgent(agentId: string): Promise<void> {
  await http<unknown>(`/api/agents/${encodeURIComponent(agentId)}/block`, { method: 'POST' });
}

export async function listAgentRuns(agentId: string): Promise<Run[]> {
  return http<Run[]>(`/api/agents/${encodeURIComponent(agentId)}/runs`);
}

export async function listAgentGroups(agentId: string): Promise<Group[]> {
  return http<Group[]>(`/api/agents/${encodeURIComponent(agentId)}/groups`);
}

export async function listAgentTags(agentId: string): Promise<AgentTag[]> {
  return http<AgentTag[]>(`/api/agents/${encodeURIComponent(agentId)}/tags`);
}

export async function addAgentTag(agentId: string, tag: string): Promise<void> {
  await http<unknown>(`/api/agents/${encodeURIComponent(agentId)}/tags`, {
    method: 'POST',
    body: JSON.stringify({ tag }),
  });
}

export async function removeAgentTag(agentId: string, tag: string): Promise<void> {
  await http<unknown>(`/api/agents/${encodeURIComponent(agentId)}/tags/remove`, {
    method: 'POST',
    body: JSON.stringify({ tag }),
  });
}

export async function listGroups(): Promise<Group[]> {
  return http<Group[]>('/api/groups');
}

export async function createGroup(name: string): Promise<Group> {
  return http<Group>('/api/groups', { method: 'POST', body: JSON.stringify({ name }) });
}

export async function listGroupAgents(groupId: string): Promise<Agent[]> {
  return http<Agent[]>(`/api/groups/${encodeURIComponent(groupId)}/agents`);
}

export async function assignAgentToGroup(groupId: string, agentId: string): Promise<void> {
  await http<unknown>(`/api/groups/${encodeURIComponent(groupId)}/assign`, {
    method: 'POST',
    body: JSON.stringify({ agent_id: agentId }),
  });
}

export async function unassignAgentFromGroup(groupId: string, agentId: string): Promise<void> {
  await http<unknown>(`/api/groups/${encodeURIComponent(groupId)}/unassign`, {
    method: 'POST',
    body: JSON.stringify({ agent_id: agentId }),
  });
}

export async function createGroupRuns(input: {
  group_id: string;
  scenario_id?: string | null;
  test_id?: string | null;
  params_json?: string | null;
}): Promise<{ runs: Run[] }> {
  return http<{ runs: Run[] }>(`/api/groups/${encodeURIComponent(input.group_id)}/runs`, {
    method: 'POST',
    body: JSON.stringify({
      scenario_id: input.scenario_id ?? null,
      test_id: input.test_id ?? null,
      params_json: input.params_json ?? null,
    }),
  });
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
