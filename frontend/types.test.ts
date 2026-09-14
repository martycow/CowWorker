import { describe, expect, it } from 'vitest';
import {
  activeApplication,
  filterVacancies,
  localDate,
  type Application,
  type Vacancy,
} from './types';

const record = (
  id: string,
  company: string,
  status: Vacancy['status'],
  workMode: Vacancy['workMode'],
): Vacancy => ({
  id,
  revision: 1,
  structured: {},
  title: 'Engineer',
  company,
  status,
  workMode,
  location: 'Seattle',
  description: 'Accessible software',
  notes: '',
  sourceUrl: '',
  createdAt: `2026-09-${id}T00:00:00Z`,
  updatedAt: '',
});
describe('vacancy queries', () => {
  const data = [
    record('10', 'Pinebox', 'saved', 'Remote'),
    record('11', 'Harbor', 'shortlisted', 'Hybrid'),
    record('12', 'Archived', 'archived', 'Remote'),
  ];
  it('excludes archived records from all saved', () =>
    expect(filterVacancies(data, '', 'all', 'all', 'newest').map((v) => v.id)).toEqual([
      '11',
      '10',
    ]));
  it('combines text, status and work mode and preserves the original collection', () => {
    expect(
      filterVacancies(data, ' accessible ', 'shortlisted', 'Hybrid', 'company').map(
        (v) => v.company,
      ),
    ).toEqual(['Harbor']);
    expect(data[0].id).toBe('10');
  });
  it('finds archived records only in the archive filter', () =>
    expect(filterVacancies(data, '', 'archived', 'all', 'newest')).toHaveLength(1));
  it('sorts companies and oldest records independently', () => {
    expect(filterVacancies(data, '', 'all', 'all', 'company')[0].company).toBe('Harbor');
    expect(filterVacancies(data, '', 'all', 'all', 'oldest')[0].id).toBe('10');
  });
});
it('uses calendar dates and excludes completed applications from reminders', () => {
  expect(localDate(new Date(2026, 8, 13, 23, 59))).toBe('2026-09-13');
  for (const stage of ['offer', 'rejected', 'withdrawn'])
    expect(activeApplication({ stage } as Application)).toBe(false);
  expect(activeApplication({ stage: 'interview' } as Application)).toBe(true);
});
