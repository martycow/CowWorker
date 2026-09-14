import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Field } from '../components';
interface Research {
  id: string;
  status: string;
  sourceUrl: string;
  createdAt: number;
  completedAt: number | null;
  error: string | null;
}
interface Proposal {
  id: string;
  status: string;
  baseRevision: number;
  fields: Record<string, unknown>;
}
export function CompanyResearch({
  id,
  revision,
  website,
  onChange,
}: {
  id: string;
  revision: number;
  website: string;
  onChange: () => Promise<void>;
}) {
  const [url, setUrl] = useState(website);
  const [consent, setConsent] = useState(false);
  const [history, setHistory] = useState<Research[]>([]);
  const [proposals, setProposals] = useState<Proposal[]>([]);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [replace, setReplace] = useState(false);
  useEffect(() => {
    let active = true;
    const load = () =>
      void Promise.all([
        invoke<Research[]>('company_research_history', { companyId: id }),
        invoke<Proposal[]>('list_proposals', { entityId: id }),
      ])
        .then(([h, p]) => {
          if (active) {
            setHistory(h);
            setProposals(p);
          }
        })
        .catch((e) => {
          if (active) setError(String(e));
        });
    load();
    const timer = setInterval(load, 2000);
    return () => {
      active = false;
      clearInterval(timer);
    };
  }, [id]);
  const last = history.find((r) => r.status === 'completed');
  const act = async (action: () => Promise<unknown>) => {
    setError('');
    setBusy(true);
    try {
      await action();
      await onChange();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <section>
      <h3>Sources and refresh</h3>
      <p>
        {last
          ? `Last successful refresh: ${new Date((last.completedAt ?? last.createdAt) * 1000).toLocaleDateString()}${Date.now() / 1000 - (last.completedAt ?? last.createdAt) > 30 * 86400 ? ' · older than 30 days' : ''}`
          : 'No successful refresh yet.'}
      </p>
      <Field label="Research website">
        <input
          type="url"
          value={url}
          onChange={(e) => {
            setUrl(e.target.value);
            setConsent(false);
          }}
        />
      </Field>
      <label>
        <input type="checkbox" checked={consent} onChange={(e) => setConsent(e.target.checked)} />
        Fetch this public website and its redirects for company research.
      </label>
      <button
        disabled={!consent || !url || busy}
        onClick={() =>
          void act(async () => {
            await invoke('start_company_research', {
              companyId: id,
              expectedRevision: revision,
              url,
            });
            setConsent(false);
          })
        }
      >
        Refresh website facts
      </button>
      <p>Website metadata is proposed for review. Refresh does not replace your edits.</p>
      {history.map((r) => (
        <p key={r.id}>
          {r.status} · {r.sourceUrl} · {new Date(r.createdAt * 1000).toLocaleDateString()}
          {r.error && ` · ${r.error}`}
        </p>
      ))}
      {proposals.some((p) => p.status === 'review') && (
        <label>
          <input type="checkbox" checked={replace} onChange={(e) => setReplace(e.target.checked)} />
          Replace my existing facts with the reviewed source values
        </label>
      )}
      {proposals.map((p) => (
        <article className="task-item" key={p.id}>
          <strong>
            {p.status} · base revision {p.baseRevision}
          </strong>
          <pre className="proposal-value">{JSON.stringify(p.fields, null, 2)}</pre>
          {p.status === 'review' && (
            <div>
              <button
                disabled={busy}
                onClick={() =>
                  void act(() =>
                    invoke('review_proposal', {
                      id: p.id,
                      accept: true,
                      replaceOverrides: replace,
                    }),
                  )
                }
              >
                Apply researched facts
              </button>
              <button
                disabled={busy}
                onClick={() =>
                  void act(() =>
                    invoke('review_proposal', { id: p.id, accept: false, replaceOverrides: false }),
                  )
                }
              >
                Dismiss research
              </button>
            </div>
          )}
        </article>
      ))}
      {error && <p role="alert">{error}</p>}
    </section>
  );
}
