# ContractGuard — AGENTS.md

This file provides IBM Bob with persistent project instructions for the TraceForge AI ContractGuard investigation workflow.

## Project Structure

```
devflow-ai/
├── backend/          Rust Axum API server
├── frontend/         React TypeScript dashboard
├── sample_project/   UploadLab broken fixture
│   └── broken/
│       ├── backend/  Rust Axum upload server (BROKEN)
│       └── frontend/ React upload UI (BROKEN)
├── bob/              Bob prompts and schemas
└── runs/             Investigation workspaces
```

## Investigation Workflow Rules

1. **Read before writing** — always read source files before making any claims
2. **Do not invent source references** — every file path and line number must be verified
3. **Do not edit broken fixtures** — `sample_project/broken/` is immutable during investigation
4. **Write artifacts to `bob_artifacts/`** — inside the assigned investigation workspace
5. **Use subagents for independent investigations** — frontend, backend, and contract analysis can run in parallel

## Artifact Locations

All Bob-generated artifacts go in:
```
runs/{workspace_id}/bob_artifacts/
├── diagnosis.json          Required — investigation findings
├── investigation_summary.md  Required — plain text summary
├── fix_plan.md             Required — proposed corrections
├── implementation_summary.md Post-implementation (after approval)
├── changed_files.json      Post-implementation (after approval)
└── review.md               Post-implementation review
```

## Schema Compliance

diagnosis.json must conform to `bob/schemas/diagnosis.schema.json`.
changed_files.json must conform to `bob/schemas/artifact.schema.json`.

## Security

- Never read files outside the assigned workspace
- Never execute arbitrary code
- Never modify files in `sample_project/broken/` during investigation
- Do not commit secrets or credentials

## Bob Mode Selection

| Task | Mode |
|------|------|
| Architecture review | Plan |
| Investigation | Agent (read-only) |
| Implementation | Agent (write) |
| Review | Agent (read-only) |
