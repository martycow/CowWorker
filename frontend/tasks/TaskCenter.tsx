import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { native } from '../api';
import { Modal } from '../components';
export interface Task {
  id: string;
  kind: string;
  status: string;
  stage: string;
  progress: number;
  createdAt: number;
  updatedAt: number;
  attempts: number;
  revision: number;
  error: string | null;
  waitingReason: string | null;
  result: Record<string, unknown> | null;
}
export function TaskCenter() {
  const [open, setOpen] = useState(false);
  const [tasks, setTasks] = useState<Task[]>([]);
  const [error, setError] = useState('');
  const [offset, setOffset] = useState(0);
  const [backup, setBackup] = useState<{
    directory: string;
    manifest: { createdAt: string; schemaVersion: number; files: Record<string, string> };
  } | null>(null);
  const [restoring, setRestoring] = useState(false);
  const refresh = async () => {
    if (native) setTasks(await invoke('list_tasks', { offset, limit: 25 }));
  };
  useEffect(() => {
    let active = true;
    const load = () => {
      if (native)
        void invoke<Task[]>('list_tasks', { offset, limit: 25 })
          .then((rows) => {
            if (active) setTasks(rows);
          })
          .catch((e) => {
            if (active) setError(String(e));
          });
    };
    const subscription = native ? listen('tasks-changed', load) : null;
    load();
    const timer = setInterval(load, 2000);
    window.addEventListener('focus', load);
    return () => {
      active = false;
      clearInterval(timer);
      window.removeEventListener('focus', load);
      void subscription?.then((unlisten) => unlisten());
    };
  }, [offset]);
  const action = async (command: string, id?: string) => {
    setError('');
    try {
      await invoke(command, id ? { id } : {});
      await refresh();
    } catch (e) {
      setError(String(e));
    }
  };
  return (
    <>
      <button onClick={() => setOpen(true)}>
        Tasks{' '}
        {tasks.filter((t) => ['running', 'queued', 'waiting'].includes(t.status)).length || ''}
      </button>
      {open && (
        <Modal title="Task Center" onClose={() => setOpen(false)} busy={restoring}>
          <p>Tasks continue while you navigate. Safe work resumes when CowWorker opens again.</p>
          {!native && (
            <p>
              Background tasks run in the desktop workspace. This browser demo has separate data.
            </p>
          )}
          {native && (
            <div className="version-toolbar">
              <button onClick={() => void action('enqueue_backup')}>Back up workspace</button>
              <button
                disabled={restoring}
                onClick={() => {
                  void invoke<typeof backup>('choose_backup')
                    .then(setBackup)
                    .catch((e) => setError(String(e)));
                }}
              >
                Restore a backup
              </button>
            </div>
          )}
          {backup && (
            <section className="task-item">
              <strong>
                Verified backup · {new Date(backup.manifest.createdAt).toLocaleString()}
              </strong>
              <p className="asset-path">{backup.directory}</p>
              <p>
                {Object.keys(backup.manifest.files).length} verified files. CowWorker will finish
                the current task, open a restored copy, and reload. Your current workspace stays in
                its original folder.
              </p>
              <button
                disabled={restoring}
                onClick={() => {
                  setRestoring(true);
                  void invoke('restore_workspace', { directory: backup.directory })
                    .then(() => window.location.reload())
                    .catch((e) => {
                      setError(String(e));
                      setRestoring(false);
                    });
                }}
              >
                {restoring ? 'Restoring…' : 'Open restored copy'}
              </button>
              <button disabled={restoring} onClick={() => setBackup(null)}>
                Keep current workspace
              </button>
            </section>
          )}
          {error && (
            <p role="alert" className="error">
              {error}
            </p>
          )}
          {!tasks.length && <p>No tasks yet.</p>}
          <div className="task-list">
            {tasks.map((task) => (
              <article className="task-item" key={task.id}>
                <strong>{task.kind === 'backup' ? 'Workspace backup' : task.kind}</strong>
                <span>
                  {task.status} · {task.stage}
                </span>
                <progress value={task.progress} max={100} aria-label={`${task.kind} progress`} />
                <small>
                  {task.attempts} attempts ·{' '}
                  {Math.max(
                    0,
                    (task.status === 'running' ? Math.floor(Date.now() / 1000) : task.updatedAt) -
                      task.createdAt,
                  )}{' '}
                  seconds
                </small>
                {task.waitingReason && <p>{task.waitingReason}</p>}
                {task.error && <p role="alert">{task.error}</p>}
                {typeof task.result?.directory === 'string' && (
                  <p className="asset-path">Saved to {task.result.directory}</p>
                )}
                {['queued', 'running', 'waiting'].includes(task.status) && (
                  <button onClick={() => void action('cancel_task', task.id)}>Cancel</button>
                )}
                {['failed', 'cancelled'].includes(task.status) && (
                  <button onClick={() => void action('retry_task', task.id)}>Retry</button>
                )}
              </article>
            ))}
          </div>
          <div className="version-toolbar">
            <button disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - 25))}>
              Previous
            </button>
            <span>Page {offset / 25 + 1}</span>
            <button disabled={tasks.length < 25} onClick={() => setOffset(offset + 25)}>
              Next
            </button>
          </div>
        </Modal>
      )}
    </>
  );
}
