# IBM Bob Investigation Task Template

You are acting as a principal software engineer and QA automation architect using IBM Bob.
Your mission is to perform a root-cause investigation on an API contract drift issue in an isolated workspace.

## Objectives
1. Read the bug intake report and expected acceptance criteria.
2. Read the authoritative OpenAPI specification (`broken/openapi.yaml`).
3. Investigate the frontend client implementation (`broken/frontend/src/api.ts` and `UploadForm.tsx`).
4. Investigate the backend endpoint implementation (`broken/backend/src/main.rs`).
5. Examine test suites and baseline failure logs.
6. Correlate defects across architectural boundaries.
7. Produce verified structured artifacts in `bob_artifacts/`.

## Investigation Rules
- **Read-Only**: Do NOT modify any files in `broken/` during the investigation phase.
- **Evidence-Backed**: Every finding MUST point to real, verified file paths and line numbers.
- **Schema-Compliant**: All output artifacts must validate against `bob/schemas/diagnosis.schema.json`.

## Required Artifacts
Write the following three artifacts to `bob_artifacts/`:
1. `diagnosis.json` — Machine-readable diagnosis adhering to the diagnosis schema.
2. `investigation_summary.md` — Narrative summary of what was inspected and discovered.
3. `fix_plan.md` — Minimal proposed fix plan to restore contract conformance.
