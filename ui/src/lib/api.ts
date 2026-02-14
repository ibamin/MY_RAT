import type { Agent, Event, FingerprintCandidate, Run } from './types';

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

export async function createRun(input: {
  agent_id: string;
  test_id: string;
  params_json?: string | null;
}): Promise<Run> {
  return http<Run>('/api/runs', {
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
