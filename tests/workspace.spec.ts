import { test, expect, type Page } from '@playwright/test';

async function addVacancy(page: Page, title = 'Frontend Engineer') {
  await page.getByRole('button', { name: 'Add vacancy', exact: true }).click();
  await page.getByLabel('Role', { exact: true }).fill(title);
  await page.getByLabel('Company', { exact: true }).fill('Workflow Test');
  await page.getByLabel('Source URL').fill('https://example.com/jobs/workflow');
  await page
    .getByLabel('Original job description')
    .fill('React, TypeScript and accessible interfaces.');
  await page.getByRole('button', { name: 'Save vacancy', exact: true }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
}
async function addDocument(page: Page) {
  await page.getByRole('button', { name: 'Documents', exact: true }).click();
  await page.getByRole('button', { name: 'New document', exact: true }).click();
  await page.getByLabel('Document title').fill('Frontend resume');
  await page
    .getByLabel('Content', { exact: true })
    .fill('Resume version one.\nOriginal experience.');
  await page.getByRole('button', { name: 'Save document', exact: true }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
}
test.beforeEach(async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('heading', { name: 'Vacancies', exact: true })).toBeVisible();
});

test('closing an edited dialog protects unsaved text', async ({ page }) => {
  await page.getByRole('button', { name: 'Add vacancy', exact: true }).click();
  await page.getByLabel('Role', { exact: true }).fill('Keep this draft');
  await page.keyboard.press('Escape');
  await expect(page.getByRole('alert')).toContainText('Discard your unsaved changes?');
  await page.getByRole('button', { name: 'Keep editing' }).click();
  await expect(page.getByLabel('Role', { exact: true })).toHaveValue('Keep this draft');
  await page.getByRole('button', { name: 'Close dialog' }).click();
  await page.getByRole('button', { name: 'Discard changes' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
});

test('vacancy, document, submitted application, next action and version history survive reload', async ({
  page,
}) => {
  await addVacancy(page);
  await addDocument(page);
  await page.getByRole('button', { name: 'Vacancies', exact: true }).click();
  await page.getByRole('button', { name: 'Prepare application', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Applications', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Update application' }).click();
  await page.getByLabel('Stage', { exact: true }).selectOption('applied');
  await page.getByLabel('Resume · v1').check();
  await page.getByLabel('Next action', { exact: true }).fill('Follow up with recruiter');
  await page.getByLabel('Due date').fill('2026-01-01');
  await page.getByRole('button', { name: 'Save application' }).click();
  await expect(page.getByRole('heading', { name: 'Submitted documents' })).toBeVisible();
  await page.getByRole('button', { name: 'Documents', exact: true }).click();
  await page.getByRole('button', { name: 'Edit latest' }).click();
  await page.getByLabel('Content', { exact: true }).fill('Resume version two.');
  await page.getByRole('button', { name: 'Save new version' }).click();
  await page.getByLabel('Document version', { exact: true }).selectOption({ index: 1 });
  await expect(page.locator('.document-paper')).toContainText('Resume version one.');
  await page.reload();
  await page.getByRole('button', { name: 'Applications', exact: true }).click();
  await expect(page.locator('.detail-panel')).toContainText('Frontend resume · v1');
  await page.getByRole('button', { name: 'Update application' }).click();
  await expect(page.getByLabel('Resume · v1')).toBeChecked();
  await expect(page.getByLabel('Resume · v1')).toBeDisabled();
  await page.getByRole('button', { name: 'Close dialog' }).click();
  await page.getByRole('button', { name: /^Today/ }).click();
  await expect(page.getByRole('button', { name: /Follow up with recruiter/ })).toBeVisible();
});

test('shortlist, search, archive and restore preserve the vacancy', async ({ page }) => {
  await addVacancy(page);
  await page.getByRole('button', { name: 'Shortlist', exact: true }).last().click();
  await expect(page.getByRole('button', { name: 'Unshortlist' })).toBeVisible();
  await page.getByRole('button', { name: 'Archive vacancy' }).click();
  await page.getByRole('button', { name: 'Archived', exact: true }).click();
  await expect(page.getByRole('button', { name: 'Restore vacancy' })).toBeVisible();
  await page.getByRole('button', { name: 'Restore vacancy' }).click();
  await page.getByRole('button', { name: 'All saved', exact: true }).click();
  await page.getByRole('textbox', { name: 'Search vacancies' }).fill('missing role');
  await expect(page.getByRole('heading', { name: 'No vacancies match' })).toBeVisible();
  await page.getByRole('button', { name: 'Reset filters' }).click();
  await expect(page.getByRole('button', { name: /Frontend Engineer Workflow Test/ })).toBeVisible();
});

test('invalid and duplicate source URLs keep the form open with actionable errors', async ({
  page,
}) => {
  await addVacancy(page);
  await page.getByRole('button', { name: 'Add vacancy', exact: true }).click();
  await page.getByLabel('Role', { exact: true }).fill('Duplicate');
  await page.getByLabel('Company', { exact: true }).fill('Workflow Test');
  await page.getByLabel('Source URL').fill('https://example.com/jobs/workflow#requirements');
  await page.getByRole('button', { name: 'Save vacancy', exact: true }).click();
  await expect(page.getByRole('alert')).toContainText('already saved');
  await expect(page.getByRole('dialog')).toBeVisible();
});

test('sidebar keyboard focus, persistent theme and narrow layout', async ({ page }) => {
  await page.getByRole('button', { name: 'Hide sidebar' }).click();
  await expect(page.getByRole('button', { name: 'Show sidebar' })).toBeFocused();
  await page.reload();
  await expect(page.getByRole('button', { name: 'Show sidebar' })).toBeVisible();
  await page.getByRole('button', { name: 'Show sidebar' }).click();
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await page.getByLabel('Theme', { exact: true }).selectOption('light');
  await page.reload();
  await expect(page.locator('html')).toHaveAttribute('data-theme', 'light');
  await page.getByRole('button', { name: 'Hide sidebar' }).click();
  await page.setViewportSize({ width: 620, height: 700 });
  await addVacancy(page, 'A very long frontend engineering role for a distributed product team');
  await expect(
    page.getByRole('button', { name: 'Prepare application', exact: true }),
  ).toBeVisible();
  await expect(page.getByRole('button', { name: 'Back to list' })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.getByRole('button', { name: 'Back to list' }).click();
  await expect(page.getByRole('table')).toBeVisible();
});

test('text import and export preserve the original content', async ({ page }) => {
  await page.getByRole('button', { name: 'Documents', exact: true }).click();
  await page.getByRole('button', { name: 'New document' }).click();
  const content = 'Original UTF-8 text.\r\nПривет, CowWorker.';
  await page
    .locator('input[type=file]')
    .setInputFiles({ name: 'Resume.txt', mimeType: 'text/plain', buffer: Buffer.from(content) });
  await expect(page.getByLabel('Document title')).toHaveValue('Resume');
  await page.getByRole('button', { name: 'Save document', exact: true }).click();
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export document version' }).click();
  const download = await downloadPromise;
  const stream = await download.createReadStream();
  const chunks: Buffer[] = [];
  for await (const chunk of stream!) chunks.push(chunk);
  expect(Buffer.concat(chunks).toString()).toBe(content);
});

test('profile saves locally and dialog traps focus and restores its trigger', async ({ page }) => {
  await page.getByRole('button', { name: 'Add vacancy', exact: true }).click();
  await expect(page.getByLabel('Role', { exact: true })).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.getByRole('button', { name: 'Add vacancy', exact: true })).toBeFocused();
  await page.getByRole('button', { name: /Your profile Job seeker/ }).click();
  await page.getByLabel('Name', { exact: true }).fill('Test User');
  await page.getByLabel('Professional headline').fill('Product engineer');
  await page.getByRole('button', { name: 'Save profile' }).click();
  await expect(page.getByRole('status')).toContainText('Profile saved');
  await page.reload();
  await expect(page.getByRole('button', { name: /Test User Product engineer/ })).toBeVisible();
});
