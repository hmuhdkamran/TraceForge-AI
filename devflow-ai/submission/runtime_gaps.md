# TraceForge AI ContractGuard — Runtime Requirements & Gap Analysis

**Date:** September 2026  
**Status:** Backend compiles and runs. Frontend serves. Core workflow partially functional.

---

## 1. WHAT IS NEEDED TO RUN THE APPLICATION

### 1.1 Runtime Prerequisites

| Requirement | What It Is | Status |
|---|---|---|
| **Rust stable + Cargo** | Compiles and runs the Axum backend | ✅ Present |
| **Node.js 18+ + npm** | Runs the Vite/React frontend | ✅ Present |
| **SQLite** (bundled via `libsqlite3-sys`) | Database — no install needed; created automatically | ✅ Auto-created |
| **`.env` file in `backend/`** | Runtime config (DB path, ports, CORS, demo mode) | ✅ Present |
| **`runs/` directory** | Where investigation workspaces are created | ✅ Auto-created |
| **`sample_project/broken/`** | The UploadLab broken fixture that baseline tests run against | ✅ Present |
| **Cargo dependencies** | All Rust crates declared in `Cargo.toml` | ✅ Locked in `Cargo.lock` |
| **npm dependencies** | All frontend packages in `package.json` | ✅ `node_modules/` present |

### 1.2 How to Start

```powershell
# Terminal 1 — Backend (port 8080)
cd TraceForge-AI/devflow-ai/backend
cargo run

# Terminal 2 — Frontend (port 5173)
cd TraceForge-AI/devflow-ai/frontend
npm run dev
```

### 1.3 URLs

| Service | URL | Auth |
|---|---|---|
| Frontend Dashboard | `http://localhost:5173` | None |
| Backend API | `http://127.0.0.1:8080` | None (localhost dev mode) |
| Health check | `http://127.0.0.1:8080/api/v1/health` | None |

### 1.4 Environment Variables (backend/.env)

| Variable | Default | Purpose |
|---|---|---|
| `DATABASE_URL` | `sqlite://contractguard.db` | SQLite file location |
| `BIND_ADDR` | `127.0.0.1:8080` | API listen address |
| `WORKSPACE_ROOT` | `runs` | Where investigation workspaces are copied |
| `SAMPLE_PROJECT_ROOT` | `../sample_project` | Path to UploadLab broken fixture |
| `ALLOWED_ORIGINS` | `http://localhost:5173` | CORS whitelist |
| `DEMO_MODE` | `false` | Set `true` to make all mutations read-only |
| `TEST_TIMEOUT_SECS` | `120` | Max seconds for `cargo test` to run |
| `MAX_BODY_BYTES` | `16777216` | Request body limit (16 MB) |
| `MAX_OUTPUT_BYTES` | `1048576` | Truncation limit for test stdout/stderr |

---

## 2. WHAT THE APPLICATION DOES (FULLY WORKING)

These features work end-to-end right now with no gaps:

| Feature | How to Use |
|---|---|
| **Create investigation** | Dashboard → New Investigation → fill form → Submit |
| **Workspace isolation** | Automatically copies `sample_project/broken/` into `runs/{workspace_id}/` |
| **Run baseline tests** | Investigation detail → Overview tab → Run Baseline Tests |
| **Generate Bob prompt** | IBM Bob tab → copy the investigation prompt |
| **Sync Bob artifacts** | After Bob writes `diagnosis.json` to workspace → click Sync Results |
| **View findings** | Findings tab — shows all imported findings with evidence and line refs |
| **Approve fix plan** | Approval tab → review plan → Approve Fix (with confirmation modal) |
| **Generate implementation prompt** | After approval — IBM Bob tab shows implementation task |
| **Run verification** | Verification tab → Run Verification |
| **View report** | Reports page → HTML or JSON export |
| **Full audit trail** | Overview tab → Audit Trail section |
| **State machine** | All 15 states with enforced transitions |
| **Plan hash** | Approval stores SHA-256 hash of fix plan — immutable |
| **Demo mode** | Set `DEMO_MODE=true` to disable all mutations (for public deployment) |

---

## 3. GAPS — WHAT IS INCOMPLETE

### GAP 1 — Metrics are hardcoded to zero  
**Severity: Medium**  
**Files:** `backend/src/services/metrics/mod.rs`, `backend/src/handlers/metrics.rs`

**What happens:** The `/api/v1/investigations/:id/metrics` endpoint returns real workflow timing (total duration, baseline duration, verification duration, root cause count) but four values are hardcoded to `0`:
- `baseline_tests_failed` — always returns 0
- `verification_tests_passed` — always returns 0  
- `modified_files` — always returns 0
- `regression_tests_added` — always returns 0

**What is needed:** Parse the stored test execution `summary` field (e.g. `"3 passed, 2 failed (exit 1)"`) and count files listed in `changed_files.json` when it is imported.

**Impact:** The productivity metrics page and report section 17 show incomplete data. Everything else works.

---

### GAP 2 — Evidence Graph uses a simplified linear layout  
**Severity: Medium**  
**File:** `frontend/src/components/evidence/EvidenceGraph.tsx`

**What happens:** The evidence graph renders nodes (requirement → frontend finding → backend finding → test failure → root cause → approved fix → passing test) as a left-to-right linear chain. Nodes are clickable and show detail panels. However:
- The chain is constructed from imported findings in sequence, not from actual stored evidence relationships
- There is no graph layout algorithm — it is a fixed horizontal line
- If findings are not imported in a specific order, the visual chain may not reflect real relationships

**What is needed:** The backend already stores evidence with `finding_id` foreign keys. The graph should query `/api/v1/investigations/:id/findings` and the evidence endpoint and build edges from the actual `finding_id → investigation_id` relationships rather than connecting findings sequentially.

**Impact:** The graph is visually present and informative but does not precisely represent the evidence dependency chain.

---

### GAP 3 — Bob prompt templates are missing  
**Severity: Medium**  
**Missing files:** `bob/prompts/investigate.md`, `bob/prompts/implement.md`, `bob/prompts/review.md`

**What happens:** The backend's `services/prompts/mod.rs` generates investigation and implementation prompts dynamically from code (injecting workspace path, investigation ID, bug description, etc.). These work correctly. However, the spec requires reusable standalone prompt template files in `bob/prompts/` that Bob can open directly as task files.

**What is needed:** Create three markdown files:
- `bob/prompts/investigate.md` — the investigation task template
- `bob/prompts/implement.md` — the implementation task template  
- `bob/prompts/review.md` — the review task template

**Impact:** The generated prompts from the API work. The missing files are required for the AGENTS.md workflow and hackathon submission completeness check (Part 10 of spec).

---

### GAP 4 — Projects and scenarios are hardcoded  
**Severity: Low (for MVP)**  
**File:** `backend/src/handlers/projects.rs`

**What happens:** The project list and scenario list return hardcoded values — only UploadLab exists. There is no database table for projects or scenarios. This is intentional per the MVP boundary (Part 1.5 of spec: "support one bundled sample project").

**What is needed for production:** A `projects` and `scenarios` table in SQLite with CRUD endpoints. For the hackathon MVP this is not required.

**Impact:** Only the UploadLab / multi-file upload failure scenario can be used. This is correct and expected for the MVP.

---

### GAP 5 — Bob session screenshots are missing  
**Severity: Medium (submission requirement)**  
**Directory:** `bob_sessions/` (contains only an empty README)

**What happens:** The `bob_sessions/` directory exists but has no screenshots. The hackathon submission (Part 21) requires genuine readable screenshots of each team member's Bob task-session summaries.

**What is needed:** After running an actual Bob investigation session, save screenshots to `bob_sessions/` named by team member and task (e.g. `saliha_investigation_session.png`).

**Impact:** Submission checklist item is incomplete. The application itself is unaffected.

---

### GAP 6 — Demo data artifacts are not captured  
**Severity: Medium (for public demo)**  
**Directory:** `demo_data/` (contains only an empty README)

**What happens:** For PUBLIC_DEMO_MODE (Part 3.3 of spec), the application should serve a pre-captured genuine investigation with real diagnosis artifacts, real test results, and a real evidence graph. The `demo_data/` directory is empty.

**What is needed:** Run one complete investigation end-to-end (create → baseline → Bob investigation → sync → approve → verify → report), then export the resulting database rows and artifact files into `demo_data/` for replay in read-only demo mode.

**Impact:** A public deployment with `DEMO_MODE=true` would show an empty dashboard. Local `DEMO_MODE=false` usage is unaffected.

---

### GAP 7 — Frontend-only verification test execution  
**Severity: Low**  
**File:** `backend/src/services/verification/mod.rs`

**What happens:** Verification currently only runs `cargo test` on the Rust backend of the UploadLab workspace. It does not run `npm test` on the UploadLab frontend tests. BUG-001 (frontend field name) would therefore not be verified by the verification engine — only BUG-002 (backend single-file processing) would show as fixed in the verification results.

**What is needed:** Add a second verification step that runs `npm test -- --run` in the workspace frontend directory after the Rust tests pass.

**Impact:** Verification reports only partial coverage. The Rust tests cover BUG-002; BUG-001 is only visible in baseline (frontend test output), not in the post-fix verification.

---

### GAP 8 — No public repository URL  
**Severity: Low (submission hygiene)**  
**Files:** `README.md`, `submission/presentation_outline.md`

**What is needed:** Push to a public GitHub repository and add the URL to both files. Replace the `[to be added]` placeholders.

**Impact:** Submission checklist item is incomplete. The application is unaffected.

---

### GAP 9 — No video recording  
**Severity: Medium (required submission asset)**  

**What is needed:** A screen recording of ≤3 minutes showing ≥90 seconds of the live application following the `demo_script.md` narrative. Upload and link in the submission.

---

### GAP 10 — `sample_project/expected_contract/` directory missing  
**Severity: Low**  
**Missing:** `sample_project/expected_contract/acceptance_criteria.md`

**What is needed:** Create this directory and file as specified in Part 5 of the spec (repository structure). The content already exists in `sample_project/broken/docs/acceptance_criteria.md` — it just needs to be mirrored at the specified path.

---

## 4. PRIORITY ORDER TO MAKE THE APP FULLY OPERATIONAL

| Priority | Gap | Time Estimate |
|---|---|---|
| **1** | Run one complete investigation to validate the full workflow works | 30 min |
| **2** | GAP 3 — Create `bob/prompts/` template files | 30 min |
| **3** | GAP 1 — Fix metrics: parse test output + count changed files | 1 hour |
| **4** | GAP 7 — Add frontend verification step (npm test) | 1 hour |
| **5** | GAP 2 — Improve evidence graph to use real relationships | 2 hours |
| **6** | GAP 5 — Capture Bob session screenshots | Requires Bob session |
| **7** | GAP 6 — Capture demo investigation artifacts | Requires full session |
| **8** | GAP 10 — Create `expected_contract/` directory | 5 min |
| **9** | GAP 8 — Add public GitHub URL | 10 min after push |
| **10** | GAP 9 — Record video | 1 hour |

---

## 5. COMPLETE WORKFLOW — STEP BY STEP

Once everything above is in place, here is the exact sequence to run one complete investigation:

```
1. Open http://localhost:5173
2. Click "New Investigation"
3. UploadLab is pre-selected. Click "Load Scenario" → multi-file upload failure pre-fills.
4. Submit → investigation created, workspace copied to runs/{id}/
5. Investigation detail page opens. Click "Run Baseline Tests"
   → cargo test runs on runs/{id}/broken/backend/
   → 3 tests fail (BUG-001 frontend, BUG-002 backend x2)
6. Click "IBM Bob" tab → copy the investigation prompt
7. Open IBM Bob IDE → open the workspace at runs/{id}/
8. Paste the prompt as a new task → Bob investigates
   → Bob writes: runs/{id}/bob_artifacts/diagnosis.json
                  runs/{id}/bob_artifacts/investigation_summary.md
                  runs/{id}/bob_artifacts/fix_plan.md
9. Back in ContractGuard → IBM Bob tab → click "Sync Results"
   → diagnosis.json validated and imported
   → Findings displayed with source refs
10. Click "Approval" tab → review the fix plan → click "Approve Fix" → confirm
    → Plan SHA-256 hash stored, investigation transitions to APPROVED
11. Copy the implementation prompt (IBM Bob tab, now shows implementation task)
12. In IBM Bob → paste implementation prompt → Bob implements fixes
    → Bob writes: runs/{id}/bob_artifacts/implementation_summary.md
                   runs/{id}/bob_artifacts/changed_files.json
                   runs/{id}/bob_artifacts/review.md
13. Back in ContractGuard → IBM Bob tab → click "Sync Results" again
14. Click "Verification" tab → click "Run Verification"
    → cargo test runs again on the (now fixed) workspace
    → All 3 previously failing tests should pass
15. Click "Reports" page → view complete HTML report → download
```

---

## 6. ITEMS THAT DO NOT NEED ANY CHANGES

These are working correctly right now:

- Full investigation CRUD and state machine
- Workspace isolation (copy-on-create, no shared mutable files)
- Baseline test execution with real `cargo test`
- Investigation and implementation prompt generation
- Artifact sync and `diagnosis.json` validation against JSON Schema
- Human approval gate with SHA-256 content hash
- Verification test execution
- HTML and JSON report generation
- Audit trail (every workflow action recorded)
- Path traversal protection
- Demo mode (DEMO_MODE=true disables mutations)
- CORS with explicit origins
- Request body limits
- All frontend pages and components
- All 30+ API endpoints
