import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Badge, Field, Modal } from '../components';
import { native } from '../api';
import { CompanyResearch } from './CompanyResearch';
import { CompanyLogo } from './CompanyLogo';
import { SourceList } from '../SourceList';
import type { AiContext } from '../ai/AiPanel';
import { stageLabels, type Workspace } from '../types';
interface Company {
  id: string;
  name: string;
  fields: Record<string, string | null>;
  notes: string;
  revision: number;
  provisional: boolean;
  vacancyCount: number;
  stages: Record<string, number>;
  relationships: {
    id: string;
    kind: string;
    role: string;
    startDate: string | null;
    endDate: string | null;
  }[];
  updatedAt: string;
}
const labels: Record<string, string> = {
  description: 'Description',
  industry: 'Industry',
  website: 'Website',
  headquarters: 'Headquarters',
  remotePolicy: 'Remote policy',
  founded: 'Founded',
  employeeCount: 'Employee count',
  stack: 'Technology',
  contacts: 'Contacts',
  socialLinks: 'Social links',
  leadership: 'Leadership',
};
export function CompanyHub({
  workspace,
  onChange,
  onSelect,
}: {
  workspace: Workspace;
  onChange: () => Promise<void>;
  onSelect: (context: AiContext | null) => void;
}) {
  const [companies, setCompanies] = useState<Company[]>([]);
  const [query, setQuery] = useState('');
  const [offset, setOffset] = useState(0);
  const [selected, setSelected] = useState<Company | null>(null);
  const [editor, setEditor] = useState<Company | null | undefined>(undefined);
  const [error, setError] = useState('');
  const [busy, setBusy] = useState(false);
  const [relationship, setRelationship] = useState(false);
  const [merge, setMerge] = useState(false);
  const [target, setTarget] = useState('');
  const [splitVacancy, setSplitVacancy] = useState('');
  const refresh = async () => {
    if (native) {
      const rows = await invoke<Company[]>('list_companies', { query, offset, limit: 25 });
      setCompanies(rows);
      setSelected((old) => rows.find((c) => c.id === old?.id) ?? null);
    }
  };
  useEffect(() => {
    let active = true;
    if (native)
      void invoke<Company[]>('list_companies', { query, offset, limit: 25 })
        .then((rows) => {
          if (active) {
            setCompanies(rows);
            setSelected((old) => rows.find((c) => c.id === old?.id) ?? null);
          }
        })
        .catch((e) => {
          if (active) setError(String(e));
        });
    return () => {
      active = false;
    };
  }, [query, offset, workspace]);
  const act = async (action: () => Promise<unknown>) => {
    setBusy(true);
    setError('');
    try {
      await action();
      await refresh();
      await onChange();
      setEditor(undefined);
      setRelationship(false);
      setMerge(false);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  const related = workspace.vacancies.filter((v) => v.companyId === selected?.id);
  useEffect(() => {
    onSelect(
      selected
        ? {
            entityType: 'company',
            entityId: selected.id,
            baseRevision: selected.revision,
            label: selected.name,
          }
        : null,
    );
  }, [selected]);
  return (
    <div className="company-hub">
      <div className="version-toolbar">
        <input
          aria-label="Search companies"
          placeholder="Search companies"
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setOffset(0);
          }}
        />
        <button disabled={!native} className="primary" onClick={() => setEditor(null)}>
          New company
        </button>
      </div>
      {!native && (
        <p>
          Company records are available in the desktop workspace. Browser examples do not represent
          SQLite company data.
        </p>
      )}
      {error && (
        <p className="error" role="alert">
          {error}
        </p>
      )}
      <div className="company-layout">
        <div className="company-list">
          {companies.map((c) => (
            <button className="record-row" key={c.id} onClick={() => setSelected(c)}>
              <CompanyLogo id={c.id} name={c.name} revision={c.revision} />
              <span className="grow">
                <strong>{c.name}</strong>
                <small>
                  {c.fields.industry || c.fields.description || 'Details not added'} ·{' '}
                  {c.vacancyCount} vacancies
                </small>
              </span>
              {c.relationships.some((r) => r.kind === 'current') && (
                <Badge tone="green">Current employer</Badge>
              )}
              {c.relationships.some((r) => r.kind === 'past') && <Badge>Past employer</Badge>}
            </button>
          ))}
          {native && !companies.length && <p>No companies match this search.</p>}
        </div>
        {selected && (
          <aside className="detail-panel">
            <div className="detail-top">
              <h2>{selected.name}</h2>
              <button
                disabled={busy}
                onClick={() =>
                  void act(() =>
                    invoke('choose_company_logo', {
                      companyId: selected.id,
                      expectedRevision: selected.revision,
                    }),
                  )
                }
              >
                Choose logo
              </button>
              <button onClick={() => setEditor(selected)}>Edit company</button>
            </div>
            {selected.provisional && (
              <p>Created from a saved company label. Confirm its identity when editing.</p>
            )}
            <dl>
              {Object.entries(labels)
                .filter(([key]) => selected.fields[key])
                .map(([key, label]) => (
                  <div key={key}>
                    <dt>{label}</dt>
                    <dd>{selected.fields[key] || 'Not added'}</dd>
                  </div>
                ))}
            </dl>
            {!Object.values(selected.fields).some(Boolean) && (
              <p>
                No company facts added yet. Use Edit company to add what you know, or review a
                website source below.
              </p>
            )}
            <p>{selected.notes}</p>
            <h3>Applications</h3>
            {Object.entries(selected.stages).map(([stage, count]) => (
              <p key={stage}>
                {stageLabels[stage as keyof typeof stageLabels] ?? stage}: {count}
              </p>
            ))}
            <h3>Vacancies</h3>
            {related.map((v) => (
              <p key={v.id}>
                {v.title} · {v.status} · saved as {v.company}
              </p>
            ))}
            <h3>Employment history</h3>
            {selected.relationships.map((r) => (
              <p key={r.id}>
                <Badge tone={r.kind === 'current' ? 'green' : 'neutral'}>{r.kind}</Badge> {r.role} ·{' '}
                {r.startDate || 'Start unknown'} –{' '}
                {r.endDate || (r.kind === 'current' ? 'present' : 'End unknown')}
              </p>
            ))}
            <button onClick={() => setRelationship(true)}>Add employment</button>
            <button
              onClick={() => {
                setTarget('');
                setSplitVacancy('');
                setMerge(true);
              }}
            >
              Correct company links
            </button>
            <SourceList entityId={selected.id} />
            <CompanyResearch
              key={selected.id}
              id={selected.id}
              revision={selected.revision}
              website={selected.fields.website ?? ''}
              onChange={async () => {
                await refresh();
                await onChange();
              }}
            />
          </aside>
        )}
      </div>
      <div className="version-toolbar">
        <button disabled={offset === 0} onClick={() => setOffset(offset - 25)}>
          Previous companies
        </button>
        <span>Page {offset / 25 + 1}</span>
        <button disabled={companies.length < 25} onClick={() => setOffset(offset + 25)}>
          Next companies
        </button>
      </div>
      {editor !== undefined && (
        <Modal
          title={editor ? 'Edit company' : 'New company'}
          busy={busy}
          onClose={() => setEditor(undefined)}
        >
          <CompanyForm
            company={editor}
            busy={busy}
            onSave={(input) => act(() => invoke('save_company', { input }))}
          />
          {error && <p role="alert">{error}</p>}
        </Modal>
      )}
      {relationship && selected && (
        <Modal title="Employment relationship" busy={busy} onClose={() => setRelationship(false)}>
          <form
            onSubmit={(e) => {
              e.preventDefault();
              const f = new FormData(e.currentTarget);
              void act(() =>
                invoke('set_employment', {
                  companyId: selected.id,
                  expectedRevision: selected.revision,
                  kind: f.get('kind'),
                  role: f.get('role'),
                  start: f.get('start') || null,
                  end: f.get('end') || null,
                }),
              );
            }}
          >
            <Field label="Relationship">
              <select name="kind">
                <option value="past">Past employer</option>
                <option value="current">Current employer</option>
              </select>
            </Field>
            <Field label="Role">
              <input name="role" maxLength={300} />
            </Field>
            <Field label="Start date">
              <input type="date" name="start" />
            </Field>
            <Field label="End date">
              <input type="date" name="end" />
            </Field>
            <button className="primary" disabled={busy}>
              Save employment
            </button>
          </form>
          {error && <p role="alert">{error}</p>}
        </Modal>
      )}
      {merge && selected && (
        <Modal title="Correct company links" busy={busy} onClose={() => setMerge(false)}>
          <p>
            Move one vacancy to split a provisional group, or merge the complete company.
            Conflicting user facts must be resolved first. Create the destination company before
            moving links.
          </p>
          <Field label="Destination company">
            <select value={target} onChange={(e) => setTarget(e.target.value)}>
              <option value="">Choose company</option>
              {companies
                .filter((c) => c.id !== selected.id)
                .map((c) => (
                  <option value={c.id} key={c.id}>
                    {c.name}
                  </option>
                ))}
            </select>
          </Field>
          <Field label="Scope">
            <select value={splitVacancy} onChange={(e) => setSplitVacancy(e.target.value)}>
              <option value="">Merge complete company</option>
              {related.map((v) => (
                <option value={v.id} key={v.id}>
                  Move {v.title}
                </option>
              ))}
            </select>
          </Field>
          <button
            disabled={busy || !target}
            onClick={() =>
              void act(() =>
                splitVacancy
                  ? invoke('link_company', {
                      vacancyId: splitVacancy,
                      expectedRevision: related.find((v) => v.id === splitVacancy)!.revision,
                      companyId: target,
                    })
                  : invoke('merge_companies', {
                      source: selected.id,
                      target,
                      sourceRevision: selected.revision,
                      targetRevision: companies.find((c) => c.id === target)!.revision,
                    }),
              )
            }
          >
            Apply company correction
          </button>
          {error && <p role="alert">{error}</p>}
        </Modal>
      )}
    </div>
  );
}
function CompanyForm({
  company,
  busy,
  onSave,
}: {
  company: Company | null;
  busy: boolean;
  onSave: (input: unknown) => Promise<void>;
}) {
  const [name, setName] = useState(company?.name ?? '');
  const [fields, setFields] = useState(company?.fields ?? {});
  const [notes, setNotes] = useState(company?.notes ?? '');
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void onSave({ id: company?.id, expectedRevision: company?.revision, name, fields, notes });
      }}
    >
      <div className="form-grid">
        <Field label="Company name">
          <input required maxLength={300} value={name} onChange={(e) => setName(e.target.value)} />
        </Field>
        {Object.entries(labels).map(([key, label]) => (
          <Field label={label} key={key}>
            <input
              value={fields[key] ?? ''}
              maxLength={10000}
              onChange={(e) => setFields({ ...fields, [key]: e.target.value })}
            />
          </Field>
        ))}
        <Field wide label="Company notes">
          <textarea value={notes} maxLength={100000} onChange={(e) => setNotes(e.target.value)} />
        </Field>
      </div>
      <button className="primary" disabled={busy}>
        Save company
      </button>
    </form>
  );
}
