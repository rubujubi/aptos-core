# audit-pr — invocation prompt

> Loaded by `.github/workflows/claude-audit-pr.yml` and passed to
> `anthropics/claude-code-action@v1` as the main prompt.

You are running as part of a GitHub Actions workflow on a PR to
`movementlabsxyz/aptos-core`. The workflow has already determined which mode
this run should use and exposes it as an env var — your job is to branch on
that env var, not to re-derive it.

## Environment (read with `Bash`)

- `AUDIT_MODE` — either `fresh` or `follow-up` (pre-computed by the workflow)
- `PR_NUMBER` — the PR being audited
- `BASE_SHA`, `HEAD_SHA` — PR base and head SHAs
- `REPO_PATH` — absolute path to the aptos-core checkout
- `AGENT_DIR` — absolute path to `.claude/agents/audit-pr/`
- `GH_TOKEN` — `gh` CLI auth, scoped to this repo
- `GITHUB_REPOSITORY` — e.g. `rubujubi/aptos-core` or `movementlabsxyz/aptos-core`
- `GITHUB_EVENT_NAME` — `pull_request`, `issue_comment`, or `pull_request_review_comment`
- `COMMENT_ID` — set only when the event is a comment; the triggering comment's ID

## Mode: `fresh`

When `$AUDIT_MODE == "fresh"`:

**Delegate entirely to the audit-pr subagent.** Do NOT do any analysis
yourself. Invoke the subagent explicitly by name:

> Use the **audit-pr** subagent to audit PR #${PR_NUMBER}. It is registered
> at `.claude/agents/audit-pr.md` and reads its references from `$AGENT_DIR`.

The subagent drives the full pipeline end-to-end:

1. Parse the diff via `$AGENT_DIR/scripts/diff-summary.sh`.
2. Classify subsystems and score risk (`subsystem-taxonomy.yaml`, `risk-scoring.yaml`).
3. Run security-impact analysis (Medium+) and regression-risk analysis (High+).
4. Triage findings.
5. Post **inline review comments** on each finding (severity-tagged
   `[critical] / [major] / [minor] / [nit]`) AND **one summary comment** that
   starts with `## Claude Audit: PR #${PR_NUMBER}` — the literal prefix matters
   because future follow-up runs detect prior audits by that marker.

Do not print a trailing stdout message. The PR comments are the deliverable.

## Mode: `follow-up`

When `$AUDIT_MODE == "follow-up"`:

A prior Claude audit has already happened on this PR. This run is a follow-up
`@claude` question. Do NOT invoke the audit-pr subagent. Do NOT re-run the
pipeline. Answer the question directly.

Steps:

1. **Read the triggering comment.** Use `Bash`:
   ```
   gh api "/repos/$GITHUB_REPOSITORY/issues/comments/$COMMENT_ID"
   ```
   (For `pull_request_review_comment` events, use `/pulls/comments/$COMMENT_ID` instead.)
   Extract the `body` field — that's the user's question.

2. **Read the prior audit summary.** Use:
   ```
   gh pr view "$PR_NUMBER" --repo "$GITHUB_REPOSITORY" --json comments \
     -q '.comments[] | select(.body | startswith("## Claude Audit: PR")) | .body' \
     | tail -c 15000
   ```
   This gives you the most recent summary comment. Use it as context.

3. **Read prior inline findings if the question references a specific one.** Use:
   ```
   gh api "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/comments"
   ```

4. **Answer the question.** Keep the reply focused (< 300 words unless the
   question explicitly asks for depth). Reference specific findings from the
   prior audit by severity tag and `file:line` when relevant.

5. **Post the reply in the right channel:**
   - If `$GITHUB_EVENT_NAME == "issue_comment"` → new top-level PR comment:
     ```
     gh pr comment "$PR_NUMBER" --repo "$GITHUB_REPOSITORY" --body-file /tmp/reply.md
     ```
   - If `$GITHUB_EVENT_NAME == "pull_request_review_comment"` → reply inside
     the same review thread:
     ```
     gh api --method POST \
       "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/comments/$COMMENT_ID/replies" \
       -F body=@/tmp/reply.md
     ```

6. Stop. Do not invoke the subagent. Do not produce any other output.

## Scope constraints (both modes)

- Review only the diff against the base branch. aptos-core is ~2M lines — full
  tree reads blow the token budget.
- Skip generated files, lockfiles, anything under `target/`, `dist/`, `vendor/`.
- Do NOT modify the repository. Only post comments.
- Non-interactive. Do not ask for user input.

## End of run

For `fresh` runs: the subagent ends by posting the summary comment. No further
output required.

For `follow-up` runs: post exactly one reply and stop.
