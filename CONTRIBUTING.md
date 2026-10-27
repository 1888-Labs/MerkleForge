# Contributing to MerkleForge Framework

Thanks for helping improve MerkleForge Framework. Contributions are welcome
across correctness, documentation, benchmarks, examples, developer experience,
and performance analysis.

By participating, you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

New here? Read [ARCHITECTURE.md](ARCHITECTURE.md) first. It explains how the
crates fit together and has a "where to make a change" table.

## Good First Contributions

- Improve README docs, Rustdoc, examples, or the [documentation site](https://github.com/1888-Labs/merkleforge-docs).
- Add benchmark reproduction notes for different hardware.
- Add test vectors for Binary, Sparse, or Patricia Merkle trees.
- Improve error messages or API examples.
- Investigate performance bottlenecks with Criterion, `perf`, or flamegraphs.

## Development Setup

```bash
git clone https://github.com/1888-Labs/MerkleForge.git
cd MerkleForge
cargo test --workspace
```

The website and documentation live in their own repository,
[1888-Labs/merkleforge-docs](https://github.com/1888-Labs/merkleforge-docs), and are
published at <https://1888-labs.github.io/MerkleForge/>.

## Recommended Workflow

1. Open or comment on an issue for large API, storage, or cryptography changes.
2. Create a focused branch, for example `fix/docs-links` or
   `feat/sparse-proof-tests`.
3. Keep the pull request small enough to review comfortably.
4. Add or update tests for behavior changes.
5. Update docs when public APIs, examples, or benchmark behavior changes.

## Required Checks

Run these before opening a pull request:

```bash
make fmt
make lint
make test
```

For documentation changes, also run:

```bash
npm --prefix docs run build
```

For performance changes, include before/after numbers from Criterion. See
[BENCHMARKS.md](BENCHMARKS.md) for the commands and [profiling/](profiling/)
for flamegraphs and `perf`.

## Pull Request Guidelines

- Explain the problem, the change, and the verification performed.
- Link related issues.
- Keep generated `target/`, local benchmark noise, editor files, and temporary
  artifacts out of commits.
- Include benchmark numbers only when the environment and command are recorded.
- Avoid unrelated refactors in feature or bug-fix pull requests.

## Releases (maintainers)

Releases run through the manual workflow in `.github/workflows/release.yml`.
Trigger it with a semantic version such as `0.5.1`. It tests the workspace,
bumps the version and changelog, tags the release, publishes the crates,
creates the GitHub release, and sends the Telegram notification. For local
release tooling, run `make install-release-tools`.

## Security

Please do not report vulnerabilities in public issues. Follow the responsible
disclosure process in [`.github/SECURITY.md`](.github/SECURITY.md).
