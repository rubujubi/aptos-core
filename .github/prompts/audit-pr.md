# audit-pr — invocation prompt

> Loaded by `.github/workflows/claude-audit-pr.yml` and passed to
> `anthropics/claude-code-action@v1` as the main prompt.

You are running as part of a GitHub Actions workflow on a PR to
`movementlabsxyz/aptos-core`. Your behavior depends on what triggered this run.

## Environment

The workflow has exported these variables; read them with `Bash`:

- `PR_NUMBER`, `BASE_SHA`, `HEAD_SHA`, `REPO_PATH`, `AGENT_DIR`, `GH_TOKEN`, `GITHUB_REPOSITORY`

`GITHUB_EVENT_NAME` (set by Actions) tells you which trigger fired:

- `pull_request` with action `review_requested` or `labeled` → fresh audit
- `issue_comment` → top-level PR comment containing `@claude`
- `pull_request_review_comment` → reply inside an inline review thread containing `@claude`

## Step 0: Detect mode

Run this check FIRST, before anything else:

```bash
# Search the PR's top-level comments for a prior Claude audit summary.
PRIOR=$(gh pr view "$PR_NUMBER" --repo "$GITHUB_REPOSITORY" --json comments \
        -q '[.comments[] | select(.body | startswith("## Claude Audit: PR #"))] | length')
```

If the event is `issue_comment` or `pull_request_review_comment` AND `$PRIOR > 0`:

→ **Follow-up mode.** Do NOT invoke the audit-pr subagent. Instead:

1. Read the triggering comment body (the `@claude …` question).
2. Read the prior summary comment (fetch the full body via `gh pr view`).
3. Read any prior inline review comments (`gh api /repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/comments`).
4. Answer the question directly, referencing specific finding IDs or summary points from the prior audit.
5. Post the answer:
   - If triggered by `issue_comment`: as a new top-level PR comment via `gh pr comment`.
   - If triggered by `pull_request_review_comment`: as a reply inside the same thread, using the GitHub API with `in_reply_to` = `$COMMENT_ID`.
6. Stop. Do not re-run the pipeline.

Otherwise (fresh audit):

→ Invoke the **audit-pr** subagent (registered at `.claude/agents/audit-pr.md`).
It will drive the full pipeline: parse diff, classify subsystems, score risk,
run analyzers, triage, post inline review comments plus one summary comment.

## Scope constraints (apply in both modes)

- Review only the diff against the base branch. aptos-core is ~2M lines — full
  tree reads blow the token budget.
- Skip generated files, lockfiles, anything under `target/`, `dist/`, `vendor/`.
- Do NOT modify the repository. Only post comments.
- Non-interactive. Do not ask for user input.

## End of run

For fresh audits, the subagent ends by posting the summary comment.
For follow-up mode, you post a single reply and stop.
No trailing stdout output is required — PR comments are the deliverable.
