# Releasing mime-infer

The Release workflow publishes `v*` tags to crates.io and GitHub Releases after
the Rust matrix, dependency audit, and spelling checks pass. Tags must match
`Cargo.toml` and point to a commit contained in `main`.

## Credentials

Set the repository Actions secret `CARGO_REGISTRY_TOKEN` to a crates.io API token
with permission to publish `mime-infer`. The workflow's GitHub token creates the
GitHub Release; it does not need a separate personal access token.

## Prepare a release

1. Update the version in `Cargo.toml` and the dependency example in `README.md`.
2. Add the release notes under `## VERSION` in `CHANGELOG.md`, including the
   minimum Rust version and any compatibility changes.
3. Merge the release pull request after CI and review pass.
4. Tag the merged commit and push the tag:

   ```sh
   git switch main
   git pull --ff-only
   git tag -a v4.1.0 -m "Release mime-infer 4.1.0"
   git push origin v4.1.0
   ```

## Recover a failed release

Fix the underlying credential or service problem and rerun the failed workflow.
The workflow also supports manual dispatch against the existing version tag.
It skips publishing if the version already exists, then checks the downloaded
package's registry checksum and embedded Git commit before creating the GitHub
Release. A version published from another commit fails verification. Publishing
to crates.io is permanent: use a new version for source changes rather than
moving a released tag.

Each GitHub Release includes the package downloaded from crates.io and its
SHA-256 checksum. Release notes come from the matching version's entry in
`CHANGELOG.md`.
