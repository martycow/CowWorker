import { invoke, isTauri } from '@tauri-apps/api/core';
import type { ApplicationInput, DocumentInput, Profile, VacancyInput, Workspace } from './types';

export const native = isTauri();
const demoKey = 'cowworker.browser-demo.v1';
const empty = (): Workspace => ({
  schemaVersion: 1,
  vacancies: [],
  applications: [],
  documents: [],
  profile: { name: '', headline: '', email: '', summary: '' },
});
function readDemo(): Workspace {
  const value = localStorage.getItem(demoKey);
  if (!value) return empty();
  const data = JSON.parse(value) as Workspace;
  if (
    data.schemaVersion !== 1 ||
    !Array.isArray(data.vacancies) ||
    !Array.isArray(data.documents) ||
    !Array.isArray(data.applications)
  )
    throw new Error(
      'Browser demo data cannot be read. Open the desktop app to use your local workspace.',
    );
  return data;
}
function writeDemo<T>(action: (data: Workspace) => T): Promise<T> {
  return Promise.resolve().then(() => {
    const data = readDemo();
    const result = action(data);
    localStorage.setItem(demoKey, JSON.stringify(data));
    return result;
  });
}
const uid = () => crypto.randomUUID();
const time = () => new Date().toISOString();
export const api = {
  source: (vacancyId: string): Promise<void> => invoke('open_source', { vacancyId }),
  load: (): Promise<Workspace> =>
    native ? invoke('load_workspace') : Promise.resolve().then(readDemo),
  vacancy: (input: VacancyInput): Promise<string> =>
    native
      ? invoke('save_vacancy', { input })
      : writeDemo((data) => {
          if (!input.title.trim() || !input.company.trim())
            throw new Error('Role and company are required.');
          let sourceUrl = input.sourceUrl.trim();
          if (sourceUrl) {
            const url = new URL(sourceUrl);
            if (!['http:', 'https:'].includes(url.protocol) || url.username || url.password)
              throw new Error('Source URL must use http or https without credentials.');
            url.hash = '';
            sourceUrl = url.toString();
          }
          if (
            sourceUrl &&
            data.vacancies.some((v) => v.sourceUrl === sourceUrl && v.id !== input.id)
          )
            throw new Error('This source URL is already saved. Open the existing vacancy.');
          const old = data.vacancies.find((v) => v.id === input.id);
          if (input.id && !old) throw new Error('Vacancy no longer exists.');
          const record = {
            ...input,
            title: input.title.trim(),
            company: input.company.trim(),
            sourceUrl,
            id: old?.id ?? uid(),
            createdAt: old?.createdAt ?? time(),
            updatedAt: time(),
          };
          data.vacancies = [record, ...data.vacancies.filter((v) => v.id !== record.id)];
          return record.id;
        }),
  document: (input: DocumentInput): Promise<string> =>
    native
      ? invoke('save_document', { input })
      : writeDemo((data) => {
          if (!input.title.trim() || !input.content.trim())
            throw new Error('Document title and content are required.');
          let doc = data.documents.find((d) => d.id === input.id);
          if (input.id && !doc) throw new Error('Document no longer exists.');
          if (!doc) {
            doc = { id: uid(), title: input.title, kind: input.kind, versions: [] };
            data.documents.push(doc);
          }
          doc.title = input.title;
          doc.versions.unshift({
            id: uid(),
            number: (doc.versions[0]?.number ?? 0) + 1,
            content: input.content,
            createdAt: time(),
          });
          return doc.id;
        }),
  prepare: (vacancyId: string): Promise<string> =>
    native
      ? invoke('prepare_application', { vacancyId })
      : writeDemo((data) => {
          const existing = data.applications.find((a) => a.vacancyId === vacancyId);
          if (existing) return existing.id;
          const vacancy = data.vacancies.find((v) => v.id === vacancyId);
          if (!vacancy || vacancy.status === 'archived')
            throw new Error('Choose an active vacancy.');
          const id = uid();
          const createdAt = time();
          data.applications.unshift({
            id,
            vacancyId,
            stage: 'preparing',
            nextAction: 'Prepare resume and cover letter',
            dueDate: '',
            notes: '',
            documentVersionIds: [],
            submittedAt: null,
            createdAt,
            updatedAt: createdAt,
            events: [{ id: uid(), stage: 'preparing', createdAt }],
          });
          return id;
        }),
  application: (input: ApplicationInput): Promise<void> =>
    native
      ? invoke('save_application', { input })
      : writeDemo((data) => {
          const app = data.applications.find((a) => a.id === input.id);
          if (!app) throw new Error('Application no longer exists.');
          if (input.dueDate && !input.nextAction.trim())
            throw new Error('Enter a next action for this date.');
          const ids = [...new Set(input.documentVersionIds)].sort();
          if (ids.some((id) => !data.documents.some((d) => d.versions.some((v) => v.id === id))))
            throw new Error('An attached document version no longer exists.');
          if (
            app.submittedAt &&
            JSON.stringify([...app.documentVersionIds].sort()) !== JSON.stringify(ids)
          )
            throw new Error('Submitted document versions are locked.');
          if (app.submittedAt && input.stage === 'preparing')
            throw new Error('A submitted application cannot return to preparation.');
          const submitting =
            !app.submittedAt && ['applied', 'interview', 'offer'].includes(input.stage);
          if (submitting && !ids.length)
            throw new Error('Attach the document versions you sent before recording submission.');
          if (app.stage !== input.stage)
            app.events.unshift({ id: uid(), stage: input.stage, createdAt: time() });
          Object.assign(app, input, {
            documentVersionIds: ids,
            updatedAt: time(),
            submittedAt: submitting ? time() : app.submittedAt,
          });
        }),
  profile: (profile: Profile): Promise<void> =>
    native
      ? invoke('save_profile', { profile })
      : writeDemo((data) => {
          data.profile = profile;
        }),
};

export async function loadExamples(): Promise<void> {
  if (native) throw new Error('Examples are only available in the browser demo.');
  const samples: [string, string, string, VacancyInput['workMode'], string][] = [
    [
      'Senior Frontend Engineer',
      'Northstar Labs',
      'United States',
      'Remote',
      'Build accessible interfaces for a product that helps people make sense of complex data.\n\nRequirements\n• React and TypeScript\n• Accessibility and semantic HTML\n• Experience with data visualization',
    ],
    [
      'Product Engineer',
      'Papertrail Studio',
      'Seattle, WA',
      'Hybrid',
      'Join a small team building thoughtful tools for writers.\n\nRequirements\n• End-to-end product development\n• React and a backend language\n• Clear written communication',
    ],
    [
      'Full Stack Developer',
      'Pinebox',
      'Global',
      'Remote',
      'Help a distributed team build software for independent businesses.\n\nRequirements\n• TypeScript\n• Relational databases\n• API design',
    ],
    [
      'Software Engineer',
      'Beaconly',
      'New York, NY',
      'Hybrid',
      'Build reliable services and clear interfaces for a growing product.',
    ],
    [
      'Frontend Developer',
      'Cinder Systems',
      'San Francisco, CA',
      'On-site',
      'Create useful, accessible dashboards with a multidisciplinary team.',
    ],
    [
      'Platform Engineer',
      'Harbor',
      'United States',
      'Remote',
      'Improve the developer experience across build and deployment tools.',
    ],
  ];
  for (const [title, company, location, workMode, description] of samples)
    await api.vacancy({
      title,
      company,
      location,
      workMode,
      description,
      sourceUrl: '',
      notes: 'Fictional example for the browser demo.',
      status: company === 'Northstar Labs' ? 'shortlisted' : 'saved',
    });
}
