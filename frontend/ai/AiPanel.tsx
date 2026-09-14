import { useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { native } from '../api';
import { Field, Modal } from '../components';
export interface AiContext {
  entityType: string;
  entityId: string;
  baseRevision: number;
  baseVersion?: string;
  label: string;
  documentKind?: string;
}
interface Request {
  operationType: string;
  entityType: string;
  entityId: string;
  baseRevision: number;
  baseVersion: string | null;
  instruction: string;
}
interface Config {
  endpoint: string;
  provider: string;
  model: string;
  local: boolean;
  credentialRef: string | null;
}
interface Preview {
  request: Request;
  config: Config;
  scope: string;
  context: unknown;
}
interface Operation {
  id: string;
  operationType: string;
  status: string;
  output: {
    summary: string;
    fields: Record<string, unknown> | null;
    content: string | null;
  } | null;
  error: string | null;
}
interface Proposal {
  id: string;
  entityType: string;
  baseRevision: number;
  fields: Record<string, unknown>;
  status: string;
}
export function AiPanel({
  context,
  onChange,
}: {
  context: AiContext | null;
  onChange: () => Promise<void>;
}) {
  const [open, setOpen] = useState(false);
  const trigger = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (!open) return;
    const key = (e: KeyboardEvent) => {
      if (e.key === 'Escape' && !document.querySelector('dialog[open]')) {
        setOpen(false);
        trigger.current?.focus();
      }
    };
    document.addEventListener('keydown', key);
    return () => document.removeEventListener('keydown', key);
  }, [open]);
  const [bottom, setBottom] = useState(
    () => localStorage.getItem('cowworker.ai.placement') === 'bottom',
  );
  const [settings, setSettings] = useState(false);
  const [config, setConfig] = useState<Config | null>(null);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [instruction, setInstruction] = useState('');
  const [operationType, setOperationType] = useState('job-analysis');
  useEffect(() => {
    if (context?.entityType === 'document')
      setOperationType(
        context.documentKind === 'cover-letter' ? 'cover-letter' : 'resume-tailoring',
      );
    else if (context?.entityType === 'vacancy') setOperationType('job-analysis');
  }, [context?.entityId]);
  const [preview, setPreview] = useState<Preview | null>(null);
  const [consent, setConsent] = useState(false);
  const [operations, setOperations] = useState<Operation[]>([]);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [replace, setReplace] = useState(false);
  const reload = async () => {
    if (native) {
      setConfig(await invoke('provider_config'));
      if (context) {
        setOperations(await invoke('ai_operations', { entityId: context.entityId }));
        setProposals(await invoke('list_proposals', { entityId: context.entityId }));
      }
    }
  };
  useEffect(() => {
    if (!native || !open) return;
    let active = true;
    const load = () => {
      void Promise.all([
        invoke<Config | null>('provider_config'),
        context
          ? invoke<Operation[]>('ai_operations', { entityId: context.entityId })
          : Promise.resolve([]),
        context
          ? invoke<Proposal[]>('list_proposals', { entityId: context.entityId })
          : Promise.resolve([]),
      ])
        .then(([c, o, p]) => {
          if (active) {
            setConfig(c);
            setOperations(o);
            setProposals(p);
          }
        })
        .catch((e) => {
          if (active) setError(String(e));
        });
    };
    load();
    const timer = setInterval(load, 2000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [open, context?.entityId]);
  useEffect(() => {
    localStorage.setItem('cowworker.ai.placement', bottom ? 'bottom' : 'right');
  }, [bottom]);
  const act = async (action: () => Promise<unknown>) => {
    setError('');
    setBusy(true);
    try {
      await action();
      await reload();
      await onChange();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <>
      <button ref={trigger} aria-expanded={open} onClick={() => setOpen(!open)}>
        ✦ AI
      </button>
      {open && (
        <aside className={`ai-panel ${bottom ? 'ai-bottom' : ''}`} aria-label="Contextual AI">
          <div className="version-toolbar">
            <strong>Contextual AI</strong>
            <button onClick={() => setBottom(!bottom)}>
              {bottom ? 'Place right' : 'Place below'}
            </button>
            <button onClick={() => setOpen(false)}>Close AI</button>
          </div>
          <p>{context?.label ?? 'Select a vacancy or document for context.'}</p>
          <button onClick={() => setSettings(true)}>AI settings</button>
          {!native && <p>Model operations are available in the desktop workspace.</p>}
          {!config && <p>No model is configured. Your manual workflow is ready offline.</p>}
          <Field label="AI action">
            <select value={operationType} onChange={(e) => setOperationType(e.target.value)}>
              {Object.entries({
                'job-analysis': 'Analyze job',
                'resume-tailoring': 'Tailor resume',
                'bullet-improvement': 'Improve a bullet',
                'cover-letter': 'Draft cover letter',
                'document-classification': 'Explain classification',
                'company-research': 'Company insights',
                other: 'Other',
              }).map(([id, label]) => (
                <option value={id} key={id}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Instruction or selected text">
            <textarea
              maxLength={4000}
              rows={3}
              value={instruction}
              onChange={(e) => setInstruction(e.target.value)}
            />
          </Field>
          <button
            className="primary"
            disabled={!native || !config || !context || busy}
            onClick={() =>
              void act(async () => {
                if (!context) return;
                const request: Request = {
                  operationType,
                  entityType: context.entityType,
                  entityId: context.entityId,
                  baseRevision: context.baseRevision,
                  baseVersion: context.baseVersion ?? null,
                  instruction,
                };
                setPreview(await invoke('preview_ai', { request }));
                setConsent(false);
              })
            }
          >
            ✦ Preview AI Operation
          </button>
          <p className="muted">
            AI Operation identifies model use. Cost and token estimates can be unknown.
          </p>
          {operations.map((op) => (
            <article className="task-item" key={op.id}>
              <strong>
                {op.operationType} · {op.status}
              </strong>
              {op.output && (
                <>
                  <p className="preserve-lines">{op.output.summary}</p>
                  {(op.output.fields || op.output.content) && (
                    <button
                      disabled={busy}
                      onClick={() => void act(() => invoke('propose_ai_output', { id: op.id }))}
                    >
                      Review suggested changes
                    </button>
                  )}
                </>
              )}
              {op.error && <p>{op.error}</p>}
            </article>
          ))}
          {proposals.filter((p) => p.status === 'review').length > 0 && (
            <label>
              <input
                type="checkbox"
                checked={replace}
                onChange={(e) => setReplace(e.target.checked)}
              />
              Replace my existing values when applying field proposals
            </label>
          )}
          {proposals.map((p) => (
            <article className="task-item" key={p.id}>
              <strong>
                Proposal · {p.status} · revision {p.baseRevision}
              </strong>
              <pre className="proposal-value">{JSON.stringify(p.fields, null, 2)}</pre>
              {p.status === 'review' && (
                <div>
                  <button
                    disabled={busy}
                    onClick={() =>
                      void act(() =>
                        p.entityType === 'document'
                          ? invoke('accept_document_proposal', { id: p.id })
                          : invoke('review_proposal', {
                              id: p.id,
                              accept: true,
                              replaceOverrides: replace,
                            }),
                      )
                    }
                  >
                    Apply proposal
                  </button>
                  <button
                    disabled={busy}
                    onClick={() =>
                      void act(() =>
                        invoke('review_proposal', {
                          id: p.id,
                          accept: false,
                          replaceOverrides: false,
                        }),
                      )
                    }
                  >
                    Dismiss
                  </button>
                </div>
              )}
            </article>
          ))}
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
        </aside>
      )}
      {preview && (
        <Modal title="AI Operation" busy={busy} onClose={() => setPreview(null)}>
          <p>
            {preview.request.operationType} · {preview.config.provider} / {preview.config.model}
          </p>
          <p>Destination: {preview.config.endpoint}</p>
          <p>
            Estimated tokens: unknown. Estimated cost:{' '}
            {preview.config.local ? 'no provider charge for a local endpoint' : 'unknown'}.
          </p>
          <details>
            <summary>Exact data for this operation</summary>
            <pre className="proposal-value">{JSON.stringify(preview.context, null, 2)}</pre>
          </details>
          <label>
            <input
              type="checkbox"
              checked={consent}
              onChange={(e) => setConsent(e.target.checked)}
            />
            Allow this operation to send the displayed data to this destination.
          </label>
          <button
            disabled={!consent || busy}
            className="primary"
            onClick={() =>
              void act(async () => {
                await invoke('start_ai', {
                  request: preview.request,
                  scope: preview.scope,
                  key: crypto.randomUUID(),
                });
                setPreview(null);
              })
            }
          >
            ✦ Run AI Operation
          </button>
          {error && <p role="alert">{error}</p>}
        </Modal>
      )}
      {settings && (
        <Modal title="AI settings" busy={busy} onClose={() => setSettings(false)}>
          <ProviderForm
            config={config}
            busy={busy}
            onSave={(next, secret) =>
              act(async () => {
                await invoke('configure_provider', { config: next, secret });
                setSettings(false);
              })
            }
          />
          {error && <p role="alert">{error}</p>}
        </Modal>
      )}
    </>
  );
}
function ProviderForm({
  config,
  busy,
  onSave,
}: {
  config: Config | null;
  busy: boolean;
  onSave: (c: Config, secret: string) => Promise<void>;
}) {
  const [form, setForm] = useState<Config>(
    config ?? { provider: '', model: '', endpoint: '', local: false, credentialRef: null },
  );
  const [models, setModels] = useState<string[]>([]);
  const [discoveryError, setDiscoveryError] = useState('');
  const [discovering, setDiscovering] = useState(false);
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        const data = new FormData(e.currentTarget);
        const secret = String(data.get('secret') ?? '');
        e.currentTarget.querySelector<HTMLInputElement>('[name=secret]')!.value = '';
        void onSave(form, secret);
      }}
    >
      <p>
        Use a server with a compatible Chat Completions JSON endpoint. Settings alone do not send a
        request.
      </p>
      {(['provider', 'model', 'endpoint'] as const).map((key) => (
        <Field label={key === 'endpoint' ? 'Full completion endpoint' : key} key={key}>
          <input
            required
            value={form[key]}
            list={key === 'model' ? 'available-models' : undefined}
            onChange={(e) => setForm({ ...form, [key]: e.target.value })}
          />
        </Field>
      ))}
      <datalist id="available-models">
        {models.map((id) => (
          <option value={id} key={id} />
        ))}
      </datalist>
      <button
        type="button"
        disabled={!native || !config || discovering}
        onClick={() => {
          setDiscovering(true);
          setDiscoveryError('');
          void invoke<string[]>('discover_models')
            .then(setModels)
            .catch((e) => setDiscoveryError(String(e)))
            .finally(() => setDiscovering(false));
        }}
      >
        {discovering ? 'Loading model IDs…' : 'Load model IDs from saved server'}
      </button>
      <p>
        This requests the saved server’s model directory with its stored credential. No workspace
        content is sent.
      </p>
      {discoveryError && <p role="alert">{discoveryError}</p>}
      <label>
        <input
          type="checkbox"
          checked={form.local}
          onChange={(e) => setForm({ ...form, local: e.target.checked })}
        />
        Local model on this computer
      </label>
      <Field label="API key (optional)">
        <input
          type="password"
          name="secret"
          autoComplete="off"
          placeholder={config?.credentialRef ? 'Stored in OS credential manager' : ''}
        />
      </Field>
      <button disabled={busy || !native} className="primary">
        Save AI settings
      </button>
    </form>
  );
}
