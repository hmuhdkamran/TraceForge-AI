# Submission Checklist
<!-- Audited against TraceForge AI Prompt.txt — all items verified against actual codebase -->

## Application

- [x] Dashboard starts successfully — React app at `http://localhost:5173` (Vite + React + Tailwind)
- [x] Rust API starts successfully — Axum backend at `http://127.0.0.1:8080`
- [x] Health endpoint responds: `GET /api/v1/health` — implemented in `handlers/health.rs`
- [x] Broken UploadLab fixture compiles and runs tests — `sample_project/broken/backend` + `frontend` both present with own `Cargo.toml` / `package.json`

## Investigation Workflow

- [x] Create investigation via dashboard — `POST /api/v1/investigations`, `NewInvestigationPage.tsx`
- [x] Workspace created automatically — `WorkspaceService` in `services/workspace/mod.rs`
- [x] Investigation prompt generated correctly — `GET /api/v1/investigations/:id/bob/investigation-prompt`
- [x] Baseline tests execute and capture failures — `POST /api/v1/investigations/:id/baseline`, `VerificationService`
- [x] Bob investigation artifacts can be synced — `POST /api/v1/investigations/:id/artifacts/sync`
- [x] Findings display with source references — `FindingsPanel.tsx`, `GET /api/v1/investigations/:id/findings`
- [x] Evidence graph renders — `EvidenceGraph.tsx`, `EvidencePage.tsx`
- [x] Fix plan approval requires confirmation — `ApprovalPanel.tsx` with confirmation dialog
- [x] Approved plan stored with content hash — `plan_hash` field in `Approval` model, `handlers/approvals.rs`
- [x] Implementation prompt generated after approval — `GET /api/v1/investigations/:id/bob/implementation-prompt`
- [x] Verification tests execute independently — `VerificationService` uses Tokio process, predefined command allowlist
- [x] Verified status requires actual test passage — exit code checked; `VERIFIED` state only set on exit 0
- [x] Report generates with HTML and JSON export — `GET /api/v1/investigations/:id/report.html` and `report.json`

## Workflow State Machine (Part 7)

- [x] All 15 states implemented — `InvestigationStatus` enum in `models/investigation.rs`
- [x] Invalid transitions produce errors — `can_transition_to()` enforced throughout services
- [x] APPROVED requires approval record — enforced in `handlers/approvals.rs`
- [x] VERIFIED requires independently executed tests — exit code gating in `VerificationService`
- [x] Audit log records all meaningful transitions — `AuditRepo::record()` called at every state change

## Sample Project

- [x] BUG-001: Frontend field name test fails on broken fixture — `api.test.ts` in `sample_project/broken/frontend`; expected failure: `should use field name "files"`
- [x] BUG-002: Multi-file backend test fails on broken fixture — `main.rs` tests; expected failures: `test_upload_two_valid_files_bug002`, `test_no_silent_omission_bug002`
- [x] OpenAPI spec present and accurate — `sample_project/broken/openapi.yaml`
- [x] Acceptance criteria documented — `sample_project/broken/docs/acceptance_criteria.md`
- [x] Tests named and described in README — `sample_project/broken/README.md` lists all expected failures

## Code Quality

- [ ] `cargo test` passes in backend — **NEEDS VERIFICATION** (run `cd backend && cargo test`)
- [ ] `npm test` passes in frontend — **NEEDS VERIFICATION** (run `cd frontend && npm test`)
- [x] TypeScript strict mode enabled — `"strict": true` in `frontend/tsconfig.json`
- [x] No hardcoded secrets — all config via env vars in `config.rs`; `.env.example` uses placeholders only
- [x] Error handling for all failure modes — `AppError` enum + `thiserror` in `errors.rs`; all 15 failure modes from Part 15 handled

## Security

- [x] Path traversal protection tested — `PathGuard::validate_workspace_path()` with 3 unit tests in `security/path_guard.rs`
- [x] No arbitrary command execution — predefined allowlist: `CMD_RUST_BACKEND_TESTS`, `CMD_FRONTEND_TESTS` only; no client-supplied args
- [x] Demo mode disables mutations — `DEMO_MODE` env var read in `config.rs`; checked in handlers
- [x] `.env.example` provided (no real secrets) — present at repo root with placeholder values only

## IBM Bob Integration Assets (Part 10)

- [x] `AGENTS.md` present — `bob/AGENTS.md` with workflow rules, artifact locations, mode selection table
- [x] `diagnosis.schema.json` present — `bob/schemas/diagnosis.schema.json`
- [x] `artifact.schema.json` present — `bob/schemas/artifact.schema.json`
- [ ] `bob/prompts/investigate.md` — **MISSING** (directory `bob/prompts/` does not exist)
- [ ] `bob/prompts/implement.md` — **MISSING**
- [ ] `bob/prompts/review.md` — **MISSING**

## Submission Assets (Part 21)

- [ ] Public repository URL — **NOT YET SET** (add to README.md and presentation_outline.md)
- [x] `problem_solution.md` (≤500 words) — present, ~500 words, balanced problem + solution
- [x] `bob_usage.md` (≤500 words) — present, ~500 words, all 8 required capabilities covered
- [x] `demo_script.md` — present with full 3-minute narrative
- [x] `presentation_outline.md` — present, 10 slides outlined
- [x] `TraceForge_AI_ContractGuard.pptx` — 12-slide deck, validated
- [ ] Bob session screenshots in `bob_sessions/` — **MISSING** (directory exists but contains only empty README)
- [ ] Video (≤3 minutes, ≥90s of live app) — **NOT YET RECORDED**
- [ ] Deployed demo URL (optional) — not yet deployed

## Documentation

- [x] Root `README.md` with startup instructions — present; Windows PowerShell + Linux/macOS commands
- [x] UploadLab README with known defects — `sample_project/broken/README.md`; both bugs listed with expected test names
- [x] API documentation (`openapi.yaml`) — `sample_project/broken/openapi.yaml`
- [x] Architecture documentation — `sample_project/broken/docs/architecture.md`
- [x] `docs/upload_api.md` — present
- [x] `docs/acceptance_criteria.md` — present

## Repository Structure (Part 5)

- [x] `backend/` — Rust Axum API, all specified modules present
- [x] `frontend/` — React TypeScript dashboard, all specified pages and components present
- [x] `sample_project/broken/` — UploadLab broken fixture
- [ ] `sample_project/expected_contract/acceptance_criteria.md` — **MISSING**
- [x] `bob/AGENTS.md` + `bob/schemas/` — present
- [ ] `bob/prompts/` — **MISSING** (investigate.md, implement.md, review.md)
- [x] `runs/.gitkeep` — present
- [x] `demo_data/` — present (README placeholder only; demo artifacts not yet captured)
- [x] `bob_sessions/` — directory present (screenshots needed)
- [x] `submission/` — all text assets present
- [x] `.github/workflows/ci.yml` — CI pipeline for backend, frontend, and both broken-fixture baselines
- [x] `Dockerfile` — present
- [x] `compose.yaml` — present
- [x] `.bobignore` — present
- [x] `.env.example` — present

## CI Pipeline (Part 18)

- [x] Rust formatting check (`cargo fmt`)
- [x] Clippy (`cargo clippy -D warnings`)
- [x] Rust unit + integration tests (`cargo test`)
- [x] TypeScript type check (`npm run typecheck`)
- [x] Frontend tests (`npm test`)
- [x] Frontend production build (`npm run build`)
- [x] Sample project baseline: broken tests verified to fail for correct reasons
- [x] Main CI does not fail due to intentionally broken sample

---

## Outstanding Items — Priority Order

| # | Item | Required by Spec | Effort |
|---|------|-----------------|--------|
| 1 | Run `cargo test` and confirm pass | Part 18 | Low |
| 2 | Run `npm test` and confirm pass | Part 18 | Low |
| 3 | Create `bob/prompts/investigate.md` | Part 10.2 | Medium |
| 4 | Create `bob/prompts/implement.md` | Part 10.7 | Medium |
| 5 | Create `bob/prompts/review.md` | Part 10.8 | Low |
| 6 | Create `sample_project/expected_contract/acceptance_criteria.md` | Part 5 | Low |
| 7 | Add Bob session screenshots to `bob_sessions/` | Part 21 | Requires Bob session |
| 8 | Capture demo investigation artifacts into `demo_data/` | Part 3.3 / Part 21 | Requires Bob session |
| 9 | Record video (≤3 min, ≥90s live app) | Part 21 | Medium |
| 10 | Push to public GitHub repository and add URL | Part 21 | Low |
| 11 | Deploy public read-only demo (optional) | Part 3.3 | Optional |
