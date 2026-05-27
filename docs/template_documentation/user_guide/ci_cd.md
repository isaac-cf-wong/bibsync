# CI/CD

GitHub Actions run on pull requests and pushes to `main`.

- `ci.yml` runs formatting, clippy, tests, docs, and audit checks.
- `documentation.yml` publishes the Zensical site to GitHub Pages.
- `release.yml` creates GitHub releases from tags.
- `publish.yml` verifies tagged crate releases but does not publish while the
  template safety switch is disabled.
- `scheduled_release.yml` checks weekly for unreleased commits, creates a tag,
  and creates a GitHub release. It only publishes the crate after publishing is
  explicitly enabled.

## Enabling crates.io Publishing

Publishing is disabled by default because this repository is a template.

To enable publishing in a real package:

1. Create a `crates-io` GitHub environment.
2. Add a `CARGO_REGISTRY_TOKEN` secret with a crates.io API token.
3. Set `PUBLISH_TO_CRATES_IO` to `true` in `.github/workflows/publish.yml`.
4. Set `publish: true` in the `publish` job of
   `.github/workflows/scheduled_release.yml` if scheduled releases should
   publish.
5. For manual publish runs, set the `publish` workflow input to `true`.
