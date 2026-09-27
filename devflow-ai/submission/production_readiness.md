# TraceForge AI ContractGuard — Production Readiness Report

**Current state:** MVP demo-ready. Not production-deployable.  
**Score:** 4 / 10  
**Blockers before any public deployment:** 4 critical issues must be resolved first.

---

## CRITICAL — Must fix before any deployment

---

### CRIT-1 · No Authentication or Authorization

**Every single API endpoint is open with zero credentials required.**

| Where | What |
|---|---|
| `backend/src/routes/mod.rs` lines 21–56 | All 30+ routes registered with no auth middleware |
| `backend/src/security/mod.rs` | Only contains `path_guard`; no auth module exists |
| `frontend/src/services/api.ts` | No Bearer token, API key, or session cookie sent |

Any network-reachable user can create investigations, approve fix plans, trigger `cargo test` execution, and download all reports. In `DEMO_MODE=false` (the default) this means unauthenticated arbitrary command execution against the host filesystem.

**What to build:**
- Add a JWT middleware layer in `lib.rs` (tower + jsonwebtoken crates)
- Protect all mutation routes (`POST`, `PUT`) behind `require_auth` extractor
- Add read-only access for `GET` routes if public viewing is needed
- Store hashed API keys or user sessions in the SQLite `users` table
- Add `approver_id` enforcement: approval must come from authenticated user

---

### CRIT-2 · Production Docker Has `ALLOWED_ORIGINS=*`

**File:** `Dockerfile` line 60

```dockerfile
ENV ALLOWED_ORIGINS="*"   # ← this line in the Dockerfile
```

The Dockerfile hardcodes CORS to allow any origin. Any website on the internet can make credentialed cross-origin requests to the API from a visitor's browser.

**What to fix:**
```dockerfile
# Remove the ENV line entirely — let it come from compose/runtime
ENV ALLOWED_ORIGINS=""
```
Then pass the correct origin at runtime via `compose.yaml` or `docker run -e`.

---

### CRIT-3 · Evidence-to-Finding Association Is Broken

**File:** `backend/src/services/artifacts/mod.rs` line 244

```rust
.find(|_| true)   // ← always selects the first finding regardless of ID
```

When Bob's `diagnosis.json` links a piece of evidence to a specific finding by ID, the import service ignores the `finding_id` field and associates the evidence with whichever finding happens to be first in the list. Every investigation with more than one finding will have its evidence stored against the wrong finding. The evidence graph, findings panel, and report will show incorrect source-code references.

**What to fix:** Replace the `find(|_| true)` fallback with a proper ID lookup:
```rust
.find(|f| f.id == evidence_finding_id)
```
If no match, log a warning and skip rather than silently misassigning.

---

### CRIT-4 · No Rate Limiting

**File:** `backend/src/lib.rs` — no `ConcurrencyLimitLayer` or per-IP limiting

Baseline and verification handlers call `tokio::spawn` which executes `cargo test` as a subprocess. With no rate limiting, a single client can trigger hundreds of simultaneous `cargo test` processes, exhausting CPU, memory, and file descriptors.

**What to build:**
- Add `tower::limit::ConcurrencyLimitLayer` on the verification and baseline routes
- Add a per-investigation mutex (already partially designed) to prevent concurrent runs on the same workspace
- Add a global semaphore limiting total concurrent test executions

---

## HIGH — Fix before user-facing deployment

---

### HIGH-1 · Background Tasks Have No Error Recovery

**File:** `backend/src/handlers/investigations.rs` lines 98–113

```rust
tokio::spawn(async move {
    // if this panics or fails, the investigation is stuck
    // in BASELINE_RUNNING forever with no way out
});
```

If the spawned task fails, errors, or is dropped, the investigation's status never transitions away from `BASELINE_RUNNING`. There is no timeout, no retry, and no dead-letter mechanism. The investigation is permanently stuck.

**What to build:**
- Wrap `tokio::spawn` in a task tracker (tokio-tasks or JoinSet)
- Add a timeout matching `TEST_TIMEOUT_SECS` that transitions to `FAILED` on expiry
- Add a background sweeper that detects investigations stuck in `*_RUNNING` states older than the timeout

---

### HIGH-2 · Path Traversal Check Is String-Based, Not Canonical

**File:** `backend/src/security/path_guard.rs` lines 28–32

```rust
if !candidate_str.starts_with(base_str.as_ref()) {
```

This uses string prefix matching instead of canonicalized path comparison. On Windows, symlinks, junctions, and short path names (`C:\PROGRA~1`) can bypass a string `starts_with` check. If `workspace_root` itself is a symlink, the entire guard fails.

**What to fix:**
- Canonicalize **both** paths before comparison
- Guard the canonicalize call — if it fails, deny access (don't fall back)
- Add a startup check that verifies `workspace_root` is not itself a symlink

---

### HIGH-3 · Config Silently Accepts Invalid Values

**File:** `backend/src/config.rs` lines 43–55

Invalid environment variables are silently replaced with defaults. Setting `TEST_TIMEOUT_SECS=abc` results in a 120-second timeout with no warning. Setting `MAX_BODY_BYTES=0` accepts 0-byte bodies.

**What to fix:**
- Add a `Config::validate()` method called at startup
- Fail hard (`anyhow::bail!`) on clearly invalid values rather than silently defaulting
- Log all resolved config values at startup (excluding secrets)

---

### HIGH-4 · Silent Failures Throughout (`.unwrap_or_default()` Pattern)

**15+ occurrences across handlers and services**, for example:

| File | Line | Pattern | Risk |
|---|---|---|---|
| `handlers/investigations.rs` | 178 | `.unwrap_or_else(|_| "Fix plan not yet available.")` | File permission error looks like missing file |
| `handlers/approvals.rs` | 25, 62 | `.unwrap_or_default()` | Corrupted artifact treated as empty |
| `services/verification/mod.rs` | 85 | `.unwrap_or_else(...)` | Broken path returns wrong directory |
| `services/reporting/mod.rs` | 39–43 | `.unwrap_or_default()` chains | Failed serialization produces empty report |
| `handlers/artifacts.rs` | 36 | `let _ = transition_status(...)` | Status not updated; investigation stuck |

**What to fix:** Replace all `unwrap_or_default()` on `Result` types with explicit error propagation using `?` or a specific error variant. Reserve `unwrap_or_default()` only for truly infallible cases.

---

### HIGH-5 · Metrics Are Hardcoded to Zero

**File:** `backend/src/services/metrics/mod.rs` lines 61–62

```rust
baseline_tests_failed: 0,      // never populated
verification_tests_passed: 0,  // never populated
modified_files: 0,             // never populated
regression_tests_added: 0,     // never populated
```

The productivity metrics are the **primary measurable output** of the hackathon submission. All four of these values are always 0 regardless of what actually happened during the investigation.

**What to build:**
- Parse `summary` field from `TestExecution` rows (already stored as `"3 passed, 2 failed (exit 1)"`)
- Count `changed_files.json` array length when imported
- Count new test functions added by Bob's implementation

---

## MEDIUM — Required for production quality

---

### MED-1 · No Input Validation on Description Length / Content

**File:** `backend/src/handlers/investigations.rs`

Title is validated (min 3 chars) but description has no maximum length. A 100 MB description would pass validation and be stored in SQLite. No sanitization for control characters, null bytes, or script injection in text fields that are later rendered in HTML reports.

**What to fix:** Add `max_length` constraints on all text fields. HTML-escape all user-supplied text before rendering (partially done but not consistently).

---

### MED-2 · Verification Only Runs Backend Tests

**File:** `backend/src/services/verification/mod.rs`

Only `cargo test` is run on the Rust backend. BUG-001 (frontend field name mismatch) is tested in the UploadLab frontend test suite (`npm test`). The verification engine never runs the frontend tests — so BUG-001 fix is never independently confirmed.

**What to build:** Add a second verification command (`CMD_FRONTEND_TESTS`) that runs `npm test -- --run` in the workspace frontend directory and records a second `TestExecution` row for the same verification run.

---

### MED-3 · No Request Correlation IDs

**File:** `backend/src/lib.rs`

There are no request IDs injected into the trace context. When multiple requests overlap in logs it is impossible to associate log lines with specific requests or investigations.

**What to build:** Add a `TraceRequestId` middleware layer that generates a UUID per request and injects it into the tracing span and response headers.

---

### MED-4 · Frontend API Has No Timeout or Retry

**File:** `frontend/src/services/api.ts` line 18

Axios is initialized with no `timeout` option (defaults to infinite). A hung baseline test execution would leave the frontend waiting indefinitely with no feedback.

**What to fix:**
```typescript
const api = axios.create({
  baseURL: ...,
  timeout: 30000,   // 30s for regular requests
});
```
Add TanStack Query retry configuration for transient failures.

---

### MED-5 · No Soft Delete / Data Recovery

**File:** `backend/migrations/001_initial.sql`

All foreign keys use `ON DELETE CASCADE`. Deleting an investigation permanently cascades to findings, evidence, approvals, test executions, and audit events with no recovery path. There is no `deleted_at` column and no archive mechanism.

**What to build:** Add `deleted_at TIMESTAMP` to `investigations`. Implement soft-delete on the API. Add a DB-level check constraint or application guard preventing hard deletes.

---

### MED-6 · Evidence Graph Ignores Actual Relationships

**File:** `frontend/src/components/evidence/EvidenceGraph.tsx` line 63

The graph connects findings in sequential array order regardless of their actual stored evidence relationships. Two independent root causes are drawn as causally linked.

**What to build:** Fetch evidence from `/api/v1/investigations/:id/findings` and use the `finding_id` references in the evidence rows to build edges. Use a proper DAG layout (even a simple topological sort) rather than sequential position.

---

### MED-7 · Approval Has No Idempotency Guard

**File:** `backend/src/handlers/approvals.rs`

Approving an already-approved investigation inserts a second approval record. There is no guard preventing re-approval. The `plan_hash` immutability check only prevents changing the plan after approval — it does not prevent duplicate approvals.

**What to fix:** Check for existing `approved` decision before inserting. Return the existing approval if one already exists for the current plan hash.

---

### MED-8 · Frontend Uses `as any` in Critical Path

**File:** `frontend/src/pages/InvestigationDetailPage.tsx` lines 149, 153, 155–171 (8 occurrences)

The investigation detail page casts the investigation object to `any` before passing it to every tab component. Type errors in any of these components are invisible to the compiler.

**What to fix:** Update the `Investigation` TypeScript interface to match the actual API response shape. Remove all `as any` casts. Enable `noImplicitAny` in `tsconfig.json`.

---

### MED-9 · CI Has No Security Scanning

**File:** `.github/workflows/ci.yml`

The CI pipeline runs formatting, clippy, tests, and build — but no:
- `cargo audit` (known CVEs in dependencies)
- `cargo deny` (license compliance, advisories)
- Container image scanning (Trivy, Grype)
- SAST (semgrep, CodeQL)

**What to add:**
```yaml
- name: Security audit
  run: cargo install cargo-audit && cargo audit
```

---

## LOW — Polish and maintainability

---

### LOW-1 · Projects and Scenarios Are Hardcoded

**File:** `backend/src/handlers/projects.rs` lines 7–55

Only UploadLab exists. Adding any new project requires a code change and rebuild. Acceptable for MVP; needs a `projects` table and CRUD API for production.

---

### LOW-2 · State Machine Uses String Matching

**File:** `backend/src/services/investigation/mod.rs`

Status transitions use `&str` comparisons. If an invalid status string enters the database (e.g. via a migration or direct write), the application accepts it silently. Use the `InvestigationStatus` enum throughout.

---

### LOW-3 · Test Coverage Is Minimal for Services

Only `path_guard.rs` has unit tests. The verification service, artifacts service, reporting service, and metrics service have no tests. The handlers have no integration tests.

**What to build:** At minimum add unit tests for:
- `services/metrics/mod.rs` — test output parsing
- `services/artifacts/mod.rs` — diagnosis import validation
- `services/verification/mod.rs` — output truncation, summary parsing
- Handler integration tests using `axum::test`

---

### LOW-4 · Cargo.lock Not Pinned in CI

**File:** `.github/workflows/ci.yml` line 47

`cargo test` runs without `--locked`. Dependency resolution may differ from the committed `Cargo.lock` on CI, allowing supply chain drift.

**What to fix:** Add `--locked` to all cargo commands in CI:
```yaml
run: cargo test --locked
```

---

## Summary Table

| ID | Issue | Severity | File | Effort |
|---|---|---|---|---|
| CRIT-1 | No authentication | 🔴 Critical | routes/mod.rs + lib.rs | Large |
| CRIT-2 | CORS `*` in production Docker | 🔴 Critical | Dockerfile:60 | Trivial |
| CRIT-3 | Evidence-finding mapping broken | 🔴 Critical | artifacts/mod.rs:244 | Small |
| CRIT-4 | No rate limiting | 🔴 Critical | lib.rs | Medium |
| HIGH-1 | Background tasks unrecoverable | 🟠 High | investigations.rs:98 | Medium |
| HIGH-2 | Path traversal check string-based | 🟠 High | path_guard.rs:28 | Small |
| HIGH-3 | Config silently accepts invalid values | 🟠 High | config.rs | Small |
| HIGH-4 | `.unwrap_or_default()` throughout | 🟠 High | 15+ files | Medium |
| HIGH-5 | Metrics hardcoded to zero | 🟠 High | metrics/mod.rs:61 | Medium |
| MED-1 | No input length validation | 🟡 Medium | investigations.rs | Small |
| MED-2 | Verification skips frontend tests | 🟡 Medium | verification/mod.rs | Small |
| MED-3 | No request correlation IDs | 🟡 Medium | lib.rs | Small |
| MED-4 | Frontend API no timeout | 🟡 Medium | api.ts:18 | Trivial |
| MED-5 | No soft delete / data recovery | 🟡 Medium | migrations | Medium |
| MED-6 | Evidence graph sequential-only | 🟡 Medium | EvidenceGraph.tsx:63 | Medium |
| MED-7 | Approval not idempotent | 🟡 Medium | approvals.rs | Small |
| MED-8 | Frontend `as any` casts | 🟡 Medium | InvestigationDetailPage.tsx | Small |
| MED-9 | No security scanning in CI | 🟡 Medium | ci.yml | Small |
| LOW-1 | Hardcoded projects/scenarios | 🟢 Low | projects.rs | Large |
| LOW-2 | State machine string matching | 🟢 Low | investigation/mod.rs | Small |
| LOW-3 | Minimal test coverage | 🟢 Low | services/* | Large |
| LOW-4 | Cargo not locked in CI | 🟢 Low | ci.yml | Trivial |

---

## What Is Already Production-Quality

These parts of the codebase are well-built and require no changes for production:

- **Investigation state machine** — 15 states, enforced transitions, clear error messages
- **SQLx migrations** — versioned schema, runs at startup, no raw SQL strings
- **Workspace isolation** — each investigation gets its own copy; no shared mutable state
- **Approval plan hashing** — SHA-256 of fix plan stored and verified before implementation
- **HTML report escaping** — user content is HTML-escaped in the reporting service
- **Multi-stage Docker build** — builder + runtime image; non-root user; health check
- **DEMO_MODE flag** — disables all mutations; designed for public read-only deployment
- **Audit log** — every workflow event recorded with actor, timestamp, and details
- **Timeout enforcement** — `TEST_TIMEOUT_SECS` applied to all subprocess calls
- **Output size limits** — `MAX_OUTPUT_BYTES` prevents log flooding
- **CORS configuration** — explicit origin list in production (once CRIT-2 is fixed)
- **Bob prompt generation** — complete, structured, workspace-path-injected prompts
- **Artifact schema validation** — `diagnosis.json` validated against JSON Schema before import
- **Idempotent artifact sync** — SHA-256 deduplication prevents duplicate findings on re-sync
