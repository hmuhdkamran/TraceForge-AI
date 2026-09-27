# IBM Bob Review Task Template

Perform a senior engineer code review of the implemented corrections against the original bug intake and contract requirements.

## Review Checklist
1. **Contract Conformance**: Does `broken/frontend/src/api.ts` now strictly use field name `"files"`?
2. **File Processing Logic**: Does `broken/backend/src/main.rs` iterate over all parts in the multipart stream?
3. **Regression Tests**: Do tests cover single file, multi-file, unsupported format, and size limits?
4. **Unintended Side Effects**: Did any unrelated files or behaviors change?
5. **Security**: Are size limits and path checks preserved?

## Output
Write `bob_artifacts/review.md` summarizing the assessment, test outcomes, and any remaining risks or limitations.
