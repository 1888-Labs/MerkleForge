# AGENTS.md

This file gives coding agents repository-specific guidance for MerkleForge.
It applies to the entire repository unless a deeper `AGENTS.md` overrides it.

## Project Overview

MerkleForge is a Rust framework for authenticated data structures. The
workspace provides shared traits and types, pluggable hash adapters, Binary and
Sparse Merkle trees, an Ethereum-compatible Merkle Patricia Trie, benchmarks,
and examples. The documentation website lives in
[1888-Labs/merkleforge-docs](https://github.com/1888-Labs/merkleforge-docs).

## Repository Map

- `merkle-core/`: public traits, shared types, serialization, and errors. Keep
  this crate lightweight and independent of concrete tree implementations.
- `merkle-hash/`: SHA-256, Keccak-256, and BLAKE3 adapters implementing
  `HashFunction`. The published crate name is `merkleforge-hash`.
- `merkle-variants/`: Binary, Sparse, and Patricia tree implementations and
  their variant-specific proof APIs.
- `merkle-bench/`: Criterion benchmarks. Benchmark code must not affect the
  public library API.
- `tests/`: repository-level Python tests for supporting scripts.
- `.github/`: CI, release, Pages, and notification automation.

## Development Rules

- Read the relevant trait contract and nearby tests before changing behavior.
- Keep changes focused. Do not refactor unrelated code or rewrite benchmark
  results as part of a feature or bug fix.
- Preserve `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]` where present.
- Public Rust APIs require useful Rustdoc, including error behavior where
  applicable.
- Use workspace dependency declarations in the root `Cargo.toml` when a
  dependency is shared by multiple crates.
- Keep published crate names distinct from directory names: `merkle-core`,
  `merkleforge-hash`, and `merkle-variants`.
- Do not commit generated `target/`, Criterion output, website build output,
  profiling captures, editor files, or local environment files.
- Never add secrets, private keys, tokens, or credentials to source, examples,
  fixtures, documentation, or Git history.

## Cryptographic Invariants

- Preserve domain separation between leaves and internal nodes. Leaf and node
  hashing must remain unambiguous for every supported hash adapter.
- Roots and proofs must be deterministic for identical ordered inputs.
- Proof verification must fail for a changed leaf, sibling, path/index, or
  expected root.
- Binary-tree padding, Sparse-tree empty hashes and shortcut nodes, and
  Patricia-trie RLP/hex-prefix encoding are consensus-sensitive behavior.
  Changes require focused regression tests.
- Preserve Ethereum compatibility for Patricia nodes, references, roots, and
  fixtures. Do not update Ethereum vector expectations merely to make a test
  pass.
- Avoid data-dependent shortcuts that weaken correctness or make malformed
  proofs silently valid.

## Compatibility

- The workspace uses Rust edition 2024 and the toolchain pinned in
  `rust-toolchain.toml`.
- Keep `merkle-core` compatible with its documented `std`/`alloc` feature
  model. Do not introduce unconditional `std` usage into `no_std` paths.
- Treat existing public types, trait methods, serialized forms, and error
  behavior as compatibility boundaries. Call out intentional breaking changes.
- Keep the dual MIT/Apache-2.0 licensing headers and metadata intact.

## Testing

Run the smallest relevant check while iterating, then the full required checks
before finalizing a cross-cutting change.

```bash
# Focused examples
cargo test -p merkle-core
cargo test -p merkleforge-hash
cargo test -p merkle-variants

# Required Rust checks
make fmt
make lint
make test
```

For `no_std` or feature-gating work, also check the affected crate without its
default features, using the feature combination documented by that crate.

For Python automation changes:

```bash
python3 -m unittest discover -s tests
```

If a required command cannot be run, report that clearly with the reason.

## Benchmarks

- Use the focused Criterion target while developing; avoid running the full
  suite when a narrow benchmark is sufficient.
- Record the command, Rust version, CPU, OS, sample settings, and relevant
  feature flags for publishable numbers.
- Compare results from the same machine and configuration. Do not claim a
  regression or improvement from unlike environments.
- Update `BENCHMARKS.md` only from completed measurements. Keep incomplete or
  estimated values clearly labeled.
- Energy measurements require the platform support described in
  `benchmarks/ENERGY.md`; wall-clock time is not an energy measurement.

## Documentation And Website

- Update crate READMEs, root documentation, examples, and `CHANGELOG.md` when a public API or
  user-visible behavior changes, and the pages in merkleforge-docs to match.
- Keep examples compilable and use the current public crate names.
- Keep the security notice accurate: MerkleForge has not completed an
  independent security audit.

## Git And Pull Requests

- Use `develop` as the integration branch unless the task explicitly names a
  different base. Sync it with the latest `main` before starting new work.
- Do not discard or overwrite unrelated local changes.
- Use focused Conventional Commit-style messages such as `feat(sparse): ...`,
  `fix(binary): ...`, `test(patricia): ...`, or `docs: ...`.
- Pull requests should explain the problem, implementation, compatibility or
  security impact, and exact verification commands run.
- Do not force-push, rewrite shared history, publish crates, create releases,
  or update benchmark baselines unless the user explicitly requests it.

## Completion Checklist

- The implementation matches the owning crate's existing abstractions.
- Correctness and tamper/failure cases have focused tests.
- Formatting, Clippy, and relevant tests pass.
- Public APIs and examples are documented.
- No generated files, unrelated edits, or sensitive values are included.
- The final report lists changed files, checks run, and any remaining risk.
