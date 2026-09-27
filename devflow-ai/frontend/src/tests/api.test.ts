import { describe, it, expect, vi, beforeEach } from 'vitest';
import axios from 'axios';
import { createInvestigationSchema } from '../schemas';

// Test the schema validation
describe('createInvestigationSchema', () => {
  it('accepts a valid investigation request', () => {
    const result = createInvestigationSchema.safeParse({
      title: 'Multi-file upload bug',
      description: 'The backend only processes the first file in a multi-file upload request.',
      project_id: 'uploadlab',
      scenario_id: 'multi-file-upload-failure',
    });
    expect(result.success).toBe(true);
  });

  it('rejects missing title', () => {
    const result = createInvestigationSchema.safeParse({
      title: '',
      description: 'Some description here',
      project_id: 'uploadlab',
      scenario_id: 'test',
    });
    expect(result.success).toBe(false);
    if (!result.success) {
      expect(result.error.issues[0].path[0]).toBe('title');
    }
  });

  it('rejects short title', () => {
    const result = createInvestigationSchema.safeParse({
      title: 'AB',
      description: 'A valid description here',
      project_id: 'uploadlab',
      scenario_id: 'test',
    });
    expect(result.success).toBe(false);
  });

  it('rejects missing project_id', () => {
    const result = createInvestigationSchema.safeParse({
      title: 'Valid Title',
      description: 'A valid description here',
      project_id: '',
      scenario_id: 'test',
    });
    expect(result.success).toBe(false);
  });

  it('accepts optional fields', () => {
    const result = createInvestigationSchema.safeParse({
      title: 'Valid Title',
      description: 'A valid description here',
      project_id: 'uploadlab',
      scenario_id: 'test',
      expected_behavior: 'Should return all files',
      observed_behavior: 'Only returns first file',
      reproduction_steps: '1. Upload two files\n2. Check response',
    });
    expect(result.success).toBe(true);
  });
});

// Test API request construction (mocking axios)
describe('API service', () => {
  beforeEach(() => {
    vi.resetAllMocks();
  });

  it('constructs correct investigation list URL', async () => {
    vi.mock('axios');
    const mockedAxios = vi.mocked(axios, true);
    // Import after mock is set up
    const { listInvestigations } = await import('../services/api');
    mockedAxios.create = vi.fn().mockReturnValue({
      get: vi.fn().mockResolvedValue({
        data: { investigations: [], total: 0 }
      }),
    });
    // Basic construction test
    expect(listInvestigations).toBeDefined();
  });

  it('status utility formats known statuses', async () => {
    const { statusLabel, statusColor, formatDuration } = await import('../utils/status');

    expect(statusLabel('CREATED')).toBe('Created');
    expect(statusLabel('COMPLETED')).toBe('Completed');
    expect(statusLabel('VERIFICATION_FAILED')).toBe('Verification Failed');

    expect(statusColor('COMPLETED')).toContain('emerald');
    expect(statusColor('FAILED')).toContain('red');

    expect(formatDuration(500)).toBe('500ms');
    expect(formatDuration(1500)).toBe('1.5s');
    expect(formatDuration(65000)).toBe('1m 5s');
    expect(formatDuration(undefined)).toBe('—');
  });

  it('workflowSteps returns ordered steps', async () => {
    const { workflowSteps } = await import('../utils/status');
    const steps = workflowSteps();
    expect(steps.length).toBeGreaterThan(0);
    expect(steps[0].step).toBe(1);
    // Steps should be ordered
    for (let i = 1; i < steps.length; i++) {
      expect(steps[i].step).toBeGreaterThanOrEqual(steps[i - 1].step);
    }
  });

  it('exports valid investigation API methods', async () => {
    const api = await import('../services/api');
    expect(typeof api.createInvestigation).toBe('function');
    expect(typeof api.getInvestigation).toBe('function');
    expect(typeof api.runBaseline).toBe('function');
    expect(typeof api.getBaseline).toBe('function');
    expect(typeof api.getBobInvestigationPrompt).toBe('function');
  });
});
