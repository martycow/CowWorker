import { expect } from '@playwright/test';
import { readFile } from 'node:fs/promises';
import { resolve, join } from 'node:path';

export const morningLetter = `Dear hiring team at Morning Example,

I am applying for the Senior Software Engineer position. My experience at Example Studio includes building Rust and TypeScript desktop applications with a small product team.

I worked with designers and users to simplify document workflows. I also improved accessibility and added automated recovery checks. These experiences are relevant to your team's focus on reliable desktop applications and clear communication.

I would welcome the opportunity to discuss how I could contribute to your team. Thank you for your time and consideration.

Sincerely,
Jane Doe
jane@example.test`;

export async function morningFlow(page, ipc, output) {
  console.log(
    'Checking resume import, screenshot OCR, preparation, editing, PDF export, and settings.',
  );
  const version = JSON.parse(await readFile(resolve('package.json'), 'utf8')).version;
  await page.getByRole('button', { name: 'Settings', exact: true }).click();
  await expect(page.locator('#settings-menu')).toContainText(`CowWorker v${version}`);
  await page.screenshot({ path: join(output, 'windows-settings-version.png'), fullPage: true });
  await page.keyboard.press('Escape');
  await expect(page.locator('#settings-menu')).not.toBeVisible();

  const openImport = () => page.getByRole('button', { name: 'Universal Add', exact: true }).click();
  const file = (name) => resolve('backend/tests/fixtures', name);
  await openImport();
  await page
    .getByLabel('Import career files')
    .setInputFiles([file('old-resume.docx'), file('old-resume.pdf')]);
  await expect(
    page.getByRole('button', { name: 'Review old-resume.pdf', exact: true }),
  ).toBeVisible({ timeout: 60000 });
  await page.getByRole('button', { name: 'Review old-resume.pdf', exact: true }).click();
  await expect(page.getByLabel('Extracted content')).toContainText('Example Studio');
  await expect(page.getByLabel('Category', { exact: true })).toHaveValue('resume');
  await page.getByLabel('Title', { exact: true }).fill('Original morning resume PDF');
  await page.getByRole('button', { name: 'Save reviewed document' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.document-paper')).toContainText('Example University');
  const originals = (await ipc(page, 'load_workspace')).documents;
  const original = originals.find((d) => d.title === 'Original morning resume PDF');
  const originalText = await ipc(page, 'document_content', { versionId: original.versions[0].id });
  await openImport();
  await page.getByRole('button', { name: 'Review old-resume.docx', exact: true }).click();
  await expect(page.getByLabel('Extracted content')).toContainText('Rust and TypeScript');
  await page.getByRole('button', { name: 'Keep review draft', exact: true }).click();

  await page.getByLabel('Import career files').setInputFiles(file('vacancy-screen-1.png'));
  await expect(
    page.getByRole('button', { name: 'Review vacancy-screen-1.png', exact: true }),
  ).toBeVisible({ timeout: 60000 });
  // Synthetic clipboard event exercises the WebView paste handler with real PNG bytes.
  // It does not establish interaction with the operating system clipboard.
  await page.evaluate(
    (bytes) => {
      const data = new DataTransfer();
      data.items.add(
        new File([new Uint8Array(bytes)], 'vacancy-screen-2.png', { type: 'image/png' }),
      );
      window.dispatchEvent(new ClipboardEvent('paste', { clipboardData: data }));
    },
    [...(await readFile(file('vacancy-screen-2.png')))],
  );
  await expect(page.getByLabel('Select vacancy-screen-1.png for combining')).toBeVisible({
    timeout: 60000,
  });
  await expect(page.getByLabel('Select vacancy-screen-2.png for combining')).toBeVisible({
    timeout: 60000,
  });
  await page.getByLabel('Select vacancy-screen-1.png for combining').check();
  await page.getByLabel('Select vacancy-screen-2.png for combining').check();
  await page.getByRole('button', { name: 'Combine 2 sources into one vacancy' }).click();
  await expect(page.getByLabel('Title', { exact: true })).toHaveValue('Senior Software Engineer');
  await expect(page.getByLabel('Company', { exact: true })).toHaveValue('Morning Example');
  await expect(page.getByLabel('Extracted content')).toContainText('Work with designers and users');
  await page.screenshot({ path: join(output, 'windows-screenshot-review.png'), fullPage: true });
  await page.getByRole('button', { name: 'Save reviewed vacancy' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.detail-panel')).toContainText('Morning Example');
  const vacancy = (await ipc(page, 'load_workspace')).vacancies.find(
    (v) => v.company === 'Morning Example',
  );
  const sources = await ipc(page, 'list_sources', { entityId: vacancy.id });
  expect(sources).toHaveLength(2);
  for (const source of sources) {
    expect(Buffer.from(await ipc(page, 'source_bytes', { id: source.id }))).toEqual(
      await readFile(file(source.name)),
    );
  }

  await page.getByRole('button', { name: 'Prepare documents', exact: true }).click();
  await page.getByLabel('Original resume version').selectOption(original.versions[0].id);
  await page.getByRole('button', { name: 'Create tailored resume' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await page.getByRole('button', { name: 'Edit latest' }).click();
  const resumeText = `Jane Doe\nSenior Software Engineer\njane@example.test | Seattle, WA\n\nSUMMARY\nRust and TypeScript engineer with experience building reliable desktop applications.\n\nEXPERIENCE\nExample Studio | Software Engineer | 2022-2026\nBuilt Rust and TypeScript applications with a small product team.\nImproved accessibility and added automated recovery checks.\nCollaborated with designers and users to simplify document workflows.\n\nEDUCATION\nExample University | Bachelor of Computer Science | 2022\n\nSKILLS\nRust, TypeScript, React, SQLite, automated testing`;
  await page.getByLabel('Content', { exact: true }).fill(resumeText);
  await page.getByRole('button', { name: 'Save new version' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.document-paper')).toContainText('Rust and TypeScript engineer');
  const resumeDownload = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export .pdf', exact: true }).click();
  await (await resumeDownload).saveAs(join(output, 'morning-resume.pdf'));
  expect((await readFile(join(output, 'morning-resume.pdf'))).subarray(0, 5).toString()).toBe(
    '%PDF-',
  );
  await page.screenshot({ path: join(output, 'windows-tailored-resume.png'), fullPage: true });

  await page.getByRole('button', { name: 'Vacancies', exact: true }).click();
  await page.getByRole('button', { name: /Senior Software Engineer.*Morning Example/ }).click();
  await page.getByRole('button', { name: 'Prepare documents', exact: true }).click();
  await page.getByLabel('Original resume version').selectOption(original.versions[0].id);
  await page.getByRole('button', { name: 'Create cover letter' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.document-paper')).toContainText(
    'Dear hiring team at Morning Example',
  );
  await page.getByRole('button', { name: '✦ AI', exact: true }).click();
  await expect(page.getByLabel('AI action')).toHaveValue('cover-letter');
  await page.getByRole('button', { name: '✦ Preview AI Operation', exact: true }).click();
  await page
    .getByLabel('Allow this operation to send the displayed data to this destination.')
    .check();
  await page.getByRole('button', { name: '✦ Run AI Operation', exact: true }).click();
  await expect(
    page.getByText('Fixture cover letter based on the supplied resume and vacancy.', {
      exact: true,
    }),
  ).toBeVisible({ timeout: 45000 });
  await page.getByRole('button', { name: 'Review suggested changes', exact: true }).click();
  await page.getByRole('button', { name: 'Apply proposal', exact: true }).click();
  await page.getByRole('button', { name: 'Close AI', exact: true }).click();
  await expect(page.locator('.document-paper')).toContainText('My experience at Example Studio');
  await page.getByRole('button', { name: 'Edit latest' }).click();
  await page
    .getByLabel('Content', { exact: true })
    .fill(`${morningLetter}\n\nAvailable for a conversation next week.`);
  await page.getByRole('button', { name: 'Save new version' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await expect(page.locator('.document-paper')).toContainText(
    'Available for a conversation next week.',
  );
  const letterDownload = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export .pdf', exact: true }).click();
  await (await letterDownload).saveAs(join(output, 'morning-cover-letter.pdf'));
  await page.screenshot({ path: join(output, 'windows-cover-letter.png'), fullPage: true });
  expect(await ipc(page, 'document_content', { versionId: original.versions[0].id })).toBe(
    originalText,
  );

  await openImport();
  await page.getByLabel('Import career files').setInputFiles(file('scanned-vacancy.pdf'));
  await expect(
    page.getByRole('button', { name: 'Review scanned-vacancy.pdf', exact: true }),
  ).toBeVisible({ timeout: 60000 });
  await page.getByRole('button', { name: 'Review scanned-vacancy.pdf', exact: true }).click();
  await expect(page.getByLabel('Extracted content')).toContainText('Morning Example');
  await page.getByRole('button', { name: 'Keep review draft', exact: true }).click();
  await page.getByRole('button', { name: 'Close dialog' }).click();
  if (process.env.COWWORKER_NATIVE_PUBLIC_URL) {
    await openImport();
    await page
      .getByLabel('Paste text', { exact: true })
      .fill(process.env.COWWORKER_NATIVE_PUBLIC_URL);
    await expect(page.getByLabel('Public source URL', { exact: true })).toBeVisible();
    await page
      .getByLabel('Fetch this public URL and its redirects. No model will be called.')
      .check();
    await page.getByRole('button', { name: 'Add to import queue' }).click();
    await expect(page.getByRole('button', { name: 'Review Web source', exact: true })).toBeVisible({
      timeout: 45000,
    });
    await page.getByRole('button', { name: 'Review Web source', exact: true }).click();
    await expect(page.getByLabel('Extracted content')).toContainText('Example Domain');
    await page.getByRole('button', { name: 'Keep review draft', exact: true }).click();
    await page.getByRole('button', { name: 'Close dialog' }).click();
  }
  return { originalId: original.id, vacancyId: vacancy.id, originalText };
}
