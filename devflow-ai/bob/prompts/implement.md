# IBM Bob Implementation Task Template

The developer has reviewed and approved the investigation fix plan.
Your task is to implement the authorized code modifications in the isolated workspace.

## Objectives
1. Read the approved `bob_artifacts/diagnosis.json` and `bob_artifacts/fix_plan.md`.
2. Verify the plan hash matches the approved plan.
3. Correct the frontend multipart field name contract mismatch in `broken/frontend/src/api.ts`.
4. Correct the backend multiple-file processing loop in `broken/backend/src/main.rs`.
5. Execute regression tests:
   - Backend: `cargo test` in `broken/backend/`
   - Frontend: `npm test -- --run` in `broken/frontend/`
6. Verify all targeted regression tests pass.
7. Generate implementation artifacts in `bob_artifacts/`.

## Rules
- Scope boundary: Only make changes approved in the fix plan.
- Documentation integrity: Do not change acceptance criteria or OpenAPI spec to match broken code.
- Test integrity: Do not weaken test assertions to force a pass.

## Required Artifacts
1. `implementation_summary.md` — Detailed summary of modifications made.
2. `changed_files.json` — Conforming to `bob/schemas/artifact.schema.json`.
3. `review.md` — Self-review covering correctness, test coverage, and remaining risks.
