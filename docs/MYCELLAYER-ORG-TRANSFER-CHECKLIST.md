# MycelLayer Org Transfer Checklist

Status: proposed implementation checklist for moving `ctf2090/Mycel` to the
`MycelLayer` GitHub organization

Use this checklist when we are ready to move the repository from the current
personal account owner to a dedicated organization.

Read this together with:

- [`docs/PROJECT-NAMING.md`](./PROJECT-NAMING.md)
- [`docs/GITHUB-ADOPTION-PLAN.md`](./GITHUB-ADOPTION-PLAN.md)
- [`docs/OUTWARD-RELEASE-CHECKLIST.md`](./OUTWARD-RELEASE-CHECKLIST.md)

This checklist is intentionally operational. It is about transfer sequencing,
ownership, and follow-up validation, not about changing protocol or product
behavior.

## 1. Decision And Scope

- [ ] Confirm that `MycelLayer` is the chosen organization name.
- [ ] Confirm that the organization will be the long-term public-facing owner
      for the repository, not just a temporary transfer target.
- [ ] Confirm whether the repository should remain named `Mycel` after the
      transfer.
- [ ] Decide whether any other repositories, Pages sites, or GitHub Apps should
      move in the same batch.
- [ ] Decide who the initial organization owners are.

## 2. Organization Setup

- [ ] Create the `MycelLayer` organization instead of converting the existing
      personal account.
- [ ] Set the initial organization owners.
- [ ] Configure basic organization profile metadata:
      description, avatar, URL, and public profile copy.
- [ ] Decide the default repository permissions for organization members.
- [ ] Decide whether only organization owners can delete or transfer
      repositories.
- [ ] Create the initial teams if we want team-based ownership immediately
      after transfer.

Suggested first teams:

- `maintainers`
- `docs`
- `delivery`

## 3. Pre-Transfer Repository Audit

- [ ] Record the current repository URL: `https://github.com/ctf2090/Mycel`
- [ ] Record the current Pages URL and any custom-domain state.
- [ ] Record current branch protection or ruleset settings.
- [ ] Record current secrets, variables, webhooks, GitHub Apps, and deploy keys.
- [ ] Record current issue labels, milestones, projects, pinned issues, and
      discussion settings.
- [ ] Record package registries or release workflows that assume the current
      owner namespace.
- [ ] Record any local scripts, docs, badges, or links that hard-code
      `ctf2090/Mycel`.

Useful commands:

```bash
gh repo view ctf2090/Mycel --json nameWithOwner,url,owner,hasIssuesEnabled,hasDiscussionsEnabled,deleteBranchOnMerge
gh api repos/ctf2090/Mycel/pages
gh api repos/ctf2090/Mycel/branches/main/protection
gh api repos/ctf2090/Mycel/hooks
gh api repos/ctf2090/Mycel/actions/secrets
gh api repos/ctf2090/Mycel/actions/variables
gh label list --repo ctf2090/Mycel
gh api repos/ctf2090/Mycel/teams
```

## 4. Pages And Domain Plan

- [ ] Decide whether we will keep the default Pages URL or move to a custom
      domain before or after transfer.
- [ ] If we keep the default Pages URL temporarily, note that the owner segment
      will change from `ctf2090.github.io` to `mycellayer.github.io`.
- [ ] If we already use or plan to use a custom domain, prepare the DNS updates
      and repository Pages settings update.
- [ ] Record every public surface that currently points to the old Pages URL.

## 5. Access And Ownership Plan

- [ ] Confirm who needs `owner`, `admin`, `maintain`, `write`, or `triage`
      access after the transfer.
- [ ] Decide whether `ctf2090` should remain an organization owner, repository
      admin, or both.
- [ ] Decide whether to move to team-based access immediately or in a later
      cleanup step.
- [ ] Draft a first-pass `CODEOWNERS` if we want ownership routing soon after
      transfer.

## 6. Transfer Readiness Check

- [ ] Confirm that the target organization can create repositories.
- [ ] Confirm that `MycelLayer` does not already contain a conflicting `Mycel`
      repository or same-network fork.
- [ ] Confirm that repository admins understand the transfer warnings.
- [ ] Confirm that active work is paused or coordinated during the transfer
      window.
- [ ] Freeze non-essential repo-setting changes during the transfer window.

## 7. Execute The Transfer

- [ ] Transfer `ctf2090/Mycel` to the `MycelLayer` organization.
- [ ] Keep the repository name as `Mycel` unless there is an explicit rename
      decision.
- [ ] Confirm that the resulting owner/repo path is
      `https://github.com/MycelLayer/Mycel`.
- [ ] Confirm that repository collaborators, issues, pull requests, releases,
      and discussions remain present after transfer.

## 8. Immediate Post-Transfer Checks

- [ ] Confirm that redirects from `ctf2090/Mycel` to `MycelLayer/Mycel` work on
      the web.
- [ ] Update local clones with:

```bash
git remote set-url origin https://github.com/MycelLayer/Mycel.git
```

- [ ] Confirm `git fetch` and `git push` succeed against the new remote.
- [ ] Re-check branch protection or rulesets because organization defaults and
      policies may now apply.
- [ ] Re-check issue assignment behavior and team visibility.
- [ ] Re-check repository roles and team access.
- [ ] Re-check Actions secrets, variables, and environment protections.
- [ ] Re-check webhooks, deploy keys, and installed GitHub Apps.
- [ ] Re-check package links if we publish any package from this repository.

## 9. Pages And Public Surface Follow-Up

- [ ] Re-verify GitHub Pages build and deployment.
- [ ] Re-verify the live public URL.
- [ ] Update README, homepage, badges, and docs links if they still point to
      `ctf2090/Mycel`.
- [ ] Update any share cards or public screenshots that show the old owner.
- [ ] Run [`docs/OUTWARD-RELEASE-CHECKLIST.md`](./OUTWARD-RELEASE-CHECKLIST.md)
      after public-surface edits.

## 10. Workflow And Automation Follow-Up

- [ ] Re-check GitHub Actions workflow permissions after transfer.
- [ ] Re-check any automation that calls `gh`, GitHub REST APIs, or owner-bound
      URLs.
- [ ] Re-check Pages, CI status widgets, and issue/report scripts.
- [ ] Re-check any automation that depends on the old owner name in badge URLs
      or API calls.
- [ ] Re-check token and secret ownership if any workflow used
      personal-account-specific credentials.

## 11. Documentation Follow-Up

- [ ] Update repo documentation that still names `ctf2090/Mycel` as the primary
      repository path.
- [ ] Keep the naming boundary clear:
      `MycelLayer` remains the outward-facing owner/brand, while `Mycel`
      remains the repository and protocol name.
- [ ] Avoid rewriting protocol/spec text to call the protocol itself
      `MycelLayer`.
- [ ] Update onboarding docs if the default clone URL changes.

## 12. Validation Checklist

- [ ] `gh repo view MycelLayer/Mycel` returns the expected owner and settings.
- [ ] `gh issue list --repo MycelLayer/Mycel` works.
- [ ] `gh pr list --repo MycelLayer/Mycel` works.
- [ ] Latest CI run succeeds after the transfer.
- [ ] Latest Pages deployment succeeds if Pages is enabled.
- [ ] The old repository URL redirects correctly.
- [ ] The current public homepage URL returns `200`.
- [ ] No critical doc or automation path still depends on the old owner.

## 13. Rollback And Recovery Notes

- [ ] If something breaks during the transfer, keep a short incident log with:
      exact time, broken surface, suspected cause, and mitigation.
- [ ] If redirects are insufficient for a critical surface, patch the affected
      docs, badges, or workflows immediately instead of waiting for a broad
      cleanup pass.
- [ ] Do not create a new repository at the old owner/path unless we are sure
      we no longer need GitHub's redirects.

## 14. Recommended Order

Use this order unless a narrower migration plan is required:

1. create and configure the organization
2. inventory current repository settings and public surfaces
3. decide access, team, and Pages handling
4. freeze non-essential repo setting changes
5. transfer `ctf2090/Mycel` to `MycelLayer/Mycel`
6. validate redirects, CI, Pages, and permissions
7. update public-facing docs and automation references
8. adopt follow-up governance improvements such as rulesets and `CODEOWNERS`
