---
name: gh-issue-dialog-refine-and-file
description: Refine rough notes/log snippets/conversation history into a complete issue draft and file it with gh CLI. Use when the user already has partial material and asks to clean it up, fill gaps through minimal questions, and create a high-quality issue aligned with .github/ISSUE_TEMPLATE.
---

# GH Issue Dialog Refine And File

Convert rough input into a complete issue with minimal back-and-forth.

## Workflow

1. Normalize inputs.
- Accept notes, logs, and pasted chat excerpts.
- Extract facts, requests, constraints, and evidence.

2. Classify target type.
- Choose `bug`, `feature`, or `task`.
- Use `references/field_mapping.md` to map required fields.

3. Detect missing critical fields.
- Write current draft to `/tmp/issue-draft.md`.
- Run:
  - `python3 scripts/check_required_fields.py --file /tmp/issue-draft.md --kind <bug|feature|task>`
- Ask only for missing required items.

4. Rebuild into final draft format.
- Rewrite body using `.github/ISSUE_TEMPLATE/04-ai-dialog-report.md` heading order.
- Keep statements concrete and testable.

5. Run safety and duplicate checks.
- Search duplicates with:
  - `gh issue list --state all --search "<query> in:title,body" --limit 20`
- Verify no credentials/personal secrets are included.

6. Confirm and file.
- Ask for final approval.
- Run `gh issue create --title "..." --body-file /tmp/issue-final.md`.
- Return issue number/URL and unresolved assumptions.

## References

- `references/field_mapping.md`
- `docs/rules/issue_creation.md`
- `.ai/docs/issue_creation.md`
- `.github/ISSUE_TEMPLATE/01-bug-report.yml`
- `.github/ISSUE_TEMPLATE/02-feature-request.yml`
- `.github/ISSUE_TEMPLATE/03-task.yml`
- `.github/ISSUE_TEMPLATE/04-ai-dialog-report.md`
