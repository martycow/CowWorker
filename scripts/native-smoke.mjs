import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { once } from 'node:events';

if (process.platform !== 'win32')
  throw new Error('The native smoke test currently requires Windows WebView2.');
const dataDir = await mkdtemp(join(tmpdir(), 'cowworker-native-'));
const output = resolve('output/verification');
await mkdir(output, { recursive: true });
const errors = [];
async function start() {
  const server = createServer();
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const port = server.address().port;
  await new Promise((r) => server.close(r));
  const processHandle = spawn(resolve('target/debug/cowworker.exe'), [], {
    windowsHide: true,
    env: {
      ...process.env,
      COWWORKER_DATA_DIR: dataDir,
      WEBVIEW2_USER_DATA_FOLDER: join(dataDir, 'webview'),
      WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    },
    stdio: 'ignore',
  });
  let browser;
  for (let attempt = 0; attempt < 100; attempt++) {
    if (processHandle.exitCode !== null)
      throw new Error(`CowWorker exited with ${processHandle.exitCode}`);
    try {
      browser = await chromium.connectOverCDP(`http://127.0.0.1:${port}`);
      break;
    } catch {
      await new Promise((r) => setTimeout(r, 200));
    }
  }
  if (!browser) {
    processHandle.kill();
    throw new Error('WebView2 debugging endpoint did not become available.');
  }
  const context = browser.contexts()[0];
  let page;
  for (let attempt = 0; attempt < 50; attempt++) {
    page = context.pages()[0];
    if (page) break;
    await new Promise((r) => setTimeout(r, 100));
  }
  if (!page) throw new Error('CowWorker did not create a WebView.');
  page.on('pageerror', (e) => errors.push(e.message));
  await expect(page.getByText('Local workspace', { exact: true })).toBeVisible();
  return { page, browser, processHandle };
}
async function stop(session) {
  await session.browser.close();
  if (session.processHandle.exitCode === null) {
    session.processHandle.kill();
    await once(session.processHandle, 'exit');
  }
}
let session;
try {
  session = await start();
  const { page } = session;
  await page.getByRole('button', { name: 'Add vacancy', exact: true }).click();
  await page.getByLabel('Role', { exact: true }).fill('Senior Frontend Engineer');
  await page.getByLabel('Company', { exact: true }).fill('Northstar Labs · QA example');
  await page.getByLabel('Location', { exact: true }).fill('United States');
  await page.getByLabel('Work mode', { exact: true }).selectOption('Remote');
  await page.getByLabel('Source URL').fill('https://example.com/native-test');
  await page
    .getByLabel('Original job description')
    .fill(
      'Build accessible interfaces for a product that helps people make sense of complex data.\n\nRequirements\n• React and TypeScript\n• Accessibility and semantic HTML\n• Experience with data visualization',
    );
  await page.getByLabel('Your notes').fill('Fictional record in a temporary QA workspace.');
  await page.getByRole('button', { name: 'Save vacancy', exact: true }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await page.getByRole('button', { name: 'Shortlist', exact: true }).last().click();
  await expect(page.getByRole('button', { name: 'Unshortlist' })).toBeVisible();
  await page.screenshot({ path: join(output, 'windows-vacancies.png'), fullPage: true });
  await page.getByRole('button', { name: 'Hide sidebar' }).click();
  await expect(page.getByRole('button', { name: 'Show sidebar' })).toBeFocused();
  await page.screenshot({ path: join(output, 'windows-sidebar-hidden.png'), fullPage: true });
  await page.getByRole('button', { name: 'Show sidebar' }).click();
  await page.getByRole('button', { name: 'Documents', exact: true }).click();
  await page.getByRole('button', { name: 'New document', exact: true }).click();
  await page.getByLabel('Document title').fill('Native QA resume');
  await page.getByLabel('Content', { exact: true }).fill('Original resume v1.\nТочный текст.');
  await page.getByRole('button', { name: 'Save document', exact: true }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  const downloadEvent = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export document version' }).click();
  const download = await downloadEvent;
  await download.saveAs(join(output, 'native-export.txt'));
  expect(await readFile(join(output, 'native-export.txt'), 'utf8')).toBe(
    'Original resume v1.\nТочный текст.',
  );
  await page.getByRole('button', { name: 'Vacancies', exact: true }).click();
  await page.getByRole('button', { name: 'Prepare application', exact: true }).click();
  await page.getByRole('button', { name: 'Update application' }).click();
  await page.getByLabel('Stage', { exact: true }).selectOption('applied');
  await page.getByLabel('Resume · v1', { exact: true }).check();
  await page.getByLabel('Next action', { exact: true }).fill('Follow up with the recruiter');
  await page.getByLabel('Due date').fill('2026-01-01');
  await page.getByRole('button', { name: 'Save application', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Submitted documents' })).toBeVisible();
  await page.screenshot({ path: join(output, 'windows-application.png'), fullPage: true });
  await page.getByRole('button', { name: 'Documents', exact: true }).click();
  await page.getByRole('button', { name: 'Edit latest' }).click();
  await page.getByLabel('Content', { exact: true }).fill('Updated resume v2.');
  await page.getByRole('button', { name: 'Save new version' }).click();
  await expect(page.getByRole('dialog')).not.toBeVisible();
  await page.getByLabel('Document version', { exact: true }).selectOption({ index: 1 });
  await expect(page.locator('.document-paper')).toContainText('Original resume v1.');
  await stop(session);
  session = null;
  session = await start();
  const reopened = session.page;
  await expect(reopened.getByRole('button', { name: /Senior Frontend Engineer/ })).toBeVisible();
  await reopened.getByRole('button', { name: 'Applications', exact: true }).click();
  await expect(reopened.locator('.detail-panel')).toContainText('Native QA resume · v1');
  await reopened.getByRole('button', { name: 'Update application' }).click();
  await expect(reopened.getByLabel('Resume · v1', { exact: true })).toBeChecked();
  await expect(reopened.getByLabel('Resume · v1', { exact: true })).toBeDisabled();
  await reopened.getByRole('button', { name: 'Close dialog' }).click();
  await reopened.getByRole('button', { name: /^Today/ }).click();
  await expect(
    reopened.getByRole('button', { name: /Follow up with the recruiter/ }),
  ).toBeVisible();
  await reopened.screenshot({ path: join(output, 'windows-today.png'), fullPage: true });
  expect(errors).toEqual([]);
  const report = {
    passed: true,
    platform: process.platform,
    dataDir,
    checks: [
      'native Tauri IPC',
      'SQLite persistence across process restart',
      'vacancy capture and shortlist',
      'sidebar focus and visibility',
      'document creation and native text export',
      'application submission and next action',
      'immutable submitted version after editing',
      'Today due actions',
      'no frontend runtime errors',
    ],
    screenshots: [
      'windows-vacancies.png',
      'windows-sidebar-hidden.png',
      'windows-application.png',
      'windows-today.png',
    ],
  };
  await writeFile(join(output, 'native-report.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
} finally {
  if (session) await stop(session);
}
