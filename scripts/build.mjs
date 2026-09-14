import { readFile, writeFile, open, unlink } from 'node:fs/promises';
import { spawn } from 'node:child_process';
import { resolve } from 'node:path';

const tauri = process.argv[2] === 'tauri';
const args = process.argv.slice(3);
const versioned = !process.env.COWORKER_VERSION_RESERVED && (!tauri || args[0] === 'build');
const packagePath = resolve('package.json');
const initial = JSON.parse(await readFile(packagePath, 'utf8')).version;
const candidate = initial
  .split('.')
  .map((part, index) => Number(part) + (index === 2 ? 1 : 0))
  .join('.');
const run = (script, arguments_) =>
  new Promise((resolve_, reject) => {
    const child = spawn(process.execPath, [script, ...arguments_], {
      stdio: 'inherit',
      windowsHide: true,
      env: { ...process.env, ...(versioned ? { COWORKER_VERSION_RESERVED: candidate } : {}) },
    });
    child.on('error', reject);
    child.on('exit', (code) =>
      code === 0 ? resolve_() : reject(new Error(`Build exited with ${code}`)),
    );
  });
async function synchronize(version) {
  for (const path of ['package.json', 'package-lock.json', 'src-tauri/tauri.conf.json']) {
    const data = JSON.parse(await readFile(path, 'utf8'));
    data.version = version;
    if (data.packages?.['']) data.packages[''].version = version;
    await writeFile(path, JSON.stringify(data, null, 2) + '\n');
  }
  for (const path of ['backend/Cargo.toml', 'src-tauri/Cargo.toml']) {
    const text = await readFile(path, 'utf8');
    await writeFile(path, text.replace(/^version = "[^"]+"/m, `version = "${version}"`));
  }
  const lock = await readFile('Cargo.lock', 'utf8');
  await writeFile(
    'Cargo.lock',
    lock.replace(/(name = "cowworker(?:-core)?"\r?\nversion = ")[^"]+/g, `$1${version}`),
  );
}
let lock;
try {
  if (versioned) {
    lock = await open('.cowworker-build.lock', 'wx');
    await synchronize(candidate);
    console.log(`Building CowWorker v${candidate}`);
  }
  if (tauri) await run(resolve('node_modules/@tauri-apps/cli/tauri.js'), args);
  else {
    await run(resolve('node_modules/typescript/bin/tsc'), ['--noEmit']);
    await run(resolve('node_modules/vite/bin/vite.js'), ['build']);
  }
} catch (error) {
  if (lock) await synchronize(initial);
  console.error(String(error));
  process.exitCode = 1;
} finally {
  if (lock) {
    await lock.close();
    await unlink('.cowworker-build.lock');
  }
}
