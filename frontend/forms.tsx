import { useEffect, useState, type FormEvent } from 'react';
import { api } from './api';
import { FileUp, LockKeyhole } from 'lucide-react';
import { Field } from './components';
import {
  documentLabels,
  stageLabels,
  type Application,
  type ApplicationInput,
  type CareerDocument,
  type DocumentInput,
  type DocumentKind,
  type Vacancy,
  type VacancyInput,
} from './types';

type Save<T> = { onSave: (input: T) => Promise<void>; busy: boolean };
export function VacancyForm({ vacancy, onSave, busy }: { vacancy?: Vacancy } & Save<VacancyInput>) {
  const [form, setForm] = useState<VacancyInput>(
    vacancy
      ? {
          id: vacancy.id,
          expectedRevision: vacancy.revision,
          title: vacancy.title,
          company: vacancy.company,
          location: vacancy.location,
          workMode: vacancy.workMode,
          sourceUrl: vacancy.sourceUrl,
          description: vacancy.description,
          notes: vacancy.notes,
          status: vacancy.status,
        }
      : {
          title: '',
          company: '',
          location: '',
          workMode: 'Unspecified',
          sourceUrl: '',
          description: '',
          notes: '',
          status: 'saved',
        },
  );
  const change = <K extends keyof VacancyInput>(key: K, value: VacancyInput[K]) =>
    setForm({ ...form, [key]: value });
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void onSave(form);
      }}
    >
      <div className="form-grid">
        <Field label="Role">
          <input
            required
            maxLength={300}
            value={form.title}
            onChange={(e) => change('title', e.target.value)}
            placeholder="Senior Gameplay Engineer"
          />
        </Field>
        <Field label="Company">
          <input
            required
            maxLength={300}
            value={form.company}
            onChange={(e) => change('company', e.target.value)}
            placeholder="Company name"
          />
        </Field>
        <Field label="Location">
          <input
            maxLength={500}
            value={form.location}
            onChange={(e) => change('location', e.target.value)}
            placeholder="City, country or region"
          />
        </Field>
        <Field label="Work mode">
          <select
            value={form.workMode}
            onChange={(e) => change('workMode', e.target.value as VacancyInput['workMode'])}
          >
            {['Unspecified', 'Remote', 'Hybrid', 'On-site'].map((v) => (
              <option key={v}>{v}</option>
            ))}
          </select>
        </Field>
        <Field wide label="Source URL">
          <input
            type="url"
            maxLength={4000}
            value={form.sourceUrl}
            onChange={(e) => change('sourceUrl', e.target.value)}
            placeholder="https://company.com/careers/role"
          />
        </Field>
        <Field wide label="Original job description">
          <textarea
            rows={8}
            maxLength={200000}
            value={form.description}
            onChange={(e) => change('description', e.target.value)}
            placeholder="Paste the original posting here. The text stays with this vacancy."
          />
        </Field>
        <Field wide label="Your notes">
          <textarea
            rows={3}
            maxLength={100000}
            value={form.notes}
            onChange={(e) => change('notes', e.target.value)}
            placeholder="Questions, impressions, things to follow up on…"
          />
        </Field>
      </div>
      <div className="form-footer">
        <p>Saved on this device. No employer is contacted.</p>
        <button className="primary" disabled={busy}>
          {busy ? 'Saving…' : 'Save vacancy'}
        </button>
      </div>
    </form>
  );
}

export function DocumentForm({
  document,
  onSave,
  busy,
}: { document?: CareerDocument } & Save<DocumentInput>) {
  const [title, setTitle] = useState(document?.title ?? '');
  const [kind, setKind] = useState<DocumentKind>(document?.kind ?? 'resume');
  const [content, setContent] = useState(document?.versions[0]?.content ?? '');
  const [importError, setImportError] = useState('');
  const [reading, setReading] = useState(!!document);
  useEffect(() => {
    if (!document) return;
    let active = true;
    void api
      .content(document.versions[0].id)
      .then((text) => {
        if (active) setContent(text);
      })
      .catch((e) => {
        if (active) setImportError(String(e));
      })
      .finally(() => {
        if (active) setReading(false);
      });
    return () => {
      active = false;
    };
  }, [document]);
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void onSave({
          id: document?.id,
          expectedRevision: document?.revision,
          title,
          kind,
          content,
        });
      }}
    >
      <div className="form-grid">
        <Field label="Document title">
          <input
            required
            maxLength={300}
            value={title}
            onChange={(e) => setTitle(e.target.value)}
            placeholder="Frontend resume"
          />
        </Field>
        <Field label="Document type">
          <select
            disabled={!!document}
            value={kind}
            onChange={(e) => setKind(e.target.value as DocumentKind)}
          >
            {Object.entries(documentLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </Field>
        <div className="wide import-line">
          <label className="button">
            <FileUp size={16} />
            Import text file
            <input
              className="file-input"
              type="file"
              accept=".txt,.md,text/plain,text/markdown"
              disabled={reading || busy}
              onChange={async (e) => {
                const file = e.target.files?.[0];
                if (!file) return;
                setImportError('');
                setReading(true);
                try {
                  if (file.size > 1_000_000) throw new Error('Choose a file smaller than 1 MB.');
                  if (!/\.(txt|md)$/i.test(file.name))
                    throw new Error('Choose a .txt or .md file.');
                  const text = new TextDecoder('utf-8', { fatal: true }).decode(
                    await file.arrayBuffer(),
                  );
                  if (text.includes('\u0000')) throw new Error('Choose a UTF-8 text file.');
                  setContent(text);
                  if (!title) setTitle(file.name.replace(/\.[^.]+$/, ''));
                } catch (err) {
                  setImportError(String(err instanceof Error ? err.message : err));
                } finally {
                  setReading(false);
                  e.target.value = '';
                }
              }}
            />
          </label>
          <span>UTF-8 .txt or .md · up to 1 MB</span>
        </div>
        {importError && (
          <p className="error wide" role="alert">
            {importError}
          </p>
        )}
        <Field wide label="Content">
          <textarea
            className="document-editor"
            disabled={reading}
            required
            rows={15}
            maxLength={1000000}
            value={content}
            onChange={(e) => setContent(e.target.value)}
            placeholder="Write or paste your document…"
          />
        </Field>
      </div>
      <div className="form-footer">
        <p>
          {document
            ? 'A new version is saved. Earlier versions stay intact.'
            : 'Your original text is stored as a separate file.'}
        </p>
        <button className="primary" disabled={busy || reading}>
          {busy ? 'Saving…' : document ? 'Save new version' : 'Save document'}
        </button>
      </div>
    </form>
  );
}

export function ApplicationForm({
  application,
  documents,
  onSave,
  busy,
}: { application: Application; documents: CareerDocument[] } & Save<ApplicationInput>) {
  const [form, setForm] = useState<ApplicationInput>({
    id: application.id,
    stage: application.stage,
    nextAction: application.nextAction,
    dueDate: application.dueDate,
    notes: application.notes,
    documentVersionIds: application.documentVersionIds,
  });
  const submit = (e: FormEvent) => {
    e.preventDefault();
    void onSave(form);
  };
  return (
    <form onSubmit={submit}>
      <div className="form-grid">
        <Field label="Stage">
          <select
            value={form.stage}
            onChange={(e) =>
              setForm({ ...form, stage: e.target.value as ApplicationInput['stage'] })
            }
          >
            {Object.entries(stageLabels).map(([value, label]) => (
              <option
                value={value}
                key={value}
                disabled={!!application.submittedAt && value === 'preparing'}
              >
                {label}
              </option>
            ))}
          </select>
        </Field>
        <Field label="Due date">
          <input
            type="date"
            value={form.dueDate}
            onChange={(e) => setForm({ ...form, dueDate: e.target.value })}
          />
        </Field>
        <Field wide label="Next action">
          <input
            required={!!form.dueDate}
            maxLength={1000}
            value={form.nextAction}
            onChange={(e) => setForm({ ...form, nextAction: e.target.value })}
            placeholder="Follow up with the recruiter"
          />
        </Field>
        <fieldset className="wide attachment-list">
          <legend>Document versions</legend>
          {application.submittedAt && (
            <p>
              <LockKeyhole size={14} />
              Submitted copies are locked.
            </p>
          )}
          {documents.length === 0 ? (
            <p>Create a document before recording a submitted application.</p>
          ) : (
            documents.map((d) => (
              <div key={d.id} className="attachment-group">
                <strong>{d.title}</strong>
                {d.versions.map((v) => (
                  <label className="check-row" key={v.id}>
                    <input
                      type="checkbox"
                      disabled={!!application.submittedAt}
                      checked={form.documentVersionIds.includes(v.id)}
                      onChange={(e) =>
                        setForm({
                          ...form,
                          documentVersionIds: e.target.checked
                            ? [...form.documentVersionIds, v.id]
                            : form.documentVersionIds.filter((id) => id !== v.id),
                        })
                      }
                    />
                    {documentLabels[d.kind]} · v{v.number}
                  </label>
                ))}
              </div>
            ))
          )}
        </fieldset>
        <Field wide label="Application notes">
          <textarea
            rows={4}
            maxLength={100000}
            value={form.notes}
            onChange={(e) => setForm({ ...form, notes: e.target.value })}
          />
        </Field>
      </div>
      <div className="form-footer">
        <p>“Applied” records a submission you made. CowWorker does not send it.</p>
        <button disabled={busy} className="primary">
          {busy ? 'Saving…' : 'Save application'}
        </button>
      </div>
    </form>
  );
}
