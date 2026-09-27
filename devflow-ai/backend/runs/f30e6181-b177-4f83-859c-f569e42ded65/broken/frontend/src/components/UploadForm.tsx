import React, { useState } from 'react';
import { uploadFiles } from '../api';

export function UploadForm() {
  const [files, setFiles] = useState<File[]>([]);
  const [result, setResult] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files) {
      setFiles(Array.from(e.target.files));
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (files.length === 0) {
      setError('Please select at least one file.');
      return;
    }
    setLoading(true);
    setError(null);
    setResult(null);
    try {
      const response = await uploadFiles(files);
      setResult(JSON.stringify(response, null, 2));
    } catch (err: any) {
      setError(err?.response?.data?.error || 'Upload failed');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div>
      <h1>UploadLab</h1>
      <form onSubmit={handleSubmit}>
        <input
          type="file"
          multiple
          accept=".txt,.png"
          onChange={handleFileChange}
          data-testid="file-input"
        />
        <button type="submit" disabled={loading} data-testid="upload-button">
          {loading ? 'Uploading...' : 'Upload'}
        </button>
      </form>
      {error && <div data-testid="error">{error}</div>}
      {result && <pre data-testid="result">{result}</pre>}
    </div>
  );
}
