import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { native } from '../api';
import { Field, Modal } from '../components';
import { documentLabels, type DocumentKind } from '../types';
interface Draft {
  entityType: string;
  kind: DocumentKind;
  title: string;
  company: string;
  location: string;
  workMode: string;
  sourceUrl: string;
  content: string;
  structured: Record<string, unknown>;
  confidence: number;
  method: string;
}
interface Item {
  id: string;
  name: string;
  revision: number;
  status: string;
  taskStatus: string;
  draft: Draft | null;
  error: string | null;
  sourceId: string;
}
const empty = (title: string): Draft => ({
  entityType: 'document',
  kind: 'unknown',
  title,
  company: '',
  location: '',
  workMode: 'Unspecified',
  sourceUrl: '',
  content: '',
  structured: {},
  confidence: 0,
  method: 'manual review',
});
export function UniversalAdd({
  onChange,
  onOpen,
}: {
  onChange: () => Promise<void>;
  onOpen: (type: string, id: string) => void;
}) {
  const [open, setOpen] = useState(false);
  const [items, setItems] = useState<Item[]>([]);
  const [text, setText] = useState('');
  const [url, setUrl] = useState(false);
  const [authorized, setAuthorized] = useState(false);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [offset, setOffset] = useState(0);
  const [editing, setEditing] = useState<Item | null>(null);
  const [savedRevision, setSavedRevision] = useState(0);
  const [selected, setSelected] = useState<string[]>([]);
  useEffect(() => {
    const show = () => setOpen(true);
    window.addEventListener('open-universal-add', show);
    return () => window.removeEventListener('open-universal-add', show);
  }, []);
  const refresh = async () => {
    if (native) setItems(await invoke('list_imports', { offset }));
  };
  useEffect(() => {
    if (!native) return;
    let active = true;
    const load = () => {
      void invoke<Item[]>('list_imports', { offset })
        .then((rows) => {
          if (active) setItems(rows);
        })
        .catch((e) => {
          if (active) setError(String(e));
        });
    };
    load();
    const timer = setInterval(load, 2000);
    const subscription = listen('imports-added', () => {
      setOpen(true);
      load();
    });
    return () => {
      active = false;
      clearInterval(timer);
      void subscription.then((stop) => stop());
    };
  }, [offset]);
  const act = async (action: () => Promise<unknown>) => {
    setError('');
    setBusy(true);
    try {
      await action();
      await refresh();
      await onChange();
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const upload = (files: File[]) =>
    act(async () => {
      const failures: string[] = [];
      for (const file of files.slice(0, 20)) {
        try {
          if (file.size > 25_000_000) throw new Error('Choose a file up to 25 MB.');
          await invoke('import_upload', {
            name: file.name,
            bytes: Array.from(new Uint8Array(await file.arrayBuffer())),
          });
        } catch (e) {
          failures.push(`${file.name}: ${String(e)}`);
        }
      }
      if (files.length > 20) failures.push('Import up to 20 files at a time.');
      if (failures.length) setError(failures.join('\n'));
    });
  useEffect(() => {
    const paste = (event: ClipboardEvent) => {
      const files = Array.from(event.clipboardData?.files ?? []).filter((f) =>
        f.type.startsWith('image/'),
      );
      if (!native || !files.length || busy) return;
      event.preventDefault();
      setOpen(true);
      void upload(files);
    };
    window.addEventListener('paste', paste);
    return () => window.removeEventListener('paste', paste);
  }, [busy]);
  return (
    <>
      <button onClick={() => setOpen(true)}>Universal Add</button>
      {open && (
        <Modal
          title="Universal Add"
          onClose={() => setOpen(false)}
          busy={busy}
          savedRevision={savedRevision}
        >
          <p>
            Import a resume or vacancy from PDF, DOCX, text, or screenshots. Paste an image with
            Ctrl+V, or drop files onto the window. Original files stay preserved.
          </p>
          {!native && (
            <p>
              Universal Import runs in the desktop workspace. Use the manual forms in this separate
              browser demo.
            </p>
          )}
          {editing ? (
            <ImportReview
              item={editing}
              busy={busy}
              onSave={(draft, save) =>
                act(async () => {
                  const target = await invoke<string | null>('review_import', {
                    id: editing.id,
                    expectedRevision: editing.revision,
                    draft,
                    save,
                  });
                  setEditing(null);
                  setSavedRevision((revision) => revision + 1);
                  if (save && target) {
                    setOpen(false);
                    onOpen(draft.entityType, target);
                  }
                })
              }
            />
          ) : (
            <>
              <div className="version-toolbar">
                <label className="button">
                  Choose files
                  <input
                    className="file-input"
                    aria-label="Import career files"
                    type="file"
                    multiple
                    accept=".pdf,.docx,.txt,.md,.png,.jpg,.jpeg,.webp"
                    disabled={!native || busy}
                    onChange={(event) => {
                      const input = event.currentTarget;
                      const files = Array.from(input.files ?? []);
                      void upload(files).finally(() => {
                        input.value = '';
                      });
                    }}
                  />
                </label>
                <label>
                  <input
                    type="checkbox"
                    checked={url}
                    onChange={(e) => {
                      setUrl(e.target.checked);
                      setAuthorized(false);
                    }}
                  />
                  Import a URL
                </label>
              </div>
              <Field label={url ? 'Public source URL' : 'Paste text'}>
                <textarea
                  rows={url ? 2 : 5}
                  maxLength={1000000}
                  value={text}
                  onChange={(e) => {
                    setText(e.target.value);
                    if (/^https?:\/\/\S+$/.test(e.target.value.trim())) {
                      setUrl(true);
                      setAuthorized(false);
                    }
                  }}
                />
              </Field>
              {url && (
                <label>
                  <input
                    type="checkbox"
                    checked={authorized}
                    onChange={(e) => setAuthorized(e.target.checked)}
                  />
                  Fetch this public URL and its redirects. No model will be called.
                </label>
              )}
              <button
                disabled={!native || busy || !text.trim() || (url && !authorized)}
                onClick={() =>
                  void act(async () => {
                    await invoke('import_text', {
                      name: url ? 'Web source' : 'Pasted document',
                      text,
                      isUrl: url,
                    });
                    setText('');
                    setAuthorized(false);
                    setSavedRevision((revision) => revision + 1);
                  })
                }
              >
                Add to import queue
              </button>
              <h3>Import review</h3>
              <p>
                For several screenshots of one vacancy, select them in reading order and combine
                them before saving.
              </p>
              {selected.length > 0 && (
                <button
                  disabled={busy || selected.length < 2}
                  onClick={() =>
                    void act(async () => {
                      const id = await invoke<string>('combine_imports', { items: selected });
                      setSelected([]);
                      setOffset(0);
                      const rows = await invoke<Item[]>('list_imports', { offset: 0 });
                      setItems(rows);
                      setEditing(rows.find((item) => item.id === id) ?? null);
                    })
                  }
                >
                  Combine {selected.length} sources into one vacancy
                </button>
              )}
              {!items.length && <p>No imported sources yet.</p>}
              {items.map((item) => (
                <article className="task-item" key={item.id}>
                  <strong>{item.name}</strong>
                  <span>
                    {item.status === 'saved'
                      ? 'Saved'
                      : item.draft
                        ? 'Ready for review'
                        : item.taskStatus}
                  </span>
                  {item.error && <p>{item.error}</p>}
                  {item.status !== 'saved' && !['queued', 'running'].includes(item.taskStatus) && (
                    <div className="version-toolbar">
                      {item.draft && (
                        <label>
                          <input
                            type="checkbox"
                            aria-label={`Select ${item.name} for combining`}
                            checked={selected.includes(item.id)}
                            onChange={(e) =>
                              setSelected(
                                e.target.checked
                                  ? [...selected, item.id]
                                  : selected.filter((id) => id !== item.id),
                              )
                            }
                          />{' '}
                          {selected.includes(item.id)
                            ? `Reading order ${selected.indexOf(item.id) + 1}`
                            : 'Combine'}
                        </label>
                      )}
                      <button onClick={() => setEditing(item)}>Review {item.name}</button>
                    </div>
                  )}
                </article>
              ))}
              <div className="version-toolbar">
                <button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - 25))}>
                  Previous imports
                </button>
                <button disabled={items.length < 25} onClick={() => setOffset(offset + 25)}>
                  Next imports
                </button>
              </div>
            </>
          )}
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
        </Modal>
      )}
    </>
  );
}
function ImportReview({
  item,
  busy,
  onSave,
}: {
  item: Item;
  busy: boolean;
  onSave: (draft: Draft, save: boolean) => Promise<void>;
}) {
  const [draft, setDraft] = useState(item.draft ?? empty(item.name));
  const change = (key: keyof Draft, value: unknown) => setDraft({ ...draft, [key]: value });
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void onSave(draft, true);
      }}
    >
      <button type="button" disabled={busy} onClick={() => void onSave(draft, false)}>
        Keep draft and return to imports
      </button>
      <p>
        {draft.method} · confidence {Math.round(draft.confidence * 100)}%. Check the content and
        correct its destination.
      </p>
      <div className="form-grid">
        <Field label="Save as">
          <select value={draft.entityType} onChange={(e) => change('entityType', e.target.value)}>
            <option value="document">Document</option>
            <option value="vacancy">Vacancy</option>
          </select>
        </Field>
        <Field label="Title">
          <input
            required
            value={draft.title}
            maxLength={300}
            onChange={(e) => change('title', e.target.value)}
          />
        </Field>
        {draft.entityType === 'document' ? (
          <Field label="Category">
            <select value={draft.kind} onChange={(e) => change('kind', e.target.value)}>
              {Object.entries(documentLabels).map(([id, label]) => (
                <option value={id} key={id}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
        ) : (
          <>
            <Field label="Company">
              <input
                required
                maxLength={300}
                value={draft.company}
                onChange={(e) => change('company', e.target.value)}
              />
            </Field>
            <Field label="Location">
              <input value={draft.location} onChange={(e) => change('location', e.target.value)} />
            </Field>
            <Field label="Work mode">
              <select value={draft.workMode} onChange={(e) => change('workMode', e.target.value)}>
                {['Unspecified', 'Remote', 'Hybrid', 'On-site'].map((m) => (
                  <option key={m}>{m}</option>
                ))}
              </select>
            </Field>
            <Field label="Salary as written">
              <input
                value={String(draft.structured.salaryRaw ?? '')}
                onChange={(e) =>
                  change('structured', { ...draft.structured, salaryRaw: e.target.value })
                }
              />
            </Field>
            <Field label="Source URL">
              <input
                type="url"
                value={draft.sourceUrl}
                onChange={(e) => change('sourceUrl', e.target.value)}
              />
            </Field>
          </>
        )}
        <Field wide label={item.draft ? 'Extracted content' : 'Manual transcript'}>
          <textarea
            required
            rows={12}
            maxLength={draft.entityType === 'vacancy' ? 200000 : 1000000}
            value={draft.content}
            onChange={(e) => change('content', e.target.value)}
          />
        </Field>
      </div>
      <div className="form-footer">
        <button type="button" disabled={busy} onClick={() => void onSave(draft, false)}>
          Keep review draft
        </button>
        <button className="primary" disabled={busy}>
          Save reviewed {draft.entityType}
        </button>
      </div>
    </form>
  );
}
