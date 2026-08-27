# GitHub Actions Account-Unblock Runbook

Status: Source-control recovery procedure — no billing or workflow authority

## Trigger

Use this runbook only when a required GitHub Actions job completes with no
runner and `steps: []`, with an annotation that recent account payments failed
or the Actions spending limit must be increased. This is infrastructure-blocked,
not a repository test failure.

## Safety rule

Do not push another checkpoint, rerun a workflow, change required checks, or
weaken the workflow to test whether the account recovered. Billing and budget
changes are owner-controlled external actions. Repository agents may inspect
non-secret workflow metadata but must not read or change account billing
information.

## Owner-side prerequisite

In GitHub **Settings → Billing & licensing**, resolve the failed payment or
Actions budget condition for the private repository owner. GitHub's current
documentation explains that private-repository hosted-runner usage can be
blocked by an invalid payment method or an exhausted hard budget:

- <https://docs.github.com/en/billing/concepts/product-billing/github-actions>
- <https://docs.github.com/en/billing/how-tos/set-up-budgets>

No repository state changes during this prerequisite.

## Single-attempt resume sequence

1. Confirm in GitHub that the payment/budget condition is resolved before any
   repository publication.
2. Verify the SSD worktree is clean and the complete intended local head is
   reviewed. Do not rewrite, squash, rebase, or discard its commits.
3. Push that head once to the existing PR branch. Do not rerun an older head
   first; the updated pull-request run is the single recovery attempt.
4. Inspect the exact job metadata. Readiness requires a named runner, a
   non-empty step list, the expected source SHA, and every required step green.
5. If the job again has no runner or no steps, record one infrastructure blocker
   and stop. Do not poll, push a no-op commit, or retry.
6. If real steps execute and a repository check fails, diagnose once, batch one
   coherent repair, independently review it, and publish one new checkpoint.
7. Only after the exact PR head is real-step green may normal review, readiness,
   merge, exact-merge verification, and the distinct `main` workflow gate
   continue.

## Proof retained

Record the PR number, head SHA, workflow/run/job/attempt IDs, runner identity,
step count, conclusion, and URL. A green badge, old successful run, zero-step
conclusion, or billing-screen screenshot is not source acceptance evidence.
