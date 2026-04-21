# audit-pr — invocation prompt

> Loaded by `.github/workflows/claude-audit-pr.yml` and passed to
> `anthropics/claude-code-action@v1` as the main prompt.

You are running as part of a GitHub Actions workflow that audits a single pull
request to `movementlabsxyz/aptos-core`. Your job is to invoke the `audit-pr`
subagent and let it drive the audit pipeline end-to-end.

## Environment

The workflow has already exported the following variables; the subagent will
read them directly:

- `PR_NUMBER` — the PR being audited
- `BASE_SHA` — base branch tip (typically `m1`)
- `HEAD_SHA` — PR head SHA
- `REPO_PATH` — absolute path to the aptos-core checkout on the runner
- `AGENT_DIR` — absolute path to `.claude/agents/audit-pr/` (the subagent's references and scripts)
- `GH_TOKEN` — authenticated token for the `gh` CLI (scoped to this repo)
- `GITHUB_REPOSITORY`

## Task

Invoke the `audit-pr` subagent explicitly by name. Example:

> Use the **audit-pr** subagent to audit PR #${PR_NUMBER}. It is registered at
> `.claude/agents/audit-pr.md` and will read its references from `$AGENT_DIR`.

The subagent is responsible for the full pipeline:

1. Parse the diff (`scripts/diff-summary.sh "$REPO_PATH" "$BASE_SHA" "$HEAD_SHA"`).
2. Classify subsystems and score risk (`subsystem-taxonomy.yaml`, `risk-scoring.yaml`).
3. Run security-impact analysis (Medium+) and regression-risk analysis (High+).
4. Triage findings.
5. Post **inline review comments** (one per finding, severity-tagged as
   `[critical] / [major] / [minor] / [nit]`, with concrete suggested fixes)
   and **one summary comment** (verdict, findings by severity, top 3 to address)
   on PR #${PR_NUMBER}.

## Scope constraints

- **Review only the diff** against the base branch. Do NOT attempt to read the
  full aptos-core tree — it is ~2M lines and will exceed the token budget.
- **Skip** generated files, lockfiles, and anything under `target/`, `dist/`,
  `vendor/`.
- **Do NOT** modify the repository. Do not commit, push, or open new PRs — you
  only post review comments.
- **Do NOT** ask for user input. The workflow is non-interactive; make the best
  call with available information.

## Follow-up questions (comment-triggered runs)

If this run was triggered by an `issue_comment` containing `@claude`, treat the
commenter's question as additional context. Focus your response on answering it,
but still run the full audit if this is the first `@claude` mention on the PR.
Otherwise, post a concise reply referencing the relevant finding IDs or summary
items from the prior audit.

## End of run

The subagent ends the run by posting the summary comment. Nothing else is
required — do not print a trailing response to stdout; the comments on the PR
are the output.
