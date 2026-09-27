// Frontend API contract tests — BROKEN FIXTURE
// These tests verify the API client's request construction.
// TEST-FRONT-01 and TEST-FRONT-02 are EXPECTED TO FAIL due to BUG-001

import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';

// Import functions under test
import { uploadFiles, getExpectedFieldName, getActualFieldName } from '../api';

vi.mock('axios');
const mockedAxios = vi.mocked(axios, true);

describe('UploadLab API client — field name contract', () => {

  beforeEach(() => {
    vi.resetAllMocks();
  });

  /**
   * TEST-FRONT-01: Verify the multipart field name matches the documented contract.
   * EXPECTED TO FAIL on broken fixture (BUG-001).
   * The client uses "file" but the contract requires "files".
   */
  it('should use field name "files" as specified in the API contract', () => {
    const actual = getActualFieldName();
    const expected = getExpectedFieldName();
    // BUG-001: This assertion fails — actual is "file", expected is "files"
    expect(actual).toBe(expected);
  });

  /**
   * TEST-FRONT-02: FormData appended with correct field name.
   * EXPECTED TO FAIL on broken fixture (BUG-001).
   */
  it('should append files with field name "files" to FormData', async () => {
    const capturedFormData: FormData[] = [];

    mockedAxios.post = vi.fn().mockImplementation((url: string, formData: FormData) => {
      capturedFormData.push(formData);
      return Promise.resolve({ data: { uploaded: [], count: 0 } });
    });

    const file = new File(['content'], 'test.txt', { type: 'text/plain' });
    await uploadFiles([file]);

    expect(capturedFormData).toHaveLength(1);
    const formData = capturedFormData[0];

    // The field name used in FormData.append() must be "files"
    const entries = Array.from(formData.entries());
    const fieldNames = entries.map(([name]) => name);

    // BUG-001: fieldNames will contain "file" not "files"
    expect(fieldNames).toContain('files');
  });

  /**
   * TEST-FRONT-03: Multiple files all use the same field name.
   * Also fails due to BUG-001.
   */
  it('should use consistent field name for multiple files', async () => {
    const capturedFormData: FormData[] = [];

    mockedAxios.post = vi.fn().mockImplementation((_url: string, formData: FormData) => {
      capturedFormData.push(formData);
      return Promise.resolve({ data: { uploaded: [], count: 0 } });
    });

    const files = [
      new File(['a'], 'a.txt', { type: 'text/plain' }),
      new File(['b'], 'b.txt', { type: 'text/plain' }),
    ];
    await uploadFiles(files);

    const entries = Array.from(capturedFormData[0].entries());
    const uniqueFieldNames = [...new Set(entries.map(([name]) => name))];

    // BUG-001: This will fail — the name is "file" not "files"
    expect(uniqueFieldNames).toEqual(['files']);
  });

  /**
   * TEST-FRONT-04: Correct API endpoint is called.
   * This test PASSES even on broken fixture.
   */
  it('should call the correct API endpoint', async () => {
    mockedAxios.post = vi.fn().mockResolvedValue({ data: { uploaded: [], count: 0 } });

    const file = new File(['x'], 'x.txt');
    await uploadFiles([file]);

    expect(mockedAxios.post).toHaveBeenCalledWith(
      expect.stringContaining('/api/upload'),
      expect.any(FormData),
      expect.objectContaining({ headers: { 'Content-Type': 'multipart/form-data' } })
    );
  });

  /**
   * TEST-FRONT-05: Returns structured response.
   * This test PASSES even on broken fixture (tests response parsing, not field name).
   */
  it('should return structured UploadResponse', async () => {
    const mockResponse = {
      uploaded: [{ filename: 'test.txt', size_bytes: 7, status: 'accepted' }],
      count: 1,
    };
    mockedAxios.post = vi.fn().mockResolvedValue({ data: mockResponse });

    const file = new File(['content'], 'test.txt');
    const result = await uploadFiles([file]);

    expect(result.count).toBe(1);
    expect(result.uploaded).toHaveLength(1);
    expect(result.uploaded[0].status).toBe('accepted');
  });
});
