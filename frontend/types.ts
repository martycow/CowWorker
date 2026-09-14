export type VacancyStatus = 'saved' | 'reviewed' | 'shortlisted' | 'archived';
export type Stage = 'preparing' | 'applied' | 'interview' | 'offer' | 'rejected' | 'withdrawn';
export type DocumentKind = 'resume' | 'cover-letter' | 'note' | 'job-offer' | 'agreement' | 'tax-related' | 'other';
export type WorkMode = 'Remote' | 'Hybrid' | 'On-Site' | 'Unspecified';

export interface Vacancy {
  id: string;
  title: string;
  company: string;
  location: string;
  workMode: WorkMode;
  sourceUrl: string;
  description: string;
  notes: string;
  status: VacancyStatus;
  createdAt: string;
  updatedAt: string;
}
export type VacancyInput = Omit<Vacancy, 'id' | 'createdAt' | 'updatedAt'> & { id?: string };
export interface DocumentVersion {
  id: string;
  number: number;
  content: string;
  createdAt: string;
}
export interface CareerDocument {
  id: string;
  title: string;
  kind: DocumentKind;
  versions: DocumentVersion[];
}
export interface DocumentInput {
  id?: string;
  title: string;
  kind: DocumentKind;
  content: string;
}
export interface Application {
  id: string;
  vacancyId: string;
  stage: Stage;
  nextAction: string;
  dueDate: string;
  notes: string;
  documentVersionIds: string[];
  submittedAt: string | null;
  createdAt: string;
  updatedAt: string;
  events: { id: string; stage: Stage; createdAt: string }[];
}
export type ApplicationInput = Pick<
  Application,
  'id' | 'stage' | 'nextAction' | 'dueDate' | 'notes' | 'documentVersionIds'
>;
export interface Profile {
  name: string;
  headline: string;
  email: string;
  summary: string;
}
export interface Workspace {
  schemaVersion: number;
  vacancies: Vacancy[];
  documents: CareerDocument[];
  applications: Application[];
  profile: Profile;
}
export const stageLabels: Record<Stage, string> = {
  preparing: 'Preparing',
  applied: 'Applied',
  interview: 'Interview',
  offer: 'Offer',
  rejected: 'Rejected',
  withdrawn: 'Withdrawn',
};
export const documentLabels: Record<DocumentKind, string> = {
  resume: 'Resume',
  'cover-letter': 'Cover letter',
  note: 'Note',
  'job-offer': 'Job offer',
  agreement: 'Agreement',
  'tax-related': 'Tax related',
  other: "Other"
};
export function localDate(date = new Date()): string {
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, '0')}-${String(date.getDate()).padStart(2, '0')}`;
}
export function formatDate(value: string): string {
  if (!value) return 'No date';
  return new Intl.DateTimeFormat('en', { month: 'short', day: 'numeric', year: 'numeric' }).format(
    new Date(value.length === 10 ? `${value}T12:00:00` : value),
  );
}
export function activeApplication(app: Application): boolean {
  return !['offer', 'rejected', 'withdrawn'].includes(app.stage);
}
export function filterVacancies(
  items: Vacancy[],
  query: string,
  status: string,
  mode: string,
  sort: string,
): Vacancy[] {
  const term = query.trim().toLowerCase();
  return items
    .filter(
      (v) =>
        (status === 'all' ? v.status !== 'archived' : v.status === status) &&
        (mode === 'all' || v.workMode === mode) &&
        `${v.title} ${v.company} ${v.location} ${v.description} ${v.notes}`
          .toLowerCase()
          .includes(term),
    )
    .sort((a, b) =>
      sort === 'company'
        ? a.company.localeCompare(b.company) || a.title.localeCompare(b.title)
        : sort === 'oldest'
          ? a.createdAt.localeCompare(b.createdAt)
          : b.createdAt.localeCompare(a.createdAt),
    );
}
