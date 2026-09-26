# Submission Checklist

## Application

- [ ] Dashboard starts successfully
- [ ] Rust API starts successfully
- [ ] Health endpoint responds: `GET /api/v1/health`
- [ ] Broken UploadLab fixture compiles and runs tests

## Investigation Workflow

- [ ] Create investigation via dashboard
- [ ] Workspace created automatically
- [ ] Investigation prompt generated correctly
- [ ] Baseline tests execute and capture failures
- [ ] Bob investigation artifacts can be synced
- [ ] Findings display with source references
- [ ] Evidence graph renders
- [ ] Fix plan approval requires confirmation
- [ ] Approved plan stored with content hash
- [ ] Implementation prompt generated after approval
- [ ] Verification tests execute independently
- [ ] Verified status requires actual test passage
- [ ] Report generates with HTML and JSON export

## Sample Project

- [ ] BUG-001: Frontend field name test fails on broken fixture
- [ ] BUG-002: Multi-file backend test fails on broken fixture
- [ ] OpenAPI spec present and accurate
- [ ] Acceptance criteria documented
- [ ] Tests named and described in README

## Code Quality

- [ ] `cargo test` passes in backend
- [ ] `npm test` passes in frontend
- [ ] TypeScript strict mode enabled
- [ ] No hardcoded secrets
- [ ] Error handling for all failure modes

## Security

- [ ] Path traversal protection tested
- [ ] No arbitrary command execution
- [ ] Demo mode disables mutations
- [ ] `.env.example` provided (no real secrets)

## Submission Assets

- [ ] Public repository URL
- [ ] problem_solution.md (≤500 words)
- [ ] bob_usage.md (≤500 words)
- [ ] demo_script.md
- [ ] presentation_outline.md
- [ ] Bob session screenshots in bob_sessions/
- [ ] Video (≤3 minutes, ≥90s of live app)
- [ ] Deployed demo URL (optional)

## Documentation

- [ ] Root README.md with startup instructions
- [ ] UploadLab README with known defects
- [ ] API documentation (openapi.yaml)
- [ ] Architecture documentation
