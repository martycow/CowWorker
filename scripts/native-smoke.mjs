import { chromium, expect } from '@playwright/test';
import { spawn } from 'node:child_process';
import { mkdir, mkdtemp, readFile, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { resolve, join } from 'node:path';
import { createServer } from 'node:net';
import { once } from 'node:events';
import { createServer as createHttpServer } from 'node:http';
import { morningFlow, morningLetter } from './morning-native.mjs';
import { DatabaseSync } from 'node:sqlite';

if (process.platform !== 'win32')
  throw new Error('The native smoke test currently requires Windows WebView2.');
const dataDir = await mkdtemp(join(tmpdir(), 'cowworker-native-'));
const output = resolve('output/verification');
await mkdir(output, { recursive: true });
const errors = [];
const modelRequests = [];
const fixtureServer = createHttpServer(async (req, res) => {
  if (req.method === 'GET' && req.url === '/v1/models') {
    res.setHeader('Content-Type', 'application/json');
    res.end(JSON.stringify({ data: [{ id: 'fixture' }] }));
    return;
  }
  let body = '';
  for await (const chunk of req) body += chunk;
  const payload = JSON.parse(body);
  modelRequests.push({ url: req.url, body: payload });
  const evidence = JSON.parse(payload.messages.at(-1).content);
  const isLetter = evidence.purpose === 'cover-letter';
  if (isLetter) {
    expect(evidence.evidence.vacancy.company).toBe('Morning Example');
    expect(evidence.evidence.originalResume.content).toContain('Example Studio');
  }
  if (req.url === '/slow') return;
  res.setHeader('Content-Type', 'application/json');
  res.end(
    JSON.stringify({
      id: 'native-fixture',
      choices: [
        {
          message: {
            content: JSON.stringify({
              summary: isLetter
                ? 'Fixture cover letter based on the supplied resume and vacancy.'
                : 'Fixture analysis using only the supplied vacancy.',
              fields: isLetter ? null : { notes: 'Reviewed native AI note' },
              content: isLetter ? morningLetter : null,
            }),
          },
        },
      ],
      usage: { prompt_tokens: 12, completion_tokens: 7, total_tokens: 19 },
    }),
  );
});
fixtureServer.listen(0, '127.0.0.1');
await once(fixtureServer, 'listening');
const fixtureEndpoint = `http://127.0.0.1:${fixtureServer.address().port}`;
const ipc = (page, command, args = {}) =>
  page.evaluate(({ command, args }) => window.__TAURI_INTERNALS__.invoke(command, args), {
    command,
    args,
  });
async function start(directory = dataDir) {
  const server = createServer();
  server.listen(0, '127.0.0.1');
  await once(server, 'listening');
  const port = server.address().port;
  await new Promise((r) => server.close(r));
  const processHandle = spawn(resolve('target/debug/cowworker.exe'), [], {
    windowsHide: true,
    env: {
      ...process.env,
      COWWORKER_DATA_DIR: directory,
      WEBVIEW2_USER_DATA_FOLDER: join(directory, 'webview'),
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
  page.setDefaultTimeout(15000);
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
  console.log('Checking native import, Company Hub, AI runtime, and backup.');
  await reopened.getByRole('button', { name: 'Universal Add', exact: true }).click();
  await reopened
    .getByLabel('Paste text', { exact: true })
    .fill('Native import fixture\nТочный импортированный текст');
  await reopened.getByRole('button', { name: 'Add to import queue' }).click();
  await expect(reopened.getByRole('button', { name: 'Review Pasted document' })).toBeVisible({
    timeout: 45000,
  });
  await reopened.getByRole('button', { name: 'Review Pasted document' }).click();
  await reopened.getByLabel('Category', { exact: true }).selectOption('reference');
  await reopened.getByRole('button', { name: 'Keep review draft' }).click();
  await reopened.getByRole('button', { name: 'Close dialog' }).click();
  const savedDraft = (await ipc(reopened, 'list_imports', { offset: 0 }))[0];
  expect(savedDraft.draft.kind).toBe('reference');
  await ipc(reopened, 'review_import', {
    id: savedDraft.id,
    expectedRevision: savedDraft.revision,
    draft: savedDraft.draft,
    save: true,
  });
  await reopened.getByRole('button', { name: 'Companies', exact: true }).click();
  await expect(reopened.getByRole('button', { name: /Northstar Labs · QA example/ })).toBeVisible();
  await reopened.getByRole('button', { name: /Northstar Labs · QA example/ }).click();
  await expect(reopened.getByRole('heading', { name: 'Employment history' })).toBeVisible();
  await reopened.screenshot({ path: join(output, 'windows-companies.png'), fullPage: true });
  await reopened.getByRole('button', { name: 'Vacancies', exact: true }).click();
  await reopened.getByRole('button', { name: '✦ AI', exact: true }).click();
  await reopened.getByRole('button', { name: 'AI settings', exact: true }).click();
  await reopened.getByLabel('provider', { exact: true }).fill('Native fixture');
  await reopened.getByLabel('model', { exact: true }).fill('fixture');
  await reopened
    .getByLabel('Full completion endpoint')
    .fill(`${fixtureEndpoint}/v1/chat/completions`);
  await reopened.getByLabel('Local model on this computer').check();
  await reopened.getByRole('button', { name: 'Save AI settings', exact: true }).click();
  expect(await ipc(reopened, 'discover_models')).toEqual(['fixture']);
  expect(modelRequests).toHaveLength(0);
  await reopened.getByRole('button', { name: '✦ Preview AI Operation', exact: true }).click();
  await reopened
    .getByLabel('Allow this operation to send the displayed data to this destination.')
    .check();
  await reopened.getByRole('button', { name: '✦ Run AI Operation', exact: true }).click();
  await expect(
    reopened.getByText('Fixture analysis using only the supplied vacancy.', { exact: true }),
  ).toBeVisible({ timeout: 45000 });
  await reopened.getByRole('button', { name: 'Review suggested changes', exact: true }).click();
  await reopened.getByLabel('Replace my existing values when applying field proposals').check();
  await reopened.getByRole('button', { name: 'Apply proposal', exact: true }).click();
  await expect
    .poll(async () => (await ipc(reopened, 'load_workspace')).vacancies[0].notes)
    .toBe('Reviewed native AI note');
  await reopened.getByRole('button', { name: 'Close AI', exact: true }).click();
  await reopened.getByRole('button', { name: 'AI Usage', exact: true }).click();
  await expect(reopened.getByRole('heading', { name: 'Attempt history' })).toBeVisible();
  const ledger = await ipc(reopened, 'ai_usage', {
    filter: {
      from: null,
      to: null,
      provider: null,
      model: null,
      feature: null,
      entityId: null,
      offset: 0,
      limit: 25,
    },
  });
  expect(ledger.totals.totalTokens).toBe(19);
  expect(ledger.totals.attempts).toBe(1);
  expect(ledger.costs[0].actualMicros).toBe(0);
  await reopened.screenshot({ path: join(output, 'windows-ai-usage.png'), fullPage: true });
  const backupId = await ipc(reopened, 'enqueue_backup');
  await expect
    .poll(
      async () =>
        (await ipc(reopened, 'list_tasks', { offset: 0, limit: 100 })).find(
          (t) => t.id === backupId,
        )?.status,
      { timeout: 45000 },
    )
    .toBe('completed');
  const backupTask = (await ipc(reopened, 'list_tasks', { offset: 0, limit: 100 })).find(
    (t) => t.id === backupId,
  );
  const manifest = JSON.parse(
    await readFile(join(backupTask.result.directory, 'manifest.json'), 'utf8'),
  );
  expect(manifest.schemaVersion).toBe(9);
  expect(Object.keys(manifest.files).some((p) => p.startsWith('sources/'))).toBe(true);
  console.log('Checking native interruption after model dispatch.');
  const config = await ipc(reopened, 'provider_config');
  await ipc(reopened, 'configure_provider', {
    config: { ...config, endpoint: `${fixtureEndpoint}/slow` },
    secret: null,
  });
  const vacancy = (await ipc(reopened, 'load_workspace')).vacancies[0];
  const request = {
    operationType: 'job-analysis',
    entityType: 'vacancy',
    entityId: vacancy.id,
    baseRevision: vacancy.revision,
    baseVersion: null,
    instruction: 'Fictional recovery check',
  };
  const preview = await ipc(reopened, 'preview_ai', { request });
  await ipc(reopened, 'start_ai', { request, scope: preview.scope, key: 'native-interruption' });
  await expect.poll(() => modelRequests.filter((r) => r.url === '/slow').length).toBe(1);
  await stop(session);
  session = null;
  session = await start();
  await expect
    .poll(
      async () =>
        (await ipc(session.page, 'list_tasks', { offset: 0, limit: 100 })).find(
          (t) => t.kind === 'ai-operation' && t.status === 'waiting',
        )?.waitingReason,
      { timeout: 45000 },
    )
    .toBe('unknown-remote-outcome');
  expect(modelRequests.filter((r) => r.url === '/slow')).toHaveLength(1);
  expect((await ipc(session.page, 'list_imports', { offset: 0 }))[0].status).toBe('saved');
  console.log('Checking native restore and persistent workspace switch.');
  const restored = await ipc(session.page, 'restore_workspace', {
    directory: backupTask.result.directory,
  });
  expect(restored).toContain('restored');
  const restoredWorkspace = await ipc(session.page, 'load_workspace');
  expect(restoredWorkspace.vacancies[0].notes).toBe('Reviewed native AI note');
  expect((await ipc(session.page, 'list_imports', { offset: 0 }))[0].status).toBe('saved');
  expect(JSON.parse(await readFile(join(dataDir, 'active-workspace.json'), 'utf8'))).toBe(
    restored.split(/[\\/]/).at(-1),
  );
  await stop(session);
  session = null;
  session = await start();
  expect((await ipc(session.page, 'load_workspace')).vacancies[0].notes).toBe(
    'Reviewed native AI note',
  );
  expect(errors).toEqual([]);
  const morning = await morningFlow(session.page, ipc, output);
  await stop(session);
  session = null;
  session = await start();
  const morningWorkspace = await ipc(session.page, 'load_workspace');
  const morningOriginal = morningWorkspace.documents.find((d) => d.id === morning.originalId);
  expect(morningOriginal.versions).toHaveLength(1);
  expect(
    await ipc(session.page, 'document_content', { versionId: morningOriginal.versions[0].id }),
  ).toBe(morning.originalText);
  expect(
    morningWorkspace.documents.find(
      (d) => d.title.includes('Cover letter') && d.title.includes('Morning Example'),
    ).versions,
  ).toHaveLength(3);
  await stop(session);
  session = null;
  const activeName = JSON.parse(await readFile(join(dataDir, 'active-workspace.json'), 'utf8'));
  const futureDb = new DatabaseSync(join(dataDir, 'restored', activeName, 'cowworker.db'));
  futureDb.exec('PRAGMA user_version=999');
  futureDb.close();
  session = await start();
  await expect(
    session.page.getByText(/This workspace was created by a newer CowWorker version/),
  ).toBeVisible();
  await ipc(session.page, 'restore_workspace', { directory: backupTask.result.directory });
  expect((await ipc(session.page, 'load_workspace')).vacancies[0].notes).toBe(
    'Reviewed native AI note',
  );
  await stop(session);
  session = null;
  const legacyDir = await mkdtemp(join(tmpdir(), 'cowworker-legacy-'));
  const legacyDb = new DatabaseSync(join(legacyDir, 'cowworker.db'));
  legacyDb.exec(await readFile(resolve('backend/src/schema.sql'), 'utf8'));
  legacyDb.exec(
    "INSERT INTO vacancies VALUES ('legacy-v','Legacy role','Legacy company','','Remote','','Exact legacy source','Original note','saved','2025-01-01','2025-01-01'); INSERT INTO applications VALUES ('legacy-a','legacy-v','preparing','','','Original application',NULL,'2025-01-01','2025-01-01');",
  );
  legacyDb.close();
  session = await start(legacyDir);
  const legacyWorkspace = await ipc(session.page, 'load_workspace');
  expect(legacyWorkspace.schemaVersion).toBe(9);
  expect(legacyWorkspace.vacancies).toHaveLength(1);
  expect(legacyWorkspace.vacancies[0].description).toBe('Exact legacy source');
  expect(legacyWorkspace.applications).toHaveLength(1);
  expect(legacyWorkspace.applications[0].notes).toBe('Original application');
  expect(errors).toEqual([]);
  const report = {
    passed: true,
    platform: process.platform,
    dataDir,
    publicUrlChecked: process.env.COWWORKER_NATIVE_PUBLIC_URL ?? null,
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
      'persistent native import review and source preservation',
      'manual Company Hub with related applications',
      'native HTTP AI adapter against a loopback fixture',
      'model discovery without model execution or workspace transmission',
      'explicit AI scope approval and guarded proposal application',
      'central usage ledger with reported token totals',
      'queued workspace backup with source hashes',
      'process interruption after dispatch does not duplicate the model call',
      'verified restore to a separate directory and active workspace persistence across restart',
      'native PDF and DOCX resume import, isolated Windows screenshot and scanned PDF OCR',
      'two screenshot sources combined in reading order and retained on a vacancy',
      'separate tailored resume and cover letter with original version preserved after restart',
      'contextual cover letter fixture generation, reviewed proposal, manual editing and PDF downloads',
      'version in Settings popover and Escape dismissal',
      'failed startup keeps recovery available and verified restore clears its error',
      'legacy database migrates before runner and UI open storage, with original records retained',
    ],
    screenshots: [
      'windows-vacancies.png',
      'windows-sidebar-hidden.png',
      'windows-application.png',
      'windows-today.png',
      'windows-companies.png',
      'windows-ai-usage.png',
      'windows-settings-version.png',
      'windows-screenshot-review.png',
      'windows-tailored-resume.png',
      'windows-cover-letter.png',
    ],
  };
  await writeFile(join(output, 'native-report.json'), JSON.stringify(report, null, 2));
  console.log(JSON.stringify(report, null, 2));
} catch (error) {
  if (session) {
    await session.page
      .screenshot({ path: join(output, 'windows-failure.png'), fullPage: true })
      .catch(() => {});
    await writeFile(
      join(output, 'native-failure.json'),
      JSON.stringify(
        {
          error: String(error),
          dataDir,
          body: await session.page
            .locator('body')
            .innerText()
            .catch(() => ''),
        },
        null,
        2,
      ),
    );
  }
  throw error;
} finally {
  if (session) await stop(session);
  fixtureServer.closeAllConnections();
  await new Promise((resolve) => fixtureServer.close(resolve));
}
