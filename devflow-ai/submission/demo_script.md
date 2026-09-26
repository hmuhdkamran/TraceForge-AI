# Demo Script

## Setup (before recording)

```powershell
# Start backend
cd backend
cargo run
```

```powershell
# Start frontend
cd frontend
npm run dev
```

Open `http://localhost:5173`

## Narrative (3 minutes)

### 0:00–0:20 — The Problem

"Modern applications suffer from API contract drift. When the frontend uses a different field name than what the backend expects, or when the backend silently ignores uploaded files, these bugs are hard to diagnose and even harder to confirm are fully fixed.

ContractGuard by TraceForge AI brings structure to this debugging workflow."

### 0:20–0:40 — Introduce ContractGuard

Show the dashboard — currently empty, no fabricated data.

"ContractGuard coordinates the complete journey from bug report to verified fix. The UploadLab sample project has two real bugs built in. Let me create an investigation."

Click **New Investigation** → UploadLab pre-selected → Scenario pre-loaded with the multi-file upload failure.

Show the form fields pre-populated. Click **Submit Investigation**.

### 0:40–1:00 — Baseline Failures

Show investigation detail page — workspace created automatically.

Click **Run Baseline Tests** → shows test execution in progress.

"ContractGuard runs the actual regression tests against the broken code. Two tests fail — one for the frontend field name, and several for the backend single-file issue."

Show the baseline test results with actual failure output.

### 1:00–1:30 — IBM Bob Investigation

Switch to the **IBM Bob** tab.

"ContractGuard generates a precise investigation prompt. I'll copy it and open the workspace in IBM Bob."

Copy the prompt. Show the workspace path.

"Bob reads the OpenAPI spec, inspects the frontend API client, and examines the Rust handler. It uses explore subagents to investigate frontend and backend in parallel."

Show Bob running the investigation (screen recording of Bob IDE).

After Bob completes: click **Sync Results** in ContractGuard.

### 1:30–2:00 — Findings and Approval

Switch to **Findings** tab — show actual findings with source file references.

Switch to **Evidence Graph** tab — show the connected evidence chain.

Switch to **Approval** tab — show the fix plan from Bob.

"I can inspect exactly what Bob found and what it proposes to fix. Nothing moves forward without my approval."

Click **Approve Fix** → confirmation dialog → confirm.

### 2:00–2:30 — Verification

Switch to **Verification** tab.

"With the fix approved, Bob implements the corrections. ContractGuard then independently runs the same tests again."

Click **Run Verification** → execution in progress.

Show results: previously failing tests now pass.

"The verification engine confirms the original regression tests pass. ContractGuard independently executed these — not Bob's self-assessment."

### 2:30–3:00 — Report

Switch to **Reports** page → open the investigation report.

"Here's the complete evidence-backed report: the original bug, the documented findings, the approved fix, and the independently verified test results."

Click **Download HTML** to show export capability.

Show the evidence graph one more time.

"From bug report to verified fix — with a traceable record of every decision."

## Key Points to Emphasize

1. Real test failures, not mocked data
2. Human approval gate before any changes
3. Independent verification — not AI self-assessment
4. Evidence graph with actual source references
5. Complete audit trail of all workflow events
