# IBM Bob Integration Directory

This directory contains prompt templates, JSON schemas, and agent instructions for IBM Bob's role in the ContractGuard workflow.

## Structure

```
bob/
├── prompts/
│   ├── investigate.md        Task template for Bob investigation phase
│   ├── implement.md          Task template for Bob implementation phase
│   └── review.md             Task template for Bob code review phase
├── schemas/
│   ├── diagnosis.schema.json Formal JSON schema for diagnosis.json
│   └── artifact.schema.json  Formal JSON schema for changed_files.json
├── AGENTS.md                 Persistent project guidelines for Bob
└── README.md                 Overview documentation (this file)
```

## Workflow Integration

1. **Investigation**: ContractGuard generates a task-specific prompt from `investigate.md` for Bob to inspect the isolated workspace.
2. **Handoff**: Bob produces `diagnosis.json`, `investigation_summary.md`, and `fix_plan.md` in `bob_artifacts/`.
3. **Synchronization**: ContractGuard validates the artifacts against `diagnosis.schema.json` and updates the evidence graph.
4. **Approval**: Developer inspects findings and approves the fix plan.
5. **Implementation**: Bob receives the implementation prompt derived from `implement.md` with the immutable plan hash.
6. **Verification**: ContractGuard independently verifies the fix using cargo and npm test suites.
