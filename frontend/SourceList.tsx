import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { native } from './api';
interface Source {
  id: string;
  name: string;
  mediaType: string;
  sha256: string;
  byteLength: number;
  createdAt: string;
  sourceUrl: string | null;
}
export function SourceList({ entityId }: { entityId: string }) {
  const [sources, setSources] = useState<Source[]>([]);
  const [error, setError] = useState('');
  useEffect(() => {
    let active = true;
    if (native)
      void invoke<Source[]>('list_sources', { entityId })
        .then((s) => {
          if (active) setSources(s);
        })
        .catch((e) => {
          if (active) setError(String(e));
        });
    return () => {
      active = false;
    };
  }, [entityId]);
  return (
    <details>
      <summary>Original sources ({sources.length})</summary>
      {sources.map((source) => (
        <article className="task-item" key={source.id}>
          <strong>{source.name}</strong>
          <small>
            {source.mediaType} · {source.byteLength.toLocaleString()} bytes ·{' '}
            {new Date(source.createdAt).toLocaleString()}
          </small>
          {source.sourceUrl && <span className="asset-path">{source.sourceUrl}</span>}
          <code className="asset-path">SHA-256 {source.sha256}</code>
          <button
            onClick={() =>
              void invoke<number[]>('source_bytes', { id: source.id })
                .then((bytes) => {
                  const url = URL.createObjectURL(
                    new Blob([new Uint8Array(bytes)], { type: source.mediaType }),
                  );
                  const a = document.createElement('a');
                  a.href = url;
                  a.download = source.name.replace(/[<>:"/\\|?*]/g, '_');
                  a.click();
                  setTimeout(() => URL.revokeObjectURL(url), 1000);
                })
                .catch((e) => setError(String(e)))
            }
          >
            Export original source
          </button>
        </article>
      ))}
      {error && <p role="alert">{error}</p>}
    </details>
  );
}
