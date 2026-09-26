use crate::models::investigation::Investigation;
use crate::config::Config;

pub struct PromptService;

impl PromptService {
    pub fn generate_investigation_prompt(investigation: &Investigation, config: &Config) -> String {
        let workspace_path = config.workspace_root
            .join(&investigation.workspace_id)
            .display()
            .to_string();

        format!(r#"# ContractGuard — Investigation Task

## Investigation ID
{id}

## Bug Report
**Title:** {title}

{description}

## Your Task

You are acting as a principal software engineer investigating API contract inconsistencies.

### Step 1 — Read the project documentation
- Open `{workspace}/broken/README.md`
- Open `{workspace}/broken/openapi.yaml`
- Open `{workspace}/broken/docs/upload_api.md`
- Open `{workspace}/broken/docs/acceptance_criteria.md`
- Open `{workspace}/broken/docs/architecture.md`

### Step 2 — Investigate the frontend
Use an **Explore subagent** to:
- Read `{workspace}/broken/frontend/src/components/UploadForm.tsx`
- Read the API client module
- Identify differences between the frontend multipart field name and the documented contract

### Step 3 — Investigate the backend
Use an **Explore subagent** to:
- Read `{workspace}/broken/backend/src/main.rs`
- Read all upload handler code
- Identify how files are processed and counted in the response

### Step 4 — Examine existing tests
- Read `{workspace}/broken/backend/tests/`
- Read `{workspace}/broken/frontend/src/tests/`
- Note which tests fail and why

### Step 5 — Synthesize findings
- Correlate frontend and backend bugs
- Identify confirmed root causes
- Propose a minimal fix that restores documented behavior

### Step 6 — Write structured artifacts
Write the following files to `{workspace}/bob_artifacts/`:

**`diagnosis.json`** — must conform to the diagnosis schema:
```json
{{
  "schema_version": 1,
  "investigation_id": "{id}",
  "project_id": "{project_id}",
  "summary": "...",
  "findings": [
    {{
      "id": "FINDING-001",
      "finding_type": "contract_mismatch",
      "title": "...",
      "description": "...",
      "expected_behavior": "...",
      "observed_behavior": "...",
      "confidence": "high",
      "source_references": [
        {{
          "relative_path": "broken/frontend/src/...",
          "start_line": 10,
          "end_line": 20,
          "description": "..."
        }}
      ]
    }}
  ],
  "root_causes": [...],
  "evidence": [...],
  "affected_files": [...],
  "test_references": [...],
  "risks": [...]
}}
```

**`investigation_summary.md`** — plain text investigation summary

**`fix_plan.md`** — proposed minimal fix plan

### Important rules
- Do NOT edit any files under `{workspace}/broken/` during investigation
- Do NOT invent source code references — only reference lines you have actually read
- Do NOT declare a finding unless you have verified it in the actual source code
- Every finding must have at least one source reference with actual file paths and line numbers

After writing artifacts, return to ContractGuard and click **Sync Results**.
"#,
            id = investigation.id,
            title = investigation.title,
            description = investigation.description,
            workspace = workspace_path,
            project_id = investigation.project_id,
        )
    }

    pub fn generate_implementation_prompt(
        investigation: &Investigation,
        config: &Config,
        plan_hash: &str,
        fix_plan: &str,
    ) -> String {
        let workspace_path = config.workspace_root
            .join(&investigation.workspace_id)
            .display()
            .to_string();

        format!(r#"# ContractGuard — Implementation Task

## Investigation ID
{id}

## Approved Plan Hash
{plan_hash}

## Fix Plan
{fix_plan}

## Your Task

The fix plan above has been reviewed and approved by the developer.
Implement the approved corrections in the isolated workspace.

### Files to modify
All modifications must be made under: `{workspace}/broken/`

### Steps

1. Read `{workspace}/bob_artifacts/diagnosis.json` — understand the confirmed root causes
2. Read `{workspace}/bob_artifacts/fix_plan.md` — understand the approved approach
3. Make the minimal code corrections:
   - Fix BUG-001: Frontend multipart field name mismatch
   - Fix BUG-002: Backend single-file processing defect
4. Add regression tests that cover the fixed behaviors
5. Run `cargo test` in `{workspace}/broken/backend/` and capture results
6. Run `npm test -- --run` in `{workspace}/broken/frontend/` and capture results
7. Review the actual diff of all changed files

### Write implementation artifacts to `{workspace}/bob_artifacts/`

**`implementation_summary.md`** — what was changed and why

**`changed_files.json`**:
```json
{{
  "schema_version": 1,
  "investigation_id": "{id}",
  "plan_hash": "{plan_hash}",
  "files": [
    {{
      "relative_path": "broken/frontend/src/...",
      "change_type": "modified",
      "description": "Fixed field name"
    }}
  ]
}}
```

**`review.md`** — self-review covering correctness, test coverage, unrelated changes, remaining risks

### Rules
- Only modify files within the approved plan scope
- Do NOT weaken test assertions to make failing tests pass
- If the required changes exceed the approved plan, stop and request renewed approval
- All targeted regression tests must pass after implementation
"#,
            id = investigation.id,
            workspace = workspace_path,
            plan_hash = plan_hash,
            fix_plan = fix_plan,
        )
    }
}
