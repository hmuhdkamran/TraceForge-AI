# Problem and Solution Statement

## The Problem

Modern full-stack applications suffer from **API contract drift** — the gradual divergence between documented API specifications, frontend implementations, backend handlers, automated tests, and technical documentation. When a bug emerges from this drift, developers face a fragmented, time-consuming investigation:

1. Manually inspect multiple layers of the codebase
2. Reconstruct intended vs. actual behavior
3. Reproduce the failure
4. Coordinate cross-component changes
5. Write regression tests
6. Verify the fix is complete

This investigation is error-prone and creates significant rework. Teams regularly fix the symptom rather than the documented root cause, and lack traceable evidence that a correction resolves the complete problem.

**Target users:** Full-stack developers, backend engineers, QA engineers, and technical leads on teams maintaining API-driven applications.

## The Solution: ContractGuard

TraceForge AI ContractGuard addresses the full API contract debugging workflow by coordinating:

### Evidence-Backed Investigation
ContractGuard creates a **Contract Evidence Graph** that traces relationships between documented requirements, source code findings, failing tests, root causes, approved corrections, and verified test results. This graph is the core innovation — it transforms a fragmented debugging session into a structured, traceable engineering workflow.

### IBM Bob Integration
Rather than simulating AI analysis, ContractGuard generates precise investigation prompts and hands off real workspace artifacts to IBM Bob IDE. Bob reads the actual project documentation (OpenAPI spec, README, acceptance criteria), investigates real source files, and produces structured diagnosis artifacts with verified source-code references. This is a genuine AI-assisted workflow with human approval as a required checkpoint.

### Human Approval Gate
All proposed corrections require explicit developer approval before implementation begins. The approved plan is hashed and immutable — any material changes require renewed approval.

### Independent Verification
After implementation, ContractGuard's Rust verification engine independently executes the regression test suite and captures before/after results. Verified status requires actual test execution — not self-assessment.

### Demonstrated Results
The UploadLab sample project contains two deliberate defects:
- **BUG-001:** Frontend sends wrong multipart field name (`file` instead of `files`)
- **BUG-002:** Backend only processes the first file, silently ignoring others

These bugs produce reproducible test failures that ContractGuard traces through the complete workflow to independently verified correction.

## Impact

ContractGuard replaces a manual, fragmented debugging workflow with a structured, evidence-backed, AI-assisted investigation. The result is faster root cause identification, higher confidence corrections, and a permanent traceable record of what was found, why, and how it was verified.

**Word count: ~330**
