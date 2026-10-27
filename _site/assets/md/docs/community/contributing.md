# Contributing

Contributions are welcome across correctness, documentation, benchmarks, examples, developer
experience, and performance analysis. By participating, you agree to follow the
[Code of Conduct](/docs/community/code-of-conduct).

## Good first contributions

* Improve docs, Rustdoc, examples, or these pages (every page has a *Suggest changes* link).
* Add benchmark reproduction notes for different hardware.
* Add test vectors for Binary, Sparse, or Patricia trees.
* Improve error messages or API examples.
* Investigate performance with Criterion, `perf`, or flamegraphs.

## Development setup

```bash
git clone https://github.com/1888-Labs/MerkleForge.git
cd MerkleForge
cargo test --workspace
```

The workspace pins Rust 1.97.1 in `rust-toolchain.toml`; `rustup` installs it automatically.

For work on this website (in `docs/`, built with [Vocs](https://vocs.dev)):

```bash
cd docs
npm ci
npm run dev
```

## Recommended workflow

1. Open or comment on an issue first for large API, storage, or cryptography changes.
2. Create a focused branch, for example `fix/docs-links` or `feat/sparse-proof-tests`.
3. Keep the pull request small enough to review comfortably.
4. Add or update tests for behaviour changes.
5. Update docs when public APIs, examples, or benchmark behaviour change.

## Required checks

Run these before opening a pull request:

```bash
make fmt    # rustfmt
make lint   # Clippy across the workspace, warnings denied
make test   # all workspace tests in release mode
```

For website changes, also run:

```bash
npm --prefix docs run build
```

## Pull request guidelines

* Explain the problem, the change, and how you verified it.
* Link related issues.
* Keep `target/`, local benchmark noise, editor files, and temporary artifacts out of commits.
* Include benchmark numbers only with the environment and command recorded.
* Avoid unrelated refactors in feature or bug-fix pull requests.

## Security

Don't report vulnerabilities in public issues. Follow the disclosure process on the
[Security](/docs/reference/security) page.
