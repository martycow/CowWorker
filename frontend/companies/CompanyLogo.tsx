import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { CompanyMark } from '../components';
export function CompanyLogo({
  id,
  name,
  revision,
}: {
  id: string;
  name: string;
  revision: number;
}) {
  const [url, setUrl] = useState('');
  useEffect(() => {
    let active = true;
    let object = '';
    void invoke<number[] | null>('company_logo', { companyId: id })
      .then((bytes) => {
        if (bytes && active) {
          object = URL.createObjectURL(new Blob([new Uint8Array(bytes)], { type: 'image/png' }));
          setUrl(object);
        } else if (active) setUrl('');
      })
      .catch(() => {
        if (active) setUrl('');
      });
    return () => {
      active = false;
      if (object) URL.revokeObjectURL(object);
    };
  }, [id, revision]);
  return url ? (
    <img className="company-logo" src={url} alt={`${name} logo`} onError={() => setUrl('')} />
  ) : (
    <CompanyMark name={name} />
  );
}
