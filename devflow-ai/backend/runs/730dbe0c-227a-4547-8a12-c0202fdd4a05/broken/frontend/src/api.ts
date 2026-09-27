// UploadLab API client — BROKEN FIXTURE
// BUG-001: Uses wrong multipart field name "file" instead of documented "files"
// Do NOT fix this file — it is the immutable broken baseline.

import axios from 'axios';

const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3001';

export interface UploadedFile {
  filename: string;
  size_bytes: number;
  status: string;
}

export interface UploadResponse {
  uploaded: UploadedFile[];
  count: number;
}

export async function uploadFiles(files: File[]): Promise<UploadResponse> {
  const formData = new FormData();

  for (const file of files) {
    // BUG-001: Field name should be "files" (plural) per API contract.
    // Using "file" (singular) violates the documented multipart field name.
    formData.append('file', file, file.name);
  }

  const response = await axios.post<UploadResponse>(`${API_BASE}/api/upload`, formData, {
    headers: {
      'Content-Type': 'multipart/form-data',
    },
  });

  return response.data;
}

export function getExpectedFieldName(): string {
  // The documented correct field name according to the API contract
  return 'files';
}

export function getActualFieldName(): string {
  // The incorrect field name currently used by this broken implementation
  return 'file';
}
