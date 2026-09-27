# IBM Bob Usage Statement

## How We Used IBM Bob

IBM Bob 2.0 IDE is not a peripheral feature of TraceForge AI ContractGuard — it is the investigation and implementation engine that the entire workflow is built around. Every stage of the ContractGuard pipeline was designed and built with Bob as the active collaborator. The following describes precisely how each of Bob's capabilities was used.

### Plan Mode — Architecture and Specification Design

Plan mode was used throughout the design phase before any code was written. We described the ContractGuard investigation workflow to Bob and asked it to review the proposed state machine, identify unnecessary complexity, and recommend simpler implementations where the spec could be satisfied with fewer moving parts. Bob identified that an explicit enumerated status type — rather than derived status logic — would make state transitions auditable and prevent invalid intermediate states. This recommendation directly shaped the final Rust data model.

Bob also reviewed the UploadLab OpenAPI specification before the sample project was built. We described the intended multipart upload contract and asked Bob to confirm that the field names, response schema, error handling, and documented behavior were internally consistent and testable. Bob flagged an ambiguity in the partial-failure response that we resolved by adding an explicit rejection policy to the acceptance criteria.

### Agent Mode — Genuine Investigation

The core demonstration of ContractGuard uses IBM Bob in Agent mode to investigate the broken UploadLab project. ContractGuard generates a structured investigation prompt at `/api/v1/investigations/:id/bob/investigation-prompt` that instructs Bob to read the actual workspace files in a specific order: OpenAPI spec, acceptance criteria, architecture documentation, frontend API client, backend upload handler, and existing test suite. Bob investigates the real source files and produces a validated `diagnosis.json` artifact with verified line-number references, a `fix_plan.md`, and an `investigation_summary.md`. These artifacts are imported into ContractGuard through the sync endpoint and validated against their published JSON schemas.

### Explore Subagents — Parallel Investigation

The investigation prompt requests three parallel Explore subagents to investigate independent layers of the codebase simultaneously:

- **Subagent A (Frontend):** Inspects the React upload component and API client, tracing the wrong multipart field name `file` back to the specific line where `FormData.append` is called with an incorrect key.
- **Subagent B (Backend):** Inspects the Rust Axum upload handler, identifying the single-call `next_field()` pattern that silently ignores all files after the first.
- **Subagent C (Contract):** Reads the OpenAPI spec and acceptance criteria, identifying which documented behaviors lack corresponding test coverage and which test assertions directly contradict the current implementation.

The primary Bob agent collects the subagent findings, eliminates duplicates, reconciles any conflicting source references, and produces the final synthesized diagnosis with root causes linked to actual baseline test failures.

### Document Understanding

Before investigating any source code, Bob reads and interprets four documentation files in the isolated workspace: `openapi.yaml` (the authoritative API contract), `docs/upload_api.md` (detailed endpoint documentation), `docs/acceptance_criteria.md` (testable behavioral requirements), and `docs/architecture.md` (system structure). This document understanding step grounds the entire investigation in the stated requirements rather than in the current broken behavior.

### Implementation Assistance

After the developer approves the fix plan in the ContractGuard dashboard, Bob receives an implementation prompt referencing the approved `plan_hash`. Bob reads the approved diagnosis artifacts, implements the minimal corrections for both defects — correcting the frontend field name and replacing the single-call backend with a complete field iterator — adds regression tests for multi-file processing and field name validation, runs `cargo test` in the isolated workspace, and writes `implementation_summary.md` and `changed_files.json` artifacts for ContractGuard's verification engine to import and validate.

### Regression Test Generation

Bob adds tests to the UploadLab test suite that directly verify the corrected behaviors: correct multipart field naming, full multi-file processing, and accurate file-count responses. ContractGuard's independent Rust verification engine then re-executes the complete test suite and records pass/fail status for each test. Bob's self-assessment of correctness does not determine the verified status shown in the dashboard — only the independently recorded test execution does.

### Code Review

Bob produces a `review.md` artifact covering correctness against documented requirements, test coverage gaps, security implications of the file-handling changes, any modifications outside the approved scope, and remaining risks. The review is stored as a submission artifact and displayed in the final investigation report alongside the independently executed test results.

## Technologies Used

- IBM Bob 2.0 IDE (Plan mode and Agent mode)
- Bob Explore subagents for parallel independent investigation
- Bob's document understanding capability for specification-grounded analysis
- Structured local artifact handoff via isolated workspace files

**Word count: ~500**
