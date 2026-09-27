# Problem and Solution Statement

## The Problem

Modern full-stack applications are composed of multiple independent layers: a React frontend, a backend API, an OpenAPI specification, an automated test suite, and written documentation. In the early stages of a project these layers agree. As development continues, they drift. A backend handler silently processes only the first of several uploaded files. A frontend client sends a multipart field under the wrong name. A test is never written for the multi-file case. Documentation is updated for one release but not the next. This gradual divergence is **API contract drift** — one of the most common and expensive categories of production defect.

When a bug report arrives that originates from contract drift, the investigation is disproportionately painful. There is no single file to blame. A developer must read the OpenAPI specification, the frontend request-construction code, the backend parsing handler, the existing test suite, and relevant documentation — often across multiple files and directories — before they can form even a working hypothesis. Each step is performed without structure. Findings accumulate in a developer's head or in informal chat threads. There is no persistent record connecting the original documented requirement to the final passing test.

The rework cost compounds over time. Teams regularly fix the visible symptom without tracing it back to the documented root cause, so the same class of drift resurfaces. When a fix is eventually deployed, there is no traceable evidence that the correction resolved the complete stated requirement — only that one observable failure stopped occurring.

**Target users:** Full-stack developers, backend engineers, QA engineers, and technical leads on teams where the frontend, backend, tests, and documentation are developed and modified independently.

## The Solution: TraceForge AI ContractGuard

ContractGuard replaces this informal, fragmented workflow with a structured, evidence-backed, AI-assisted investigation pipeline organized around a central artifact: the **Contract Evidence Graph**.

The graph is a persistent, inspectable record that connects every investigation finding to verifiable evidence. Each node represents a real entity — a documented API requirement, a frontend implementation detail, a backend handler behavior, a failing test, a confirmed root cause, an approved correction, or a verified passing test. Each edge between nodes is backed by a stored artifact or a recorded test execution. Nothing is asserted without evidence; missing links are displayed as gaps rather than hidden.

The workflow proceeds through explicit, auditable states. ContractGuard creates an isolated workspace from the broken project fixture and runs baseline tests to capture genuine failures. It then generates a structured investigation prompt for IBM Bob IDE. Bob reads the actual project documentation — the OpenAPI specification, acceptance criteria, architecture notes — before inspecting any source code. Bob then runs three parallel Explore subagents: one investigating the frontend request construction, one examining the backend handler, and one reviewing the test suite against the contract. The synthesized findings, with verified line-number references, are written as structured `diagnosis.json` and `fix_plan.md` artifacts.

Before any code changes are made, the developer reviews the findings in the ContractGuard dashboard and explicitly approves the fix plan. The approved plan is content-hashed and immutable — any material deviation requires renewed approval. This human gate ensures that AI investigation informs, rather than replaces, engineering judgment.

After Bob implements the approved corrections in the isolated workspace, ContractGuard's Rust verification engine independently executes the regression test suite and records results. Verified status is determined by actual test passage, not by AI self-assessment.

The UploadLab sample project demonstrates the complete workflow end-to-end: two deliberate defects, genuine baseline failures, Bob-assisted root cause identification, human-approved correction, and independently verified passing tests — all connected in a single traceable investigation record.

**Word count: ~500**
