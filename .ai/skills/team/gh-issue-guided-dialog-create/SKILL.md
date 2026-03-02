---
name: gh-issue-guided-dialog-create
description: Guide issue creation from a live user conversation and file a GitHub issue via gh CLI. Use when a user says they want to create/report an issue and needs interactive questioning to produce a complete, secure, non-duplicate issue aligned with .github/ISSUE_TEMPLATE and docs/rules/issue_creation.md.
---

# GH Issue Guided Dialog Create

Create a complete GitHub issue from dialogue, then file it with `gh issue create`.

## Workflow

1. Confirm prerequisites.
- Run `gh auth status`.
- If authentication fails, ask the user to authenticate and stop.

2. Classify the issue type.
- Pick one: `bug`, `feature`, `task`.
- If unclear, ask one short disambiguation question.

3. Run the intake checklist.
- Follow `references/question_checklist.md`.
- Collect: background, repro/spec, expected result, acceptance criteria, impact scope.
- Do not infer missing critical facts.

4. Build draft body.
- Structure the body with `.github/ISSUE_TEMPLATE/04-ai-dialog-report.md` headings.
- Ensure at least one actionable acceptance checkbox exists.

5. Check duplicates.
- Run `scripts/search_similar_issues.sh "<query>"` with a concise query.
- If a likely duplicate appears, present it and ask whether to continue.

6. Validate issue body safety and completeness.
- Save draft to a temporary markdown file.
- Run:
  - `python3 scripts/validate_issue_body.py --file /tmp/issue.md --kind <bug|feature|task>`
- If validation fails, fix missing sections and remove secrets.

7. Ask for final confirmation.
- Show title and short summary.
- Request explicit approval before filing.

8. Create the issue.
- Run `gh issue create --title "..." --body-file /tmp/issue.md`.
- Return issue number/URL and a 1-2 line summary.

## References

- `references/question_checklist.md`
- `docs/rules/issue_creation.md`
- `.ai/docs/issue_creation.md`
- `.github/ISSUE_TEMPLATE/01-bug-report.yml`
- `.github/ISSUE_TEMPLATE/02-feature-request.yml`
- `.github/ISSUE_TEMPLATE/03-task.yml`
- `.github/ISSUE_TEMPLATE/04-ai-dialog-report.md`
