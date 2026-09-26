# Presentation Outline

## Slide 1 — Title
**TraceForge AI: ContractGuard**
From Bug Report to Verified Fix

## Slide 2 — The Problem
- API contract drift: frontend, backend, tests, and docs diverge
- Manual diagnosis: multi-layer code inspection, manual reconstruction
- Missing traceability: hard to demonstrate a complete fix
- Time cost: investigation takes hours; verification is informal

## Slide 3 — The Solution
- ContractGuard: structured investigation → approval → verification workflow
- Contract Evidence Graph: traceable relationships between requirements and test results
- IBM Bob IDE as the investigation and implementation engine
- Independent verification: the application tests the fix, not the AI

## Slide 4 — System Architecture
[Diagram showing frontend ↔ backend ↔ IBM Bob ↔ UploadLab workspace]

- Rust Axum backend: state machine, workspace isolation, verification engine
- React dashboard: investigation creation, findings review, approval, reports
- IBM Bob: document understanding, code investigation, subagents, implementation
- UploadLab: deliberately broken sample project with real defects

## Slide 5 — Contract Evidence Graph
[Screenshot of evidence graph in running application]
- Nodes: requirement, frontend finding, backend finding, test failure, root cause, fix, passing test
- Edges: only shown when supported by stored evidence
- Each node links to source evidence and artifacts

## Slide 6 — IBM Bob Workflow
1. ContractGuard generates investigation prompt
2. Bob reads OpenAPI spec + documentation
3. Bob uses explore subagents in parallel
4. Bob writes structured `diagnosis.json`
5. Developer reviews and approves fix plan
6. Bob implements corrections in isolated workspace
7. ContractGuard independently runs tests

## Slide 7 — Live Demonstration
[Switch to live demo]
- Create investigation → run baseline (see failures)
- Bob investigation → sync results (see findings)
- Approve fix → run verification (see passing tests)
- View evidence graph and report

## Slide 8 — Results
- Real baseline test failures reproduced
- Two root causes identified with source references
- Fix approved by developer
- Independent verification confirms correction
- Evidence graph with full traceability

## Slide 9 — Measured Impact
- Baseline test execution captured
- Verification test execution captured
- Workflow timing recorded (created_at → completed_at)
- Manual baseline available for comparison (if recorded)

## Slide 10 — Future Development
- Support for arbitrary repositories (sandboxed execution)
- CI/CD integration: automatic investigation on test regression
- Team collaboration: shared investigations, role-based approval
- Extended contract formats: gRPC, GraphQL, AsyncAPI
- Advanced analytics: trend detection, recurring root causes

## Notes
- Do not use estimated numbers — show actual recorded measurements
- Public repository URL: [to be added]
- Live demo URL: [to be added if deployed]
