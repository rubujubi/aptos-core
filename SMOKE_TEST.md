# Smoke Test — claude-pr-reviewer migration

This file exists only to give the `audit-pr` workflow a non-empty diff to
chew on while we verify that the new "fetch from `claude-pr-reviewer` at
runtime" plumbing works end to end. It is **safe to delete** once the
smoke test passes.

What we want the workflow run to demonstrate:

1. The `Checkout claude-pr-reviewer` step pulls
   `movementlabsxyz/claude-pr-reviewer` at the pinned ref into
   `.claude-reviewer/`.
2. The `Stage Claude assets into workspace` step materialises
   `.claude/agents/audit-pr.md`, `.claude/agents/audit-pr/...`, and
   `.claude/settings.json` in the runner's workspace.
3. The `Load prompt` step reads
   `.claude-reviewer/prompts/audit-pr.md` (NOT the deleted
   `.github/prompts/audit-pr.md`).
4. The `Run Claude audit` step invokes the subagent normally and posts
   the usual summary comment on the PR.

If all four steps succeed, the migration is good to promote to
`movementlabsxyz/aptos-core` PR #318.
