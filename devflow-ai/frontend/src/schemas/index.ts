import { z } from 'zod';

export const createInvestigationSchema = z.object({
  title: z.string().min(3, 'Title must be at least 3 characters').max(200),
  description: z.string().min(10, 'Description must be at least 10 characters').max(5000),
  project_id: z.string().min(1, 'Project is required'),
  scenario_id: z.string().min(1, 'Scenario is required'),
  expected_behavior: z.string().max(2000).optional(),
  observed_behavior: z.string().max(2000).optional(),
  reproduction_steps: z.string().max(2000).optional(),
});

export type CreateInvestigationFormData = z.infer<typeof createInvestigationSchema>;

export const approvalSchema = z.object({
  decision: z.enum(['approved', 'rejected']),
  comment: z.string().max(1000).optional(),
});

export type ApprovalFormData = z.infer<typeof approvalSchema>;

export const manualBaselineSchema = z.object({
  scenario_id: z.string().min(1),
  start_time: z.string().min(1),
  finish_time: z.string().optional(),
  active_minutes: z.number().min(0).optional(),
  completed_steps: z.array(z.string()).optional(),
  final_test_status: z.string().optional(),
});

export type ManualBaselineFormData = z.infer<typeof manualBaselineSchema>;
