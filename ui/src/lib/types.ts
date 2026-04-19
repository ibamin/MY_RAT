export type Agent = {
  id: string;
  hostname: string;
  ip: string;
  os: string;
  arch: string;
  user: string;
  last_seen: string;
  status: string;
  approval_status: string;
};

export type Group = {
  id: string;
  name: string;
  created_at: string;
};

export type AgentTag = {
  agent_id: string;
  tag: string;
  created_at: string;
};

export type ScenarioMeta = {
  scenario_id: string;
  test_id: string;
  title: string;
  difficulty: number;
  version: string;
  estimated_time_sec: number;
};

export type ScenarioActionDef = {
  action_id: string;
  title: string;
  kind: string;
};

export type ScenarioChoiceDef = {
  choice_id: string;
  title: string;
};

export type ScenarioAssertionDef = {
  assertion_id: string;
  description: string;
  required: boolean;
  type: string;
  kind?: string | null;
  contains?: string | null;
};

export type ScenarioStepDef = {
  step_id: string;
  name: string;
  requires_choice_id?: string | null;
  actions: ScenarioActionDef[];
  choices: ScenarioChoiceDef[];
  assertions: ScenarioAssertionDef[];
};

export type ScenarioDef = {
  scenario_id: string;
  test_id: string;
  title: string;
  difficulty: number;
  version: string;
  estimated_time_sec: number;
  steps: ScenarioStepDef[];
};

export type Run = {
  id: string;
  agent_id: string;
  scenario_id?: string | null;
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

export type Step = {
  id: string;
  run_id: string;
  idx: number;
  name: string;
  status: string;
  started_at?: string | null;
  ended_at?: string | null;
};

export type Evidence = {
  id: string;
  run_id: string;
  step_id: string;
  kind: string;
  locator: string;
  sha256: string;
  content_json?: string | null;
  created_at: string;
};

export type Assertion = {
  id: string;
  run_id: string;
  step_id: string;
  description: string;
  required: boolean;
  rule_type?: string | null;
  kind?: string | null;
  contains?: string | null;
  status: string;
  evidence_refs_json?: string | null;
};

export type VerdictRow = {
  id: string;
  run_id: string;
  step_id: string;
  verdict: string;
  reason_code?: string | null;
  summary?: string | null;
  updated_at: string;
};

export type StepVerdictView = {
  step: Step;
  verdict: VerdictRow;
  assertions: Assertion[];
};

export type RunVerdictView = {
  run_id: string;
  steps: StepVerdictView[];
};

export type OperatorAction = {
  id: string;
  run_id: string;
  type: string;
  action_id?: string | null;
  choice_id?: string | null;
  note?: string | null;
  ts: string;
};

export type FingerprintCandidate = {
  service: string;
  product?: string | null;
  version?: string | null;
  confidence: number;
};

export type AchievementCategory = 'combat' | 'recon' | 'stealth' | 'mastery';

export type Achievement = {
  id: string;
  name: string;
  description: string;
  category: AchievementCategory;
  icon: string;
  requirement_type: 'scenario_count' | 'verdict_streak' | 'specific_scenario';
  requirement_value: string;
  created_at: string;
};

export type AchievementStatus = {
  achievement: Achievement;
  unlocked: boolean;
  unlocked_at?: string | null;
  progress: number;
};

export type AchievementCheckResponse = {
  checked_runs: number;
  unlocked_achievement_ids: string[];
};

export type CharacterClass =
  | 'striker'    // Windows agents
  | 'phantom'    // Linux agents
  | 'sentinel'   // macOS agents
  | 'commander'  // Server/C2
  | 'analyst'    // Intel/data
  | 'operator';  // Generic field

export function osToClass(os: string): CharacterClass {
  const lower = os.toLowerCase();
  if (lower.includes('windows')) return 'striker';
  if (lower.includes('linux')) return 'phantom';
  if (lower.includes('mac') || lower.includes('darwin')) return 'sentinel';
  return 'operator';
}

// --- AI Script Generator ---

export type AiProvider = 'claude' | 'openai' | 'gemini';
export type AiAuthType = 'api_key' | 'subscription';

export type AiAccountInfo = {
  id: string;
  name: string;
  provider: AiProvider;
  auth_type: AiAuthType;
  model: string;
  is_active: boolean;
  created_at: string;
};

export type AiMessage = {
  role: 'user' | 'assistant';
  content: string;
};

export type AiConversation = {
  id: string;
  account_id: string;
  title: string;
  messages_json: string;
  scenario_draft: string | null;
  created_at: string;
  updated_at: string;
};

export type AiChatResponse = {
  reply: string;
  scenario_draft: Record<string, unknown> | null;
  messages: AiMessage[];
};
