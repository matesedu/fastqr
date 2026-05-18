# GitHub Automation

## Workflows

- Keep workflow jobs on explicit runner images, such as `ubuntu-24.04`, so CI changes are reviewed instead of inherited from moving aliases.
- Keep third-party actions pinned to full commit SHAs. Preserve the upstream version comment next to each SHA so reviews can see the intended release line.
- Keep workflow permissions scoped per job. Prefer top-level read-only permissions, or `permissions: {}` when every job declares its own permissions.
- Keep `timeout-minutes` on each job to avoid stuck runners consuming the Actions queue.

## Dependency Updates

Dependabot checks GitHub Actions, Cargo crates, and npm packages weekly. For GitHub Actions updates, accept the Dependabot PR only after confirming the new commit SHA still belongs to the expected upstream release tag.
