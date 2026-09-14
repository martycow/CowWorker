import { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { Modal, Field } from './components';
import type { Vacancy, CareerDocument } from './types';

export function JobDocuments({
  vacancy,
  documents,
  onClose,
  onCreated,
}: {
  vacancy: Vacancy;
  documents: CareerDocument[];
  onClose: () => void;
  onCreated: (id: string) => Promise<void>;
}) {
  const resumes = documents.filter((document) => document.kind === 'resume');
  const [version, setVersion] = useState(resumes[0]?.versions[0]?.id ?? '');
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState('');
  const create = async (kind: string) => {
    setBusy(true);
    setError('');
    try {
      const id = await invoke<string>('prepare_job_document', {
        vacancyId: vacancy.id,
        expectedRevision: vacancy.revision,
        resumeVersionId: version,
        kind,
      });
      await onCreated(id);
    } catch (e) {
      setError(String(e));
    } finally {
      setBusy(false);
    }
  };
  return (
    <Modal title="Prepare documents" onClose={onClose} busy={busy}>
      <p>
        <strong>{vacancy.title}</strong> · {vacancy.company}
      </p>
      <p>
        Create a separate document for this role. Your original resume stays unchanged. Write the
        content yourself, or use the AI panel with this vacancy and resume as context.
      </p>
      {resumes.length ? (
        <>
          <Field label="Original resume version">
            <select value={version} onChange={(e) => setVersion(e.target.value)}>
              {resumes.flatMap((document) =>
                document.versions.map((v) => (
                  <option key={v.id} value={v.id}>
                    {document.title} · version {v.number}
                  </option>
                )),
              )}
            </select>
          </Field>
          <div className="version-toolbar">
            <button
              className="primary"
              disabled={busy || !version}
              onClick={() => void create('resume')}
            >
              Create tailored resume
            </button>
            <button disabled={busy || !version} onClick={() => void create('cover-letter')}>
              Create cover letter
            </button>
          </div>
        </>
      ) : (
        <>
          <p>First import your old resume and save it with the Resume category.</p>
          <button
            onClick={() => {
              onClose();
              window.dispatchEvent(new Event('open-universal-add'));
            }}
          >
            Import resume
          </button>
        </>
      )}
      {error && (
        <p role="alert" className="error">
          {error}
        </p>
      )}
    </Modal>
  );
}
