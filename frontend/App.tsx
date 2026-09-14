import { useEffect, useRef, useState } from 'react';
import {
  ArrowRight,
  Bookmark,
  BriefcaseBusiness,
  CalendarDays,
  Check,
  ChevronRight,
  FileText,
  Home,
  Link,
  MapPin,
  PanelLeftClose,
  PanelLeftOpen,
  Plus,
  Search,
  Settings,
  ShieldCheck,
  Star,
  X,
  Pencil,
  Archive,
  RotateCcw,
  CheckCheck,
  UserRound,
  Download,
  CircleHelp,
} from 'lucide-react';
import { api, loadExamples, native } from './api';
import { TaskCenter } from './tasks/TaskCenter';
import { CompanyHub } from './companies/CompanyHub';
import { UniversalAdd } from './import/UniversalAdd';
import { AiPanel, type AiContext } from './ai/AiPanel';
import { AiUsage } from './ai/AiUsage';
import { SourceList } from './SourceList';
import { invoke } from '@tauri-apps/api/core';
import { Badge, CompanyMark, Empty, Field, Modal } from './components';
import { ApplicationForm, DocumentForm, VacancyForm } from './forms';
import {
  activeApplication,
  documentLabels,
  filterVacancies,
  formatDate,
  localDate,
  stageLabels,
  type Application,
  type CareerDocument,
  type Profile,
  type Vacancy,
  type Workspace,
} from './types';

type Page =
  | 'AI Usage'
  | 'Companies'
  | 'Today'
  | 'Vacancies'
  | 'Applications'
  | 'Documents'
  | 'Interviews'
  | 'Glossary'
  | 'Profile'
  | 'Settings';

type Editor =
  | { kind: 'vacancy'; record?: Vacancy }
  | { kind: 'document'; record?: CareerDocument }
  | { kind: 'application'; record: Application };

const navigation = [
  { name: 'Today', icon: Home },
  { name: 'Vacancies', icon: BriefcaseBusiness },
  { name: 'Applications', icon: CheckCheck },
  { name: 'Documents', icon: FileText },
  { name: 'Companies', icon: BriefcaseBusiness },
  { name: 'AI Usage', icon: Star },
] as const;

const captions: Record<Page, string> = {
  'AI Usage': 'Model operations, attempts, and reported usage.',
  Companies: 'Companies, opportunities, and your employment history.',
  Today: 'Keep your next move in sight.',
  Vacancies: 'Vacancies manager.',
  Applications: 'Job applications manager.',
  Interviews: 'Interviews manager',
  Documents: 'Your career path in a form of documents.',
  Profile: 'Brief information about you.',
  Settings: 'Setup and customize application.',
  Glossary: 'Storage of knowledge.',
};

export function App() {
  const [companyContext, setCompanyContext] = useState<AiContext | null>(null);
  const [workspace, setWorkspace] = useState<Workspace | null>(null);
  const [page, setPage] = useState<Page>('Vacancies');
  const [error, setError] = useState('');
  const [message, setMessage] = useState('');
  const [busy, setBusy] = useState(false);
  const busyRef = useRef(false);
  const [editor, setEditor] = useState<Editor | null>(null);
  const [selected, setSelected] = useState<string | null>(null);
  const [query, setQuery] = useState('');
  const [status, setStatus] = useState('all');
  const [mode, setMode] = useState('all');
  const [sort, setSort] = useState('newest');
  const [applicationStage, setApplicationStage] = useState('all');
  const [sidebar, setSidebar] = useState(() => {
    if (window.matchMedia('(max-width: 760px)').matches) return false;
    try {
      return localStorage.getItem('cowworker.sidebar') !== 'hidden';
    } catch {
      return true;
    }
  });
  const [theme, setTheme] = useState(() => {
    try {
      return localStorage.getItem('cowworker.theme') ?? 'dark';
    } catch {
      return 'dark';
    }
  });
  const searchRef = useRef<HTMLInputElement>(null);
  const toggleRef = useRef<HTMLButtonElement>(null);
  const reload = async () => setWorkspace(await api.load());
  useEffect(() => {
    void reload().catch((e) => setError(String(e)));
  }, []);
  useEffect(() => {
    document.documentElement.dataset.theme = theme;
    try {
      localStorage.setItem('cowworker.theme', theme);
    } catch {
      /* Preferences remain available for this session. */
    }
  }, [theme]);
  useEffect(() => {
    try {
      localStorage.setItem('cowworker.sidebar', sidebar ? 'shown' : 'hidden');
    } catch {
      /* Preferences remain available for this session. */
    }
  }, [sidebar]);
  useEffect(() => {
    if (!message) return;
    const timer = setTimeout(() => setMessage(''), 5000);
    return () => clearTimeout(timer);
  }, [message]);
  useEffect(() => {
    const key = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k' && !editor) {
        e.preventDefault();
        searchRef.current?.focus();
      }
      if (e.key === 'Escape' && !editor) {
        setSelected(null);
      }
    };
    document.addEventListener('keydown', key);
    return () => document.removeEventListener('keydown', key);
  }, [editor]);
  const act = async (action: () => Promise<unknown>, notice: string, close = false) => {
    if (busyRef.current) return;
    busyRef.current = true;
    setBusy(true);
    setError('');
    try {
      await action();
      await reload();
      setMessage(notice);
      if (close) setEditor(null);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      busyRef.current = false;
      setBusy(false);
    }
  };
  const navigate = (next: Page) => {
    if (window.matchMedia('(max-width: 760px)').matches) setSidebar(false);
    setPage(next);
    setSelected(null);
    setQuery('');
    setError('');
  };
  const openEditor = (next: Editor) => {
    setError('');
    setEditor(next);
  };
  const prepare = (vacancy: Vacancy) =>
    void act(async () => {
      const id = await api.prepare(vacancy.id);
      setPage('Applications');
      setQuery('');
      setApplicationStage('all');
      setSelected(id);
    }, 'Application is ready to prepare.');
  const updateStatus = (vacancy: Vacancy, next: Vacancy['status']) =>
    void act(() => api.status(vacancy.id, vacancy.revision, next), 'Vacancy updated.');
  const vacancies = workspace
    ? filterVacancies(workspace.vacancies, query, status, mode, sort)
    : [];
  const applications =
    workspace?.applications.filter((a) => {
      const v = workspace.vacancies.find((v) => v.id === a.vacancyId);
      return (
        (applicationStage === 'all' || a.stage === applicationStage) &&
        `${v?.title} ${v?.company} ${a.nextAction}`
          .toLowerCase()
          .includes(query.trim().toLowerCase())
      );
    }) ?? [];
  const chosenVacancy = vacancies.find((v) => v.id === selected) ?? vacancies[0];
  const chosenApplication = applications.find((a) => a.id === selected) ?? applications[0];
  const visibleDocuments =
    workspace?.documents.filter((d) =>
      d.title.toLowerCase().includes(query.trim().toLowerCase()),
    ) ?? [];
  const chosenDocument = visibleDocuments.find((d) => d.id === selected) ?? visibleDocuments[0];
  const today = localDate();
  const due =
    workspace?.applications
      .filter((a) => activeApplication(a) && a.nextAction && a.dueDate && a.dueDate <= today)
      .sort((a, b) => a.dueDate.localeCompare(b.dueDate)) ?? [];
  const upcoming =
    workspace?.applications
      .filter((a) => activeApplication(a) && a.nextAction && (!a.dueDate || a.dueDate > today))
      .sort((a, b) => (a.dueDate || '9999').localeCompare(b.dueDate || '9999')) ?? [];

  return (
    <div className={`shell ${sidebar ? '' : 'sidebar-hidden'}`}>
      {sidebar && (
        <aside id="sidebar" className="sidebar">
          <div className="brand">
            <CowLogo />
            <span>CowWorker</span>
          </div>
          <nav aria-label="Main navigation">
            {navigation.map(({ name, icon: Icon }) => (
              <button
                key={name}
                aria-current={page === name ? 'page' : undefined}
                onClick={() => navigate(name)}
              >
                <Icon size={21} />
                {name}
                {name === 'Today' && due.length > 0 && (
                  <span className="nav-count">{due.length}</span>
                )}
              </button>
            ))}
          </nav>
          <div className="sidebar-bottom">
            <button
              aria-current={page === 'Settings' ? 'page' : undefined}
              onClick={() => navigate('Settings')}
            >
              <Settings size={21} />
              Settings
            </button>
            <button
              className="profile-nav"
              aria-current={page === 'Profile' ? 'page' : undefined}
              onClick={() => navigate('Profile')}
            >
              <span className="avatar">
                <UserRound size={23} />
              </span>
              <span>
                <strong>{workspace?.profile.name || 'Your profile'}</strong>
                <small>{workspace?.profile.headline || 'Job seeker'}</small>
              </span>
              <ChevronRight size={16} />
            </button>
          </div>
        </aside>
      )}
      <main>
        <div className="topbar">
          <TaskCenter />
          <UniversalAdd onChange={reload} />
          <AiPanel
            onChange={reload}
            context={
              page === 'Vacancies' && chosenVacancy
                ? {
                    entityType: 'vacancy',
                    entityId: chosenVacancy.id,
                    baseRevision: chosenVacancy.revision,
                    label: chosenVacancy.title,
                  }
                : page === 'Documents' && chosenDocument
                  ? {
                      entityType: 'document',
                      entityId: chosenDocument.id,
                      baseRevision: chosenDocument.revision,
                      baseVersion: chosenDocument.versions[0]?.id,
                      label: chosenDocument.title,
                    }
                  : page === 'Companies'
                    ? companyContext
                    : null
            }
          />
          <div className="workspace-label">
            <button
              ref={toggleRef}
              className="icon-button sidebar-toggle"
              aria-label={sidebar ? 'Hide sidebar' : 'Show sidebar'}
              aria-expanded={sidebar}
              aria-controls="sidebar"
              onClick={() => {
                setSidebar(!sidebar);
                toggleRef.current?.focus();
              }}
            >
              {sidebar ? <PanelLeftClose size={20} /> : <PanelLeftOpen size={20} />}
            </button>
            <span className="workspace-chip">
              {native ? 'Local workspace' : 'Browser demo'}
              <ShieldCheck size={16} />
            </span>
            <span className="workspace-note">
              {native
                ? 'On this device. Ready offline.'
                : 'Separate demo data. Not your desktop workspace.'}
            </span>
          </div>
          <time dateTime={today}>
            {new Intl.DateTimeFormat('en', {
              weekday: 'short',
              month: 'long',
              day: 'numeric',
              year: 'numeric',
            }).format(new Date())}
          </time>
        </div>
        <header className="page-header">
          <div>
            <h1>{page}</h1>
            <p>{captions[page]}</p>
          </div>
          {page === 'Vacancies' && (
            <button
              className="primary"
              disabled={!workspace || busy}
              onClick={() => openEditor({ kind: 'vacancy' })}
            >
              <Plus size={20} />
              Add vacancy
            </button>
          )}
          {page === 'Documents' && (
            <button
              className="primary"
              disabled={!workspace || busy}
              onClick={() => openEditor({ kind: 'document' })}
            >
              <Plus size={20} />
              New document
            </button>
          )}
        </header>
        {error && !editor && (
          <div className="error-banner" role="alert">
            <span>{error}</span>
            <button onClick={() => void act(reload, 'Workspace reloaded.')}>Retry</button>
            <button className="icon-button" aria-label="Dismiss error" onClick={() => setError('')}>
              <X size={16} />
            </button>
          </div>
        )}
        {!workspace ? (
          <div className="loading" role="status">
            {error ? 'Workspace could not be opened.' : 'Opening your workspace…'}
          </div>
        ) : (
          <>
            {['Vacancies', 'Applications', 'Documents'].includes(page) && (
              <div className="toolbar">
                <label className="search">
                  <Search size={20} />
                  <input
                    ref={searchRef}
                    aria-label={`Search ${page.toLowerCase()}`}
                    value={query}
                    onChange={(e) => setQuery(e.target.value)}
                    placeholder={
                      page === 'Vacancies'
                        ? 'Search by role, company or keyword…'
                        : `Search ${page.toLowerCase()}…`
                    }
                  />
                  {query ? (
                    <button
                      className="icon-button"
                      aria-label="Clear search"
                      onClick={() => setQuery('')}
                    >
                      <X size={16} />
                    </button>
                  ) : (
                    <kbd>Ctrl K</kbd>
                  )}
                </label>
                {page === 'Vacancies' && (
                  <>
                    <div className="segments" aria-label="Vacancy status">
                      {[
                        ['all', 'All saved'],
                        ['saved', 'To review'],
                        ['shortlisted', 'Shortlist'],
                        ['archived', 'Archived'],
                      ].map(([value, label]) => (
                        <button
                          key={value}
                          aria-pressed={status === value}
                          onClick={() => {
                            setStatus(value);
                            setSelected(null);
                          }}
                        >
                          {value === 'shortlisted' && <Star size={16} />} {label}
                        </button>
                      ))}
                    </div>
                    <select
                      aria-label="Work mode filter"
                      value={mode}
                      onChange={(e) => setMode(e.target.value)}
                    >
                      <option value="all">All work modes</option>
                      {['Remote', 'Hybrid', 'On-site', 'Unspecified'].map((v) => (
                        <option key={v}>{v}</option>
                      ))}
                    </select>
                    <select
                      aria-label="Sort vacancies"
                      value={sort}
                      onChange={(e) => setSort(e.target.value)}
                    >
                      <option value="newest">Newest first</option>
                      <option value="oldest">Oldest first</option>
                      <option value="company">Company A–Z</option>
                    </select>
                  </>
                )}
                {page === 'Applications' && (
                  <select
                    aria-label="Application stage filter"
                    value={applicationStage}
                    onChange={(e) => setApplicationStage(e.target.value)}
                  >
                    <option value="all">All stages</option>
                    {Object.entries(stageLabels).map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                )}
              </div>
            )}
            {page === 'Vacancies' &&
              (vacancies.length ? (
                <div className={`split ${selected ? 'detail-open' : ''}`}>
                  <section className="list-panel">
                    <table className="vacancy-table">
                      <thead>
                        <tr>
                          <th>Role & company</th>
                          <th>
                            <MapPin size={15} />
                            Location
                          </th>
                          <th>Review</th>
                          <th>Saved</th>
                        </tr>
                      </thead>
                      <tbody>
                        {vacancies.map((v) => (
                          <tr key={v.id} className={chosenVacancy?.id === v.id ? 'selected' : ''}>
                            <td>
                              <button
                                className="row-select"
                                onClick={() => setSelected(v.id)}
                                aria-pressed={chosenVacancy?.id === v.id}
                              >
                                <CompanyMark name={v.company} />
                                <span>
                                  <strong>{v.title}</strong>
                                  <small>{v.company}</small>
                                </span>
                              </button>
                            </td>
                            <td>
                              {v.location || 'Not specified'}
                              <small>{v.workMode}</small>
                            </td>
                            <td>
                              <Badge
                                tone={
                                  v.status === 'shortlisted'
                                    ? 'green'
                                    : v.status === 'saved'
                                      ? 'amber'
                                      : 'blue'
                                }
                              >
                                {v.status === 'shortlisted' ? (
                                  <Star size={13} />
                                ) : v.status === 'reviewed' ? (
                                  <Check size={13} />
                                ) : null}
                                {v.status === 'saved'
                                  ? 'To review'
                                  : v.status === 'shortlisted'
                                    ? 'Shortlisted'
                                    : v.status === 'archived'
                                      ? 'Archived'
                                      : 'Reviewed'}
                              </Badge>
                            </td>
                            <td>{formatDate(v.createdAt)}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                    <div className="list-footer">
                      {vacancies.length} {vacancies.length === 1 ? 'vacancy' : 'vacancies'}
                      {(query || mode !== 'all') && (
                        <button
                          className="text-button"
                          onClick={() => {
                            setQuery('');
                            setMode('all');
                          }}
                        >
                          Clear filters
                        </button>
                      )}
                    </div>
                  </section>
                  {chosenVacancy && (
                    <aside className="detail-panel">
                      <DetailClose onClose={() => setSelected(null)} />
                      <div className="detail-top">
                        <CompanyMark name={chosenVacancy.company} large />
                        <button
                          className="icon-button"
                          aria-label="Edit vacancy"
                          onClick={() => openEditor({ kind: 'vacancy', record: chosenVacancy })}
                        >
                          <Pencil size={18} />
                        </button>
                      </div>
                      <h2>{chosenVacancy.title}</h2>
                      <p className="company-name">{chosenVacancy.company}</p>
                      <p className="icon-line">
                        <MapPin size={17} />
                        {chosenVacancy.location || 'Location not specified'} ·{' '}
                        {chosenVacancy.workMode}
                      </p>
                      {chosenVacancy.sourceUrl && (
                        <a
                          className="icon-line"
                          href={chosenVacancy.sourceUrl}
                          onClick={(e) => {
                            if (native) {
                              e.preventDefault();
                              void act(
                                () => api.source(chosenVacancy.id),
                                'Source opened in your browser.',
                              );
                            }
                          }}
                          target="_blank"
                          rel="noreferrer"
                        >
                          <Link size={17} />
                          Original posting ↗
                        </a>
                      )}
                      <section className="detail-section">
                        <h3>Original description</h3>
                        <p className="preserve-text">
                          {chosenVacancy.description ||
                            'No description saved yet. Edit this vacancy to add the original posting.'}
                        </p>
                      </section>
                      <section className="detail-section">
                        <h3>Your notes</h3>
                        <SourceList entityId={chosenVacancy.id} />
                        <p className="preserve-text">
                          {chosenVacancy.notes || 'Keep your questions and impressions here.'}
                        </p>
                      </section>
                      <div className="evidence-note">
                        <CircleHelp size={17} />
                        <span>
                          Match analysis has not been run. Review the original requirements before
                          applying.
                        </span>
                      </div>
                      <div className="detail-actions">
                        {chosenVacancy.status !== 'archived' && (
                          <button
                            className="primary"
                            disabled={busy}
                            onClick={() => prepare(chosenVacancy)}
                          >
                            {workspace.applications.some((a) => a.vacancyId === chosenVacancy.id)
                              ? 'Open application'
                              : 'Prepare application'}
                            <ArrowRight size={18} />
                          </button>
                        )}
                        <div className="action-row">
                          {chosenVacancy.status !== 'archived' ? (
                            <>
                              <button
                                disabled={busy}
                                onClick={() =>
                                  updateStatus(
                                    chosenVacancy,
                                    chosenVacancy.status === 'shortlisted'
                                      ? 'reviewed'
                                      : 'shortlisted',
                                  )
                                }
                              >
                                <Star size={16} />
                                {chosenVacancy.status === 'shortlisted'
                                  ? 'Unshortlist'
                                  : 'Shortlist'}
                              </button>
                              {chosenVacancy.status === 'saved' && (
                                <button
                                  disabled={busy}
                                  onClick={() => updateStatus(chosenVacancy, 'reviewed')}
                                >
                                  <Check size={16} />
                                  Reviewed
                                </button>
                              )}
                              <button
                                className="icon-button"
                                aria-label="Archive vacancy"
                                disabled={busy}
                                onClick={() => updateStatus(chosenVacancy, 'archived')}
                              >
                                <Archive size={17} />
                              </button>
                            </>
                          ) : (
                            <button
                              disabled={busy}
                              onClick={() => updateStatus(chosenVacancy, 'saved')}
                            >
                              <RotateCcw size={16} />
                              Restore vacancy
                            </button>
                          )}
                        </div>
                        <small className="icon-line">
                          <Bookmark size={15} />
                          Saved {formatDate(chosenVacancy.createdAt)}
                        </small>
                      </div>
                    </aside>
                  )}
                </div>
              ) : (
                <Empty
                  title={
                    workspace.vacancies.length
                      ? 'No vacancies match'
                      : 'Your next opportunity starts here'
                  }
                  action={
                    <div className="action-row">
                      {workspace.vacancies.length ? (
                        <button
                          onClick={() => {
                            setQuery('');
                            setStatus('all');
                            setMode('all');
                          }}
                        >
                          Reset filters
                        </button>
                      ) : (
                        <>
                          <button
                            className="primary"
                            onClick={() => openEditor({ kind: 'vacancy' })}
                          >
                            <Plus size={18} />
                            Add your first vacancy
                          </button>
                          {!native && (
                            <button
                              disabled={busy}
                              onClick={() => void act(loadExamples, 'Fictional examples loaded.')}
                            >
                              Explore with examples
                            </button>
                          )}
                        </>
                      )}
                    </div>
                  }
                >
                  {workspace.vacancies.length
                    ? 'Try another search or change the filters.'
                    : 'Save a role, paste the posting and decide what comes next.'}
                </Empty>
              ))}
            {page === 'Applications' &&
              (applications.length ? (
                <div className={`split ${selected ? 'detail-open' : ''}`}>
                  <section className="list-panel">
                    <div className="list-caption">
                      Role & company<span>Stage</span>
                    </div>
                    {applications.map((a) => {
                      const v = workspace.vacancies.find((v) => v.id === a.vacancyId)!;
                      return (
                        <button
                          className={`record-row ${chosenApplication?.id === a.id ? 'selected' : ''}`}
                          key={a.id}
                          onClick={() => setSelected(a.id)}
                          aria-pressed={chosenApplication?.id === a.id}
                        >
                          <CompanyMark name={v.company} />
                          <span className="grow">
                            <strong>{v.title}</strong>
                            <small>{v.company}</small>
                            <span className="row-next">
                              {a.nextAction || 'No next action'}
                              {a.dueDate && ` · ${formatDate(a.dueDate)}`}
                            </span>
                          </span>
                          <Badge tone={a.stage === 'offer' ? 'green' : 'blue'}>
                            {stageLabels[a.stage]}
                          </Badge>
                          <ChevronRight size={17} />
                        </button>
                      );
                    })}
                    <div className="list-footer">{applications.length} applications</div>
                  </section>
                  {chosenApplication && (
                    <ApplicationDetail
                      key={chosenApplication.id}
                      application={chosenApplication}
                      workspace={workspace}
                      onEdit={() => openEditor({ kind: 'application', record: chosenApplication })}
                      onClose={() => setSelected(null)}
                    />
                  )}
                </div>
              ) : (
                <Empty
                  title="No applications here yet"
                  action={
                    <button className="primary" onClick={() => navigate('Vacancies')}>
                      Browse saved vacancies
                      <ArrowRight size={17} />
                    </button>
                  }
                >
                  Choose “Prepare application” on a vacancy to start keeping its documents and next
                  steps together.
                </Empty>
              ))}
            {page === 'Documents' &&
              (visibleDocuments.length ? (
                <div className={`split document-split ${selected ? 'detail-open' : ''}`}>
                  <section className="list-panel">
                    {visibleDocuments.map((d) => (
                      <button
                        className={`record-row ${chosenDocument?.id === d.id ? 'selected' : ''}`}
                        key={d.id}
                        aria-pressed={chosenDocument?.id === d.id}
                        onClick={() => setSelected(d.id)}
                      >
                        <span className="document-icon">
                          <FileText size={23} />
                        </span>
                        <span className="grow">
                          <strong>{d.title}</strong>
                          <small>
                            {documentLabels[d.kind]} · {d.versions.length}{' '}
                            {d.versions.length === 1 ? 'version' : 'versions'}
                          </small>
                        </span>
                        <ChevronRight size={17} />
                      </button>
                    ))}
                  </section>
                  {chosenDocument && (
                    <DocumentDetail
                      key={`${chosenDocument.id}-${chosenDocument.versions.length}`}
                      document={chosenDocument}
                      onChange={reload}
                      onEdit={() => openEditor({ kind: 'document', record: chosenDocument })}
                      onClose={() => setSelected(null)}
                    />
                  )}
                </div>
              ) : (
                <Empty
                  title={query ? 'No matching documents' : 'Give your experience a home'}
                  action={
                    <button className="primary" onClick={() => openEditor({ kind: 'document' })}>
                      <Plus size={18} />
                      Create a document
                    </button>
                  }
                >
                  Write a resume or cover letter, or import a text file. Each save keeps a separate
                  version.
                </Empty>
              ))}
            {page === 'Today' && (
              <div className="today-content">
                <div className="stats">
                  <div>
                    <strong>
                      {workspace.vacancies.filter((v) => v.status !== 'archived').length}
                    </strong>
                    <span>Saved vacancies</span>
                  </div>
                  <div>
                    <strong>{workspace.applications.filter(activeApplication).length}</strong>
                    <span>Active applications</span>
                  </div>
                  <div>
                    <strong>
                      {workspace.applications.filter((a) => a.stage === 'interview').length}
                    </strong>
                    <span>Interviews</span>
                  </div>
                  <div>
                    <strong>{due.length}</strong>
                    <span>Actions due</span>
                  </div>
                </div>
                <section className="today-section">
                  <h2>
                    Ready for your attention <Badge>{due.length}</Badge>
                  </h2>
                  {due.length ? (
                    due.map((a) => (
                      <ActionRow
                        key={a.id}
                        app={a}
                        workspace={workspace}
                        onClick={() => {
                          setPage('Applications');
                          setApplicationStage('all');
                          setSelected(a.id);
                        }}
                      />
                    ))
                  ) : (
                    <div className="quiet-state">
                      <CheckCheck size={23} />
                      <div>
                        <strong>Nothing due today</strong>
                        <p>Set a next action on an application to bring it here.</p>
                      </div>
                    </div>
                  )}
                </section>
                <section className="today-section">
                  <h2>Up next</h2>
                  {upcoming.length ? (
                    upcoming.map((a) => (
                      <ActionRow
                        key={a.id}
                        app={a}
                        workspace={workspace}
                        onClick={() => {
                          setPage('Applications');
                          setApplicationStage('all');
                          setSelected(a.id);
                        }}
                      />
                    ))
                  ) : (
                    <p className="muted">Your upcoming actions will appear here.</p>
                  )}
                </section>
                <button className="primary" onClick={() => navigate('Vacancies')}>
                  Review vacancies
                  <ArrowRight size={17} />
                </button>
              </div>
            )}
            {page === 'Profile' && (
              <ProfileForm
                key={JSON.stringify(workspace.profile)}
                profile={workspace.profile}
                busy={busy}
                onSave={(profile) => act(() => api.profile(profile), 'Profile saved.')}
              />
            )}
            {page === 'Companies' && (
              <CompanyHub workspace={workspace} onChange={reload} onSelect={setCompanyContext} />
            )}
            {page === 'AI Usage' && <AiUsage />}
            {page === 'Settings' && (
              <div className="settings-content">
                <section>
                  <h2>Appearance</h2>
                  <Field label="Theme">
                    <select value={theme} onChange={(e) => setTheme(e.target.value)}>
                      <option value="dark">Graphite</option>
                      <option value="light">Light</option>
                    </select>
                  </Field>
                  <label className="check-row">
                    <input
                      type="checkbox"
                      checked={sidebar}
                      onChange={(e) => setSidebar(e.target.checked)}
                    />
                    Show sidebar
                  </label>
                  <p className="muted">
                    When hidden, use the navigation button at the top left to restore it.
                  </p>
                </section>
                <section>
                  <h2>Your workspace</h2>
                  <p>
                    {native
                      ? 'Vacancies, applications and your profile are stored locally in SQLite. Document versions are separate text files.'
                      : 'This browser demo uses separate browser storage. Launch the desktop app for SQLite and local document files.'}
                  </p>
                  <p className="muted">
                    Server synchronization, AI analysis and external submissions are not connected
                    in this version.
                  </p>
                  <Badge tone="green">
                    <ShieldCheck size={14} />
                    Local work needs no account
                  </Badge>
                </section>
                <section>
                  <h2>Document copies</h2>
                  <p>
                    Each saved edit creates a new version. Submitted applications keep their
                    attached versions. You can read and export any version from Documents.
                  </p>
                  <button onClick={() => navigate('Documents')}>
                    Open documents
                    <ArrowRight size={16} />
                  </button>
                </section>
              </div>
            )}
          </>
        )}
      </main>
      {message && (
        <div className="toast" role="status">
          <Check size={18} />
          {message}
        </div>
      )}
      {editor && (
        <Modal
          title={
            editor.kind === 'vacancy'
              ? editor.record
                ? 'Edit vacancy'
                : 'Add vacancy'
              : editor.kind === 'document'
                ? editor.record
                  ? 'Edit document'
                  : 'New document'
                : 'Update application'
          }
          busy={busy}
          onClose={() => {
            setEditor(null);
            setError('');
          }}
        >
          {error && (
            <div className="error-banner" role="alert">
              {error}
            </div>
          )}
          {editor.kind === 'vacancy' && (
            <VacancyForm
              vacancy={editor.record}
              busy={busy}
              onSave={(input) =>
                act(
                  async () => {
                    const id = await api.vacancy(input);
                    setSelected(id);
                    setQuery('');
                    setMode('all');
                    setStatus(input.status === 'archived' ? 'archived' : 'all');
                  },
                  'Vacancy saved.',
                  true,
                )
              }
            />
          )}{' '}
          {editor.kind === 'document' && (
            <DocumentForm
              document={editor.record}
              busy={busy}
              onSave={(input) =>
                act(
                  async () => {
                    const id = await api.document(input);
                    setSelected(id);
                    setQuery('');
                  },
                  'Document version saved.',
                  true,
                )
              }
            />
          )}{' '}
          {editor.kind === 'application' && workspace && (
            <ApplicationForm
              application={editor.record}
              documents={workspace.documents}
              busy={busy}
              onSave={(input) => act(() => api.application(input), 'Application saved.', true)}
            />
          )}
        </Modal>
      )}
    </div>
  );
}

function CowLogo() {
  return (
    <svg viewBox="0 0 40 44" width="34" height="38" aria-hidden="true">
      <path d="M11 10 5 5 3 11 10 17M29 10l6-5 2 6-7 6" fill="var(--muted)" />
      <path d="M10 8Q20 3 30 8L28 29Q33 42 20 42 7 42 12 29Z" fill="var(--text)" />
      <path d="M11 11Q21 7 17 23L12 30 9 19M27 10Q21 9 23 23l6 7 2-12" fill="var(--bg)" />
      <ellipse cx="20" cy="34" rx="9" ry="6" fill="var(--muted)" />
      <circle cx="16" cy="34" r="1.5" fill="var(--bg)" />
      <circle cx="24" cy="34" r="1.5" fill="var(--bg)" />
    </svg>
  );
}
function DetailClose({ onClose }: { onClose: () => void }) {
  return (
    <button className="detail-close" onClick={onClose}>
      <X size={16} />
      Back to list
    </button>
  );
}
function ApplicationDetail({
  application: a,
  workspace,
  onEdit,
  onClose,
}: {
  application: Application;
  workspace: Workspace;
  onEdit: () => void;
  onClose: () => void;
}) {
  const v = workspace.vacancies.find((v) => v.id === a.vacancyId)!;
  return (
    <aside className="detail-panel">
      <DetailClose onClose={onClose} />
      <CompanyMark name={v.company} large />
      <h2>{v.title}</h2>
      <p className="company-name">{v.company}</p>
      <Badge tone="blue">{stageLabels[a.stage]}</Badge>
      <section className="detail-section">
        <h3>Next action</h3>
        <p>{a.nextAction || 'Set a next action to keep moving.'}</p>
        {a.dueDate && (
          <p className="icon-line">
            <CalendarDays size={17} />
            {formatDate(a.dueDate)}
          </p>
        )}
      </section>
      <section className="detail-section">
        <h3>{a.submittedAt ? 'Submitted documents' : 'Attached documents'}</h3>
        {a.documentVersionIds.length ? (
          a.documentVersionIds.map((id) => {
            const d = workspace.documents.find((d) => d.versions.some((v) => v.id === id));
            const version = d?.versions.find((v) => v.id === id);
            return (
              <p key={id} className="icon-line">
                <FileText size={16} />
                {d?.title} · v{version?.number}
              </p>
            );
          })
        ) : (
          <p className="muted">No versions attached yet.</p>
        )}
        {a.submittedAt && <small>Locked on {formatDate(a.submittedAt)}</small>}
      </section>
      {a.notes && (
        <section className="detail-section">
          <h3>Notes</h3>
          <p className="preserve-text">{a.notes}</p>
        </section>
      )}
      <section className="detail-section">
        <h3>History</h3>
        <ol className="timeline">
          {a.events.map((event) => (
            <li key={event.id}>
              <span>{stageLabels[event.stage]}</span>
              <small>{formatDate(event.createdAt)}</small>
            </li>
          ))}
        </ol>
      </section>
      <button className="primary" onClick={onEdit}>
        <Pencil size={17} />
        Update application
      </button>
    </aside>
  );
}
function DocumentDetail({
  document: d,
  onChange,
  onEdit,
  onClose,
}: {
  document: CareerDocument;
  onChange: () => Promise<void>;
  onEdit: () => void;
  onClose: () => void;
}) {
  const [versionId, setVersionId] = useState(d.versions[0]?.id);
  const v = d.versions.find((v) => v.id === versionId) ?? d.versions[0];
  const [body, setBody] = useState<{ id: string; text: string } | null>(null);
  const [bodyError, setBodyError] = useState('');
  useEffect(() => {
    let active = true;
    setBodyError('');
    void api
      .content(v.id)
      .then((text) => {
        if (active) setBody({ id: v.id, text });
      })
      .catch((e) => {
        if (active) setBodyError(String(e));
      });
    return () => {
      active = false;
    };
  }, [v.id]);
  const download = () => {
    if (body?.id !== v.id) return;
    const blob = new Blob([body.text], { type: 'text/plain;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${d.title.replace(/[<>:"/\\|?*\u0000-\u001f]/g, '_')}-v${v.number}.txt`;
    link.click();
    setTimeout(() => URL.revokeObjectURL(url), 1000);
  };
  return (
    <aside className="detail-panel document-preview">
      <DetailClose onClose={onClose} />
      <div className="detail-top">
        <div>
          <h2>{d.title}</h2>
          <p className="company-name">{documentLabels[d.kind]}</p>
        </div>
        <button onClick={onEdit}>
          <Pencil size={16} />
          Edit latest
        </button>
      </div>
      <div className="version-toolbar">
        <select
          aria-label="Document version"
          value={v.id}
          onChange={(e) => setVersionId(e.target.value)}
        >
          {d.versions.map((v) => (
            <option key={v.id} value={v.id}>
              Version {v.number} · {formatDate(v.createdAt)}
            </option>
          ))}
        </select>
        <button
          aria-label="Export document version"
          disabled={body?.id !== v.id}
          onClick={download}
        >
          <Download size={16} />
          Export .txt
        </button>
        {native &&
          ['pdf', 'docx'].map((format) => (
            <button
              key={format}
              onClick={() =>
                void invoke<number[]>('export_document', { versionId: v.id, format })
                  .then((bytes) => {
                    const url = URL.createObjectURL(
                      new Blob([new Uint8Array(bytes)], {
                        type:
                          format === 'pdf'
                            ? 'application/pdf'
                            : 'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
                      }),
                    );
                    const a = document.createElement('a');
                    a.href = url;
                    a.download = `${d.title.replace(/[<>:"/\\|?*]/g, '_')}-v${v.number}.${format}`;
                    a.click();
                    setTimeout(() => URL.revokeObjectURL(url), 1000);
                  })
                  .catch((e) => setBodyError(String(e)))
              }
            >
              Export .{format}
            </button>
          ))}
      </div>
      {bodyError && (
        <p role="alert" className="error">
          {bodyError}
        </p>
      )}
      {native && (
        <div className="version-toolbar">
          <label>
            Document category{' '}
            <select
              value={d.kind}
              onChange={(e) =>
                void invoke('correct_document', {
                  id: d.id,
                  expectedRevision: d.revision,
                  title: d.title,
                  kind: e.target.value,
                })
                  .then(onChange)
                  .catch((e) => setBodyError(String(e)))
              }
            >
              {Object.entries(documentLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </label>
        </div>
      )}
      <SourceList entityId={d.id} />
      <pre className="document-paper">
        {body?.id === v.id ? body.text : bodyError ? '' : 'Loading document…'}
      </pre>
    </aside>
  );
}
function ActionRow({
  app,
  workspace,
  onClick,
}: {
  app: Application;
  workspace: Workspace;
  onClick: () => void;
}) {
  const v = workspace.vacancies.find((v) => v.id === app.vacancyId)!;
  return (
    <button className="record-row" onClick={onClick}>
      <CalendarDays size={22} />
      <span className="grow">
        <strong>{app.nextAction}</strong>
        <small>
          {v.company} · {v.title}
        </small>
      </span>
      <Badge tone={app.dueDate && app.dueDate < localDate() ? 'amber' : 'neutral'}>
        {app.dueDate ? formatDate(app.dueDate) : 'No date set'}
      </Badge>
      <ChevronRight size={16} />
    </button>
  );
}
function ProfileForm({
  profile,
  busy,
  onSave,
}: {
  profile: Profile;
  busy: boolean;
  onSave: (p: Profile) => Promise<void>;
}) {
  const [form, setForm] = useState(profile);
  return (
    <form
      className="profile-form"
      onSubmit={(e) => {
        e.preventDefault();
        void onSave(form);
      }}
    >
      <div className="form-grid">
        <Field label="Name">
          <input
            maxLength={300}
            value={form.name}
            onChange={(e) => setForm({ ...form, name: e.target.value })}
          />
        </Field>
        <Field label="Email">
          <input
            type="email"
            maxLength={500}
            value={form.email}
            onChange={(e) => setForm({ ...form, email: e.target.value })}
          />
        </Field>
        <Field wide label="Professional headline">
          <input
            maxLength={500}
            value={form.headline}
            onChange={(e) => setForm({ ...form, headline: e.target.value })}
            placeholder="Frontend engineer · Accessible product experiences"
          />
        </Field>
        <Field wide label="Experience, skills and goals">
          <textarea
            rows={12}
            maxLength={100000}
            value={form.summary}
            onChange={(e) => setForm({ ...form, summary: e.target.value })}
            placeholder="Keep the facts you want to draw on when preparing applications."
          />
        </Field>
      </div>
      <div className="form-footer">
        <p>Only stored on this device.</p>
        <button className="primary" disabled={busy}>
          {busy ? 'Saving…' : 'Save profile'}
        </button>
      </div>
    </form>
  );
}
