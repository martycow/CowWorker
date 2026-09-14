import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { native } from '../api';
import { Field } from '../components';
interface Ledger {
  totals: {
    operations: number;
    attempts: number;
    inputTokens: number | null;
    outputTokens: number | null;
    totalTokens: number | null;
    unknownCostAttempts: number;
  };
  costs: { currency: string | null; actualMicros: number | null; estimatedMicros: number | null }[];
  history: {
    attemptId: string;
    operationId: string;
    feature: string;
    provider: string;
    model: string;
    status: string;
    inputTokens: number | null;
    outputTokens: number | null;
    totalTokens: number | null;
    actualMicros: number | null;
    currency: string | null;
    startedAt: number;
  }[];
  groups: Record<string, { label: string; attempts: number; tokens: number | null }[]>;
}
export function AiUsage() {
  const [data, setData] = useState<Ledger | null>(null);
  const [error, setError] = useState('');
  const [filters, setFilters] = useState({
    from: '',
    to: '',
    provider: '',
    model: '',
    feature: '',
    entityId: '',
  });
  const [offset, setOffset] = useState(0);
  const [retry, setRetry] = useState(0);
  useEffect(() => {
    if (!native) return;
    let active = true;
    const to = filters.to ? new Date(`${filters.to}T00:00:00`) : null;
    if (to) to.setDate(to.getDate() + 1);
    void invoke<Ledger>('ai_usage', {
      filter: {
        ...filters,
        from: filters.from
          ? Math.floor(new Date(`${filters.from}T00:00:00`).getTime() / 1000)
          : null,
        to: to ? Math.floor(to.getTime() / 1000) : null,
        provider: filters.provider || null,
        model: filters.model || null,
        feature: filters.feature || null,
        entityId: filters.entityId || null,
        offset,
        limit: 25,
      },
    })
      .then((d) => {
        if (active) {
          setData(d);
          setError('');
        }
      })
      .catch((e) => {
        if (active) setError(String(e));
      });
    return () => {
      active = false;
    };
  }, [filters, offset, retry]);
  const count = (value: number | null | undefined) =>
    value == null ? 'Unknown' : value.toLocaleString();
  return (
    <div className="ai-usage">
      <p>
        Logical operations and provider attempts are counted separately. Unknown usage is not zero.
      </p>
      {!native && <p>Open the desktop workspace to view its AI ledger.</p>}
      <div className="form-grid">
        {Object.entries(filters).map(([key, value]) => (
          <Field key={key} label={key === 'entityId' ? 'Vacancy or company ID' : key}>
            <input
              type={['from', 'to'].includes(key) ? 'date' : 'text'}
              value={value}
              onChange={(e) => {
                setFilters({ ...filters, [key]: e.target.value });
                setOffset(0);
              }}
            />
          </Field>
        ))}
      </div>
      {error && (
        <p role="alert">
          {error} <button onClick={() => setRetry(retry + 1)}>Retry usage query</button>
        </p>
      )}
      {data && (
        <>
          <div className="usage-summary">
            {Object.entries({
              Operations: data.totals.operations,
              Attempts: data.totals.attempts,
              'Input tokens': data.totals.inputTokens,
              'Output tokens': data.totals.outputTokens,
              'Total tokens': data.totals.totalTokens,
              'Attempts with unknown cost': data.totals.unknownCostAttempts,
            }).map(([label, value]) => (
              <article className="task-item" key={label}>
                <span>{label}</span>
                <strong>{count(value)}</strong>
              </article>
            ))}
          </div>
          <h2>Cost by currency</h2>
          {data.costs.map((c) => (
            <p key={c.currency ?? 'unknown'}>
              {c.currency ?? 'Unknown currency'} · actual{' '}
              {c.actualMicros == null ? 'Unknown' : (c.actualMicros / 1000000).toFixed(6)} ·
              estimated{' '}
              {c.estimatedMicros == null ? 'Unknown' : (c.estimatedMicros / 1000000).toFixed(6)}
            </p>
          ))}
          {Object.entries(data.groups).map(([name, rows]) => (
            <details key={name}>
              <summary>
                By {name}
                {name === 'day' ? ' (UTC)' : ''}
              </summary>
              <table>
                <thead>
                  <tr>
                    <th>{name}</th>
                    <th>Attempts</th>
                    <th>Reported tokens</th>
                    <th>Relative attempts</th>
                  </tr>
                </thead>
                <tbody>
                  {rows.map((row) => (
                    <tr key={row.label}>
                      <td>{row.label}</td>
                      <td>{row.attempts}</td>
                      <td>{count(row.tokens)}</td>
                      <td>
                        <meter
                          min={0}
                          max={Math.max(1, ...rows.map((r) => r.attempts))}
                          value={row.attempts}
                          aria-label={`${row.label}: ${row.attempts} attempts`}
                        />
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </details>
          ))}
          <h2>Attempt history</h2>
          <div className="table-scroll">
            <table>
              <thead>
                <tr>
                  <th>Time</th>
                  <th>Feature</th>
                  <th>Provider / model</th>
                  <th>Status</th>
                  <th>Tokens</th>
                  <th>Actual cost</th>
                </tr>
              </thead>
              <tbody>
                {data.history.map((row) => (
                  <tr key={row.attemptId}>
                    <td>{new Date(row.startedAt * 1000).toLocaleString()}</td>
                    <td>{row.feature}</td>
                    <td>
                      {row.provider} / {row.model}
                    </td>
                    <td>{row.status}</td>
                    <td>{count(row.totalTokens)}</td>
                    <td>
                      {row.actualMicros == null
                        ? 'Unknown'
                        : `${(row.actualMicros / 1000000).toFixed(6)} ${row.currency ?? ''}`}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
          {!data.history.length && <p>No AI attempts match these filters.</p>}
          <div className="version-toolbar">
            <button disabled={offset === 0} onClick={() => setOffset(offset - 25)}>
              Previous attempts
            </button>
            <span>Page {offset / 25 + 1}</span>
            <button disabled={data.history.length < 25} onClick={() => setOffset(offset + 25)}>
              Next attempts
            </button>
          </div>
        </>
      )}
    </div>
  );
}
