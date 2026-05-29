import { useEffect, useRef, useState } from 'react';
import type { AiAccountInfo, AiConversation, AiMessage, AiProvider } from '../lib/types';
import {
  aiChat,
  createAiAccount,
  createAiConversation,
  getAiConversation,
  listAiAccounts,
  listAiConversations,
  removeAiAccount,
  removeAiConversation,
  saveAiScenario,
} from '../lib/api';

const PROVIDER_LABELS: Record<AiProvider, string> = {
  claude: 'Claude (Anthropic)',
  openai: 'ChatGPT (OpenAI)',
  gemini: 'Gemini (Google)',
};

const PROVIDER_COLORS: Record<AiProvider, string> = {
  claude: '#ff6b35',
  openai: '#10a37f',
  gemini: '#4285f4',
};

const DEFAULT_MODELS: Record<AiProvider, string> = {
  claude: 'claude-sonnet-4-6',
  openai: 'gpt-4o',
  gemini: 'gemini-2.0-flash',
};

type AddAccountForm = {
  name: string;
  provider: AiProvider;
  auth_type: 'api_key' | 'subscription';
  api_key: string;
  model: string;
};

const EMPTY_FORM: AddAccountForm = {
  name: '',
  provider: 'claude',
  auth_type: 'api_key',
  api_key: '',
  model: '',
};

export function AIScriptPanel() {
  const [accounts, setAccounts] = useState<AiAccountInfo[]>([]);
  const [conversations, setConversations] = useState<AiConversation[]>([]);
  const [selectedAccountId, setSelectedAccountId] = useState<string | null>(null);
  const [selectedConvId, setSelectedConvId] = useState<string | null>(null);
  const [messages, setMessages] = useState<AiMessage[]>([]);
  const [draft, setDraft] = useState<Record<string, unknown> | null>(null);
  const [input, setInput] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const abortRef = useRef<AbortController | null>(null);
  const [showAddAccount, setShowAddAccount] = useState(false);
  const [form, setForm] = useState<AddAccountForm>(EMPTY_FORM);
  const [formBusy, setFormBusy] = useState(false);
  const [saveResult, setSaveResult] = useState<string | null>(null);
  const [showDraft, setShowDraft] = useState(false);
  const chatEndRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    loadAll();
  }, []);

  useEffect(() => {
    chatEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  async function loadAll() {
    try {
      const [accs, convs] = await Promise.all([listAiAccounts(), listAiConversations()]);
      setAccounts(accs);
      setConversations(convs);
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleSelectConversation(convId: string) {
    setSelectedConvId(convId);
    setSaveResult(null);
    setDraft(null);
    setShowDraft(false);
    try {
      const conv = await getAiConversation(convId);
      let msgs: AiMessage[] = [];
      try { msgs = JSON.parse(conv.messages_json || '[]'); } catch { msgs = []; }
      setMessages(msgs);
      if (conv.scenario_draft) {
        try {
          setDraft(JSON.parse(conv.scenario_draft));
        } catch {
          setDraft(null);
        }
      }
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleNewConversation() {
    if (!selectedAccountId) {
      setError('Select an account first.');
      return;
    }
    try {
      const conv = await createAiConversation({ account_id: selectedAccountId });
      setConversations((prev) => [conv, ...prev]);
      await handleSelectConversation(conv.id);
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleSend() {
    if (!selectedConvId || !input.trim() || loading) return;
    const msg = input.trim();
    setInput('');
    setLoading(true);
    setError(null);
    const controller = new AbortController();
    abortRef.current = controller;
    try {
      const res = await aiChat(selectedConvId, msg, controller.signal);
      setMessages(res.messages);
      if (res.scenario_draft) {
        setDraft(res.scenario_draft);
        setShowDraft(true);
      }
      setConversations((prev) =>
        prev.map((c) =>
          c.id === selectedConvId
            ? { ...c, updated_at: new Date().toISOString() }
            : c,
        ),
      );
    } catch (e: unknown) {
      const err = e as { name?: string };
      if (err.name === 'AbortError') { /* user cancelled */ }
      else if (err.name === 'TimeoutError') setError('AI response timed out (120s)');
      else setError(String(e));
    } finally {
      abortRef.current = null;
      setLoading(false);
    }
  }

  async function handleSaveScenario() {
    if (!selectedConvId) return;
    setSaveResult(null);
    try {
      const res = await saveAiScenario(selectedConvId);
      setSaveResult(`Saved: ${res.scenario_id} → ${res.path}\n${res.note}`);
    } catch (e: unknown) {
      setSaveResult(`Error: ${String(e)}`);
    }
  }

  async function handleRemoveAccount(id: string) {
    try {
      await removeAiAccount(id);
      setAccounts((prev) => prev.filter((a) => a.id !== id));
      if (selectedAccountId === id) setSelectedAccountId(null);
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleRemoveConversation(id: string) {
    try {
      await removeAiConversation(id);
      setConversations((prev) => prev.filter((c) => c.id !== id));
      if (selectedConvId === id) {
        setSelectedConvId(null);
        setMessages([]);
        setDraft(null);
      }
    } catch (e: unknown) {
      setError(String(e));
    }
  }

  async function handleAddAccount(e: React.FormEvent) {
    e.preventDefault();
    if (!form.name || !form.api_key) return;
    setFormBusy(true);
    try {
      const acc = await createAiAccount({
        ...form,
        model: form.model || DEFAULT_MODELS[form.provider],
      });
      setAccounts((prev) => [acc, ...prev]);
      setSelectedAccountId(acc.id);
      setForm(EMPTY_FORM);
      setShowAddAccount(false);
    } catch (e: unknown) {
      setError(String(e));
    } finally {
      setFormBusy(false);
    }
  }

  const selectedAccount = accounts.find((a) => a.id === selectedAccountId);

  return (
    <div style={{ display: 'flex', height: '100%', gap: 12, padding: 12 }}>
      {/* Left column */}
      <div
        style={{
          width: 260,
          flexShrink: 0,
          display: 'flex',
          flexDirection: 'column',
          gap: 12,
        }}
      >
        {/* Accounts */}
        <div className="panel" style={{ flex: '0 0 auto' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 8,
            }}
          >
            <span className="mono" style={{ color: 'var(--neon-cyan)', fontSize: 11 }}>
              AI ACCOUNTS
            </span>
            <button
              className="gameBtn"
              style={{ fontSize: 10, padding: '2px 8px' }}
              onClick={() => setShowAddAccount((v) => !v)}
            >
              {showAddAccount ? 'CANCEL' : '+ ADD'}
            </button>
          </div>

          {showAddAccount && (
            <form onSubmit={handleAddAccount} style={{ marginBottom: 8 }}>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                <input
                  className="gameInput"
                  placeholder="Account name"
                  value={form.name}
                  onChange={(e) => setForm((f) => ({ ...f, name: e.target.value }))}
                  required
                />
                <select
                  className="gameInput"
                  value={form.provider}
                  onChange={(e) =>
                    setForm((f) => ({
                      ...f,
                      provider: e.target.value as AiProvider,
                      model: DEFAULT_MODELS[e.target.value as AiProvider],
                    }))
                  }
                >
                  <option value="claude">Claude (Anthropic)</option>
                  <option value="openai">ChatGPT (OpenAI)</option>
                  <option value="gemini">Gemini (Google)</option>
                </select>
                <select
                  className="gameInput"
                  value={form.auth_type}
                  onChange={(e) =>
                    setForm((f) => ({
                      ...f,
                      auth_type: e.target.value as 'api_key' | 'subscription',
                    }))
                  }
                >
                  <option value="api_key">API Key</option>
                  <option value="subscription">Subscription</option>
                </select>
                <input
                  className="gameInput"
                  placeholder="API Key / Token"
                  type="password"
                  value={form.api_key}
                  onChange={(e) => setForm((f) => ({ ...f, api_key: e.target.value }))}
                  required
                />
                <input
                  className="gameInput"
                  placeholder={`Model (default: ${DEFAULT_MODELS[form.provider]})`}
                  value={form.model}
                  onChange={(e) => setForm((f) => ({ ...f, model: e.target.value }))}
                />
                <button className="gameBtn" type="submit" disabled={formBusy}>
                  {formBusy ? 'SAVING…' : 'SAVE ACCOUNT'}
                </button>
              </div>
            </form>
          )}

          {accounts.length === 0 ? (
            <div style={{ color: 'var(--text-dim)', fontSize: 11 }}>No accounts configured.</div>
          ) : (
            accounts.map((acc) => (
              <div
                key={acc.id}
                onClick={() => setSelectedAccountId(acc.id)}
                style={{
                  padding: '6px 8px',
                  marginBottom: 4,
                  cursor: 'pointer',
                  border: `1px solid ${selectedAccountId === acc.id ? PROVIDER_COLORS[acc.provider] : 'var(--border)'}`,
                  borderRadius: 4,
                  background:
                    selectedAccountId === acc.id ? 'rgba(0,240,255,0.05)' : 'transparent',
                }}
              >
                <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                  <span
                    style={{
                      fontSize: 12,
                      color:
                        selectedAccountId === acc.id
                          ? PROVIDER_COLORS[acc.provider]
                          : 'var(--text)',
                    }}
                  >
                    {acc.name}
                  </span>
                  <button
                    className="mono"
                    style={{
                      background: 'none',
                      border: 'none',
                      color: 'var(--text-dim)',
                      cursor: 'pointer',
                      fontSize: 10,
                      padding: 0,
                    }}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRemoveAccount(acc.id);
                    }}
                  >
                    ✕
                  </button>
                </div>
                <div style={{ fontSize: 10, color: 'var(--text-dim)' }}>
                  {PROVIDER_LABELS[acc.provider]} · {acc.auth_type}
                </div>
                <div style={{ fontSize: 10, color: 'var(--text-dim)' }}>{acc.model}</div>
              </div>
            ))
          )}
        </div>

        {/* Conversations */}
        <div className="panel" style={{ flex: 1, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}>
          <div
            style={{
              display: 'flex',
              justifyContent: 'space-between',
              alignItems: 'center',
              marginBottom: 8,
            }}
          >
            <span className="mono" style={{ color: 'var(--neon-cyan)', fontSize: 11 }}>
              CONVERSATIONS
            </span>
            <button
              className="gameBtn"
              style={{ fontSize: 10, padding: '2px 8px' }}
              onClick={handleNewConversation}
              disabled={!selectedAccountId}
            >
              + NEW
            </button>
          </div>

          <div style={{ overflow: 'auto', flex: 1 }}>
            {conversations.length === 0 ? (
              <div style={{ color: 'var(--text-dim)', fontSize: 11 }}>
                {selectedAccountId ? 'No conversations yet.' : 'Select an account first.'}
              </div>
            ) : (
              conversations.map((conv) => (
                <div
                  key={conv.id}
                  onClick={() => handleSelectConversation(conv.id)}
                  style={{
                    padding: '6px 8px',
                    marginBottom: 4,
                    cursor: 'pointer',
                    border: `1px solid ${selectedConvId === conv.id ? 'var(--neon-cyan)' : 'var(--border)'}`,
                    borderRadius: 4,
                    background:
                      selectedConvId === conv.id ? 'rgba(0,240,255,0.04)' : 'transparent',
                  }}
                >
                  <div style={{ display: 'flex', justifyContent: 'space-between' }}>
                    <span
                      style={{
                        fontSize: 11,
                        color: selectedConvId === conv.id ? 'var(--neon-cyan)' : 'var(--text)',
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                        maxWidth: 170,
                      }}
                    >
                      {conv.title}
                    </span>
                    <button
                      className="mono"
                      style={{
                        background: 'none',
                        border: 'none',
                        color: 'var(--text-dim)',
                        cursor: 'pointer',
                        fontSize: 10,
                        padding: 0,
                        flexShrink: 0,
                      }}
                      onClick={(e) => {
                        e.stopPropagation();
                        handleRemoveConversation(conv.id);
                      }}
                    >
                      ✕
                    </button>
                  </div>
                  {conv.scenario_draft && (
                    <div style={{ fontSize: 10, color: 'var(--neon-yellow)' }}>● draft ready</div>
                  )}
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* Right column: chat + draft */}
      <div style={{ flex: 1, display: 'flex', flexDirection: 'column', gap: 12, minWidth: 0 }}>
        {error && (
          <div
            style={{
              padding: '8px 12px',
              background: 'rgba(255,0,0,0.1)',
              border: '1px solid var(--neon-red)',
              borderRadius: 4,
              fontSize: 12,
              color: 'var(--neon-red)',
            }}
          >
            {error}
            <button
              style={{
                marginLeft: 8,
                background: 'none',
                border: 'none',
                color: 'var(--neon-red)',
                cursor: 'pointer',
              }}
              onClick={() => setError(null)}
            >
              ✕
            </button>
          </div>
        )}

        {!selectedConvId ? (
          <div
            className="panel"
            style={{
              flex: 1,
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flexDirection: 'column',
              gap: 12,
              color: 'var(--text-dim)',
            }}
          >
            <div style={{ fontSize: 32 }}>🤖</div>
            <div className="mono" style={{ fontSize: 13, color: 'var(--neon-cyan)' }}>
              AI SCRIPT GENERATOR
            </div>
            <div style={{ fontSize: 12, textAlign: 'center', maxWidth: 320 }}>
              Add an AI account, then start a conversation to generate BAS scenarios with Claude,
              ChatGPT, or Gemini.
            </div>
          </div>
        ) : (
          <>
            {/* Chat area */}
            <div
              className="panel"
              style={{ flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}
            >
              {/* Header */}
              <div
                style={{
                  display: 'flex',
                  justifyContent: 'space-between',
                  alignItems: 'center',
                  marginBottom: 8,
                  flexShrink: 0,
                }}
              >
                <div>
                  <span className="mono" style={{ color: 'var(--neon-cyan)', fontSize: 11 }}>
                    CHAT
                  </span>
                  {selectedAccount && (
                    <span
                      style={{
                        marginLeft: 8,
                        fontSize: 10,
                        color: PROVIDER_COLORS[selectedAccount.provider],
                      }}
                    >
                      via {selectedAccount.name} ({selectedAccount.model})
                    </span>
                  )}
                </div>
                {draft && (
                  <button
                    className="gameBtn"
                    style={{
                      fontSize: 10,
                      padding: '2px 10px',
                      borderColor: 'var(--neon-yellow)',
                      color: 'var(--neon-yellow)',
                    }}
                    onClick={() => setShowDraft((v) => !v)}
                  >
                    {showDraft ? 'HIDE DRAFT' : '▶ SHOW DRAFT'}
                  </button>
                )}
              </div>

              {/* Messages */}
              <div style={{ flex: 1, overflow: 'auto', marginBottom: 8 }}>
                {messages.length === 0 ? (
                  <div style={{ color: 'var(--text-dim)', fontSize: 12 }}>
                    Ask the AI to generate a scenario. For example:
                    <br />
                    <span style={{ color: 'var(--neon-cyan)', fontStyle: 'italic' }}>
                      "Create a Windows privilege escalation scenario with 3 steps"
                    </span>
                  </div>
                ) : (
                  messages.map((msg, i) => (
                    <div
                      key={i}
                      style={{
                        marginBottom: 12,
                        display: 'flex',
                        flexDirection: msg.role === 'user' ? 'row-reverse' : 'row',
                        gap: 8,
                      }}
                    >
                      <div
                        style={{
                          maxWidth: '80%',
                          padding: '8px 12px',
                          borderRadius: 6,
                          fontSize: 12,
                          lineHeight: 1.5,
                          background:
                            msg.role === 'user'
                              ? 'rgba(0,240,255,0.1)'
                              : 'rgba(255,255,255,0.04)',
                          border: `1px solid ${msg.role === 'user' ? 'var(--neon-cyan)' : 'var(--border)'}`,
                          color: msg.role === 'user' ? 'var(--neon-cyan)' : 'var(--text)',
                          whiteSpace: 'pre-wrap',
                          wordBreak: 'break-word',
                        }}
                      >
                        {msg.content}
                      </div>
                    </div>
                  ))
                )}
                {loading && (
                  <div style={{ color: 'var(--text-dim)', fontSize: 12 }}>
                    <span style={{ animation: 'pulse 1s infinite' }}>Thinking…</span>
                  </div>
                )}
                <div ref={chatEndRef} />
              </div>

              {/* Input */}
              <div style={{ display: 'flex', gap: 8, flexShrink: 0 }}>
                <textarea
                  className="gameInput"
                  style={{ flex: 1, resize: 'none', height: 60, fontSize: 12 }}
                  placeholder="Describe the scenario you want to create…"
                  value={input}
                  onChange={(e) => setInput(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter' && !e.shiftKey) {
                      e.preventDefault();
                      handleSend();
                    }
                  }}
                  disabled={loading}
                />
                <button
                  className="gameBtn"
                  style={{ width: 64, flexShrink: 0 }}
                  onClick={handleSend}
                  disabled={loading || !input.trim()}
                >
                  SEND
                </button>
                {loading && (
                  <button
                    className="gameBtn"
                    style={{ width: 64, flexShrink: 0, borderColor: 'var(--neon-red)', color: 'var(--neon-red)' }}
                    onClick={() => abortRef.current?.abort()}
                  >
                    CANCEL
                  </button>
                )}
              </div>
            </div>

            {/* Scenario draft */}
            {draft && showDraft && (
              <div
                className="panel"
                style={{ flexShrink: 0, maxHeight: 320, overflow: 'hidden', display: 'flex', flexDirection: 'column' }}
              >
                <div
                  style={{
                    display: 'flex',
                    justifyContent: 'space-between',
                    alignItems: 'center',
                    marginBottom: 8,
                    flexShrink: 0,
                  }}
                >
                  <span className="mono" style={{ color: 'var(--neon-yellow)', fontSize: 11 }}>
                    SCENARIO DRAFT — {(draft.scenario_id as string) || '?'} /{' '}
                    {(draft.title as string) || '?'}
                  </span>
                  <button
                    className="gameBtn"
                    style={{
                      fontSize: 10,
                      padding: '2px 12px',
                      borderColor: 'var(--neon-green)',
                      color: 'var(--neon-green)',
                    }}
                    onClick={handleSaveScenario}
                  >
                    SAVE TO DISK
                  </button>
                </div>

                <pre
                  style={{
                    flex: 1,
                    overflow: 'auto',
                    fontSize: 11,
                    color: 'var(--neon-cyan)',
                    background: 'rgba(0,0,0,0.3)',
                    padding: 8,
                    borderRadius: 4,
                    margin: 0,
                  }}
                >
                  {JSON.stringify(draft, null, 2)}
                </pre>

                {saveResult && (
                  <div
                    style={{
                      marginTop: 6,
                      fontSize: 11,
                      color: saveResult.startsWith('Error')
                        ? 'var(--neon-red)'
                        : 'var(--neon-green)',
                      whiteSpace: 'pre-wrap',
                    }}
                  >
                    {saveResult}
                  </div>
                )}
              </div>
            )}
          </>
        )}
      </div>
    </div>
  );
}
