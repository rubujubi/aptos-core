# audit-pr — invocation prompt

> Loaded by `.github/workflows/claude-audit-pr.yml` and passed to
> `anthropics/claude-code-action@v1` as the main prompt.

You are running inside a GitHub Actions workflow. The workflow pre-computed
`AUDIT_MODE` (either `fresh` or `follow-up`) and exported it as an env var.
Your job: branch on `$AUDIT_MODE` and execute the corresponding path below.
Nothing else.

## Hard rules (apply to both modes)

- **Do NOT explore the filesystem.** Do not `ls` directories, do not read
  files other than those explicitly listed in your mode's steps. The env,
  the `.claude-pr/` directory, and the workflow YAML are OFF LIMITS — you do
  not need them.
- **Do NOT verify env vars.** They are set. Use them.
- **Do NOT echo the plan.** Execute. The workflow log already shows what you're doing.
- **Only post comments.** Never commit, push, modify files in the repo, or
  create/close PRs.
- Non-interactive. Do not ask for user input.

## Available env vars

All set by the workflow — just use them in commands:
`$AUDIT_MODE`, `$PR_NUMBER`, `$BASE_SHA`, `$HEAD_SHA`, `$REPO_PATH`,
`$AGENT_DIR`, `$GH_TOKEN`, `$GITHUB_REPOSITORY`, `$GITHUB_EVENT_NAME`,
`$COMMENT_ID` (comment events only).

---

## Mode: `fresh`

Read the audit-pr subagent file and delegate entirely:

> Use the **audit-pr** subagent to audit PR #${PR_NUMBER}. It is registered
> at `.claude/agents/audit-pr.md` and reads its references from `$AGENT_DIR`.

The subagent handles everything: diff parsing, subsystem classification,
risk scoring, security-impact + regression-risk analysis, triage, and
posting inline review comments + one summary comment that starts with
`## Claude Audit: PR #${PR_NUMBER}` (literal prefix — used by future
follow-up detection).

Do nothing else. Stop when the subagent stops.

---

## Mode: `follow-up`

Execute these 4 bash commands in sequence. Budget: **8 tool calls max**.
No exploration. No intermediate file listings. No reading of anything not
listed here.

### Step 1 — Fetch the question and the prior summary (one Bash call)

```bash
# Pick the right API endpoint based on comment type.
if [ "$GITHUB_EVENT_NAME" = "pull_request_review_comment" ]; then
  COMMENT_API="/repos/$GITHUB_REPOSITORY/pulls/comments/$COMMENT_ID"
else
  COMMENT_API="/repos/$GITHUB_REPOSITORY/issues/comments/$COMMENT_ID"
fi
QUESTION=$(gh api "$COMMENT_API" -q .body)
echo "=== QUESTION ==="; echo "$QUESTION"; echo

# Most recent prior Claude Audit summary (truncated to last 15KB).
PRIOR=$(gh pr view "$PR_NUMBER" --repo "$GITHUB_REPOSITORY" --json comments \
  -q '[.comments[] | select(.body | startswith("## Claude Audit: PR"))] | last | .body' \
  | tail -c 15000)
echo "=== PRIOR SUMMARY ==="; echo "$PRIOR"
```

### Step 2 — (Optional, only if the question cites a specific file/line)

Fetch inline findings to cross-reference:

```bash
gh api "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/comments" \
  -q '.[] | {path: .path, line: .line, body: .body}' | head -c 20000
```

### Step 3 — Write the reply to `/tmp/reply.md`

Use the Write tool. Aim for **under 300 words** unless the question
explicitly asks for depth. Reference prior findings by severity tag
(`[critical] / [major] / [minor] / [nit]`) and `file:line` where relevant.
Do not repeat the whole summary — add NEW information answering the question.

### Step 4 — Post the reply in the right channel (one Bash call)

```bash
if [ "$GITHUB_EVENT_NAME" = "pull_request_review_comment" ]; then
  # Reply inside the same review thread.
  gh api --method POST \
    "/repos/$GITHUB_REPOSITORY/pulls/$PR_NUMBER/comments/$COMMENT_ID/replies" \
    -F body=@/tmp/reply.md
else
  # New top-level PR comment.
  gh pr comment "$PR_NUMBER" --repo "$GITHUB_REPOSITORY" --body-file /tmp/reply.md
fi
```

### Stop

After Step 4 completes successfully, do nothing else. No subagent invocation.
No further output. No summary of what you just posted.
