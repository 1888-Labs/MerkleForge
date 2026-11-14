# Contributing

Contributions are welcome across correctness, documentation, benchmarks, examples, developer
experience, and performance analysis. By participating, you agree to follow the
[Code of Conduct](/docs/community/code-of-conduct).

The Rust crates live in [1888-Labs/MerkleForge](https://github.com/1888-Labs/MerkleForge) and the
TypeScript packages in [1888-Labs/MerkleForge-ts](https://github.com/1888-Labs/MerkleForge-ts).

<LangBar />

## Good first contributions

<Rust>
  * Improve docs, Rustdoc, examples, or these pages (every page has a *Suggest changes* link).
  * Add benchmark reproduction notes for different hardware.
  * Add test vectors for Binary, Sparse, or Patricia trees.
  * Improve error messages or API examples.
  * Investigate performance with Criterion, `perf`, or flamegraphs.
</Rust>

<TypeScript>
  * More test vectors and property-style tests (tampered proofs, edge sizes).
  * Docs and examples for browser, Deno, and Bun usage.
  * Benchmarks for tree builds and proof verification.
  * Issues labelled `good first issue`.
  * Improve these pages (every page has a *Suggest changes* link).
</TypeScript>

## Development setup

<Rust>
  ```bash
  git clone https://github.com/1888-Labs/MerkleForge.git
  cd MerkleForge
  cargo test --workspace
  ```

  The workspace pins Rust 1.97.1 in `rust-toolchain.toml`; `rustup` installs it automatically.
</Rust>

<TypeScript>
  You need Node.js 22+ and pnpm. The pnpm version is pinned in `package.json`; run
  `corepack enable` to pick it up.

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  cd MerkleForge-ts
  pnpm install
  pnpm check   # lint, typecheck, test, and package checks
  ```

  | Command | What it does |
  | --- | --- |
  | `pnpm build` | Compile every package with `tsc -b` into `dist/` |
  | `pnpm typecheck` | Type-check sources and tests |
  | `pnpm test` | Run Vitest against the sources (no build needed) |
  | `pnpm coverage` | Tests with V8 coverage |
  | `pnpm lint` / `pnpm format` | Biome lint and format check / auto-fix |
  | `pnpm check:packages` | publint and arethetypeswrong on what npm will ship |
  | `pnpm fixtures:solidity` | Regenerate `contracts/test/fixtures` from the TypeScript packages |
  | `pnpm test:solidity` | Foundry tests for the Solidity verifiers (needs [Foundry](https://getfoundry.sh)) |
</TypeScript>

This website lives in its own repository,
[1888-Labs/merkleforge-docs](https://github.com/1888-Labs/merkleforge-docs), built with
[Vocs](https://vocs.dev):

```bash
git clone https://github.com/1888-Labs/merkleforge-docs.git
cd merkleforge-docs
npm ci
npm run dev
```

## Recommended workflow

<Rust>
  1. Open or comment on an issue first for large API, storage, or cryptography changes.
  2. Create a focused branch, for example `fix/docs-links` or `feat/sparse-proof-tests`.
  3. Keep the pull request small enough to review comfortably.
  4. Add or update tests for behaviour changes.
  5. Update docs when public APIs, examples, or benchmark behaviour change.
</Rust>

<TypeScript>
  1. Branch from `develop` (`feat/...`, `fix/...`, `docs/...`).
  2. Make a focused change with tests.
  3. If it changes a published package, run `pnpm changeset` and describe the change for users.
     Docs, CI, and test-only changes do not need one.
  4. Run `pnpm check`.
  5. Open a pull request into `develop`. `develop` is merged into `main` to release.

  Commits use [Conventional Commits](https://www.conventionalcommits.org/): `feat(sparse): ...`,
  `fix(binary): ...`, `test(patricia): ...`, `docs: ...`.

  The Rust crates are the reference implementation: roots, proofs, and encodings must match them
  byte for byte. When changing tree logic, read the matching Rust source first, and add or regenerate
  test vectors from the Rust crates rather than hand-writing expected hashes. Never edit an expected
  value just to make a test pass.
</TypeScript>

## Required checks

Run these before opening a pull request:

<Rust>
  ```bash
  make fmt    # rustfmt
  make lint   # Clippy across the workspace, warnings denied
  make test   # all workspace tests in release mode
  ```
</Rust>

<TypeScript>
  ```bash
  pnpm check   # pnpm lint, pnpm typecheck, pnpm test, and pnpm check:packages

  # When touching contracts/ or proof shapes
  pnpm fixtures:solidity && pnpm test:solidity
  ```
</TypeScript>

For website changes, run `npm run build` in the docs repository.

## Pull request guidelines

<Rust>
  * Explain the problem, the change, and how you verified it.
  * Link related issues.
  * Keep `target/`, local benchmark noise, editor files, and temporary artifacts out of commits.
  * Include benchmark numbers only with the environment and command recorded.
  * Avoid unrelated refactors in feature or bug-fix pull requests.
</Rust>

<TypeScript>
  * Explain the problem, the change, and any compatibility or security impact.
  * List the commands you ran.
  * Link related issues.
  * Keep unrelated refactors out of the pull request.
</TypeScript>

## Security

Don't report vulnerabilities in public issues. Follow the disclosure process on the
[Security](/docs/reference/security) page.
