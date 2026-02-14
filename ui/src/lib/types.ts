export type Agent = {
  id: string;
  hostname: string;
  ip: string;
  os: string;
  arch: string;
  user: string;
  last_seen: string;
  status: string;
};

export type Run = {
  id: string;
  agent_id: string;
  test_id: string;
  params_json?: string | null;
  status: string;
  result_json?: string | null;
  created_at: string;
  updated_at: string;
};

export type Event = {
  id: string;
  run_id?: string | null;
  agent_id?: string | null;
  level: string;
  message: string;
  ts: string;
};

export type FingerprintCandidate = {
  service: string;
  product?: string | null;
  version?: string | null;
  confidence: number;
};
