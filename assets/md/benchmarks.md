<div className="mf-page" style={{ paddingTop: 0 }}>
  # Benchmarks

  Rust and TypeScript are measured separately: the Rust crates with Criterion on every site build,
  the TypeScript packages against other JavaScript Merkle tree libraries. Lower is better.

  ## Rust

  Criterion results from the latest site build, across tree variants, operations, hash adapters, and
  sizes. Every row links to its full Criterion report.

  <BenchmarkExplorer />

  ### Faster than what?

  Measured 2026-11-06 for MerkleForge 0.5.2 (i7-1185G7, 4 cores / 8 threads, rustc 1.97.1), SHA-256 everywhere, the same 32-byte leaves for every library, Criterion medians. Every ratio is MerkleForge against the named library, same operation, same leaves, same hash, same run. The libraries produce different roots (MerkleForge domain-separates leaves and nodes), so this compares speed for the same job, not identical output.

  Against [`rs-merkle`](https://crates.io/crates/rs_merkle) 1.5, at 1,048,576 leaves:

  | Operation | MerkleForge | `rs-merkle` | Faster | Threads (MerkleForge / rs-merkle) |
  | --- | ---: | ---: | ---: | --- |
  | Generate one proof | 191 ns | 2.26 µs | **11.8x** | 1 / 1 |
  | Generate a multiproof (100 leaves) | 8.1 µs | 48.3 µs | **6.0x** | 1 / 1 |
  | Build the tree (`parallel` feature) | 72.3 ms | 458 ms | **6.3x** | 8 / 1 |
  | Verify 10,000 proofs (`verify_batch`, `parallel`) | 6.1 ms | 89.0 ms | **14.5x** | 8 / 1 |
  | Verify 10,000 proofs (`verify_batch`, one thread) | 19.3 ms | 88.7 ms | **4.6x** | 1 / 1 |
  | Verify one proof | 1.86 µs | 8.36 µs | **4.5x** | 1 / 1 |
  | Build the tree (one thread) | 183 ms | 437 ms | **2.4x** | 1 / 1 |
  | Verify a multiproof (100 leaves) | 144 µs | 290 µs | **2.0x** | 1 / 1 |

  Against [`merkle_light`](https://crates.io/crates/merkle_light) at 1,048,576
  leaves, MerkleForge builds **2.7x** faster and verifies 10,000 proofs **3.7x**
  faster with `parallel`; single-threaded, the two are within about 20% on every
  operation. `merkle_light` has no multiproofs.

  What favours whom:

  * MerkleForge hashes every leaf with a `0x00` prefix and every node with a
    `0x01` prefix, which blocks second-preimage attacks. That is extra work the
    other libraries skip, and it is counted in MerkleForge's times.
  * `rs-merkle` is given its leaves already hashed in verification (its API
    takes leaf hashes); MerkleForge hashes the raw leaf inside `verify`. In
    construction, `rs-merkle`'s leaf hashing is timed.
  * Rows marked 8 threads use the `parallel` feature (rayon) on this 4-core
    laptop; the other libraries have no multi-threaded build or batch verifier.
    The one-thread rows are the like-for-like comparison.

  ### MerkleForge vs `rs-merkle` vs `merkle_light`

  **10,000 leaves**

  | Operation | MerkleForge | MerkleForge `parallel` | `rs-merkle` | `merkle_light` |
  | --- | ---: | ---: | ---: | ---: |
  | Build (`from_leaves`) | 1.59 ms | 1.20 ms | 3.02 ms | 1.73 ms |
  | Build (one `insert` at a time) | 13.6 ms | 14.6 ms | n/a | n/a |
  | Generate one proof | 193 ns | 172 ns | 1.61 µs | 181 ns |
  | Verify one proof | 1.46 µs | 1.36 µs | 5.93 µs | 1.57 µs |
  | Verify 10,000 proofs | 15.0 ms | 4.55 ms | 68.2 ms | 16.5 ms |
  | Generate a multiproof (100 leaves) | 3.89 µs | 4.29 µs | 24.5 µs | n/a |
  | Verify a multiproof (100 leaves) | 84.6 µs | 82.5 µs | 168 µs | n/a |

  **1,048,576 leaves**

  | Operation | MerkleForge | MerkleForge `parallel` | `rs-merkle` | `merkle_light` |
  | --- | ---: | ---: | ---: | ---: |
  | Build (`from_leaves`) | 183 ms | 72.3 ms | 437 ms | 196 ms |
  | Build (one `insert` at a time) | 2.01 s | 2.10 s | n/a | n/a |
  | Generate one proof | 191 ns | 193 ns | 2.26 µs | 261 ns |
  | Verify one proof | 2.33 µs | 1.86 µs | 8.63 µs | 2.15 µs |
  | Verify 10,000 proofs | 19.3 ms | 6.15 ms | 88.7 ms | 22.6 ms |
  | Generate a multiproof (100 leaves) | 8.11 µs | 7.88 µs | 48.3 µs | n/a |
  | Verify a multiproof (100 leaves) | 144 µs | 142 µs | 290 µs | n/a |

  The `rs-merkle` and `merkle_light` columns come from the single-thread run;
  their numbers in the `parallel` run were within noise (they do not use the
  feature). Proof sizes for a 10,000-leaf tree: MerkleForge 469 bytes,
  `rs-merkle` 448, `merkle_light` 512.

  Full tables and caveats: [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md#faster-than-what) in the Rust repository.

  ### Reproduce

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge.git
  cd MerkleForge

  cargo bench --workspace
  xdg-open target/criterion/report/index.html   # `open` on macOS

  # Focused groups
  cargo bench --bench hash_throughput
  cargo bench --bench binary_tree
  cargo bench --bench sparse_tree
  cargo bench --bench patricia_trie
  cargo bench --bench comparison
  cargo bench --bench comparison --features parallel   # multi-threaded build and verify_batch
  COMPARISON_LEAVES=1048576 cargo bench --bench comparison
  cargo bench --bench bench_energy

  # Comparative proof sizes
  MERKLEFORGE_PRINT_COMPARISON_SIZES=1 cargo bench --bench comparison -- --list
  ```

  The full published tables, environment, and caveats are on the
  [Benchmarks docs page](/docs/performance/benchmarks). To profile hotspots or measure energy, see
  [Profiling & energy](/docs/performance/profiling).

  ## TypeScript

  `@merkleforge/*` compared with
  [`@openzeppelin/merkle-tree`](https://github.com/OpenZeppelin/merkle-tree),
  [`merkletreejs`](https://github.com/merkletreejs/merkletreejs),
  [`@aztec/merkle-tree`](https://www.npmjs.com/package/@aztec/merkle-tree), and
  [`@aztec/world-state`](https://www.npmjs.com/package/@aztec/world-state), from one run of the
  repository's `bench/` suite (Node.js 24.15, i7-1185G7). Read them as ratios, not absolutes.

  ### Faster than what?

  Every TypeScript speed ratio names its baseline. There are three, and they are never mixed:

  | Baseline | What is compared | Headline (131,072 leaves) |
  | --- | --- | --- |
  | **Another JavaScript library** | `@merkleforge/openzeppelin` against [`@openzeppelin/merkle-tree`](https://github.com/OpenZeppelin/merkle-tree) 1.0.8, the library it replaces, on the same leaves in the same run | builds 8–15x faster across sizes; 12.3x for an `[address, uint256]` airdrop tree (1,708 ms vs 21,021 ms), with byte-for-byte identical output |
  | **The previous MerkleForge release** | 0.2.3 against 0.2.2, same harness and machine | `BinaryMerkleTree` with `@merkleforge/hash-wasm` builds 5x faster (485 ms to 98 ms) |
  | **A Keccak-256 implementation** | `keccak256` from `@merkleforge/hash` against `keccak_256` from `@noble/hashes` | about 2x faster per hash, identical digests |

  **There is no speed claim against the Rust crates.** The TypeScript packages are a port of them and
  are checked against them for identical output, not raced against them.

  ### Parity: same bytes first

  A speedup only counts if the output is the same. `bench/parity.mjs` builds the benchmark's own
  trees (the same leaves that are timed) and compares the output byte for byte with OpenZeppelin,
  `@noble/hashes`, and the Rust crates, which it runs live. It runs in CI on every pull request and
  fails the build on any difference.

  | MerkleForge | Compared with | Leaves | What must match | Result |
  |---|---|---|---|---|
  | `keccak256` (@merkleforge/hash) | `keccak_256` (@noble/hashes 2.4.0) | — | 101,025 digests, lengths 0–1024 + random | identical |
  | `SimpleMerkleTree` (@merkleforge/openzeppelin) | @openzeppelin/merkle-tree 1.0.8 | 1,024, 16,384, 131,072 | root, full dump (every node), 2000 proofs, 1 multiproof | identical |
  | `StandardMerkleTree` (@merkleforge/openzeppelin) | @openzeppelin/merkle-tree 1.0.8 | 1,024, 16,384, 131,072 | root, full dump (every node), 2000 proofs, 1 multiproof | identical |
  | `BinaryMerkleTree` + @merkleforge/hash-wasm | `BinaryMerkleTree` + @merkleforge/hash (pure JS) | 1,024, 16,384, 131,072 | root, 2000 encoded proofs | identical |
  | `BinaryMerkleTree` Keccak-256 (@merkleforge/variants) | Rust `merkle-variants` `BinaryMerkleTree` | 1,024, 16,384, 131,072 | root, bincode bytes of 2000 proofs | identical |
  | `BinaryMerkleTree` SHA-256 (@merkleforge/variants) | Rust `merkle-variants` `BinaryMerkleTree` | 1,024, 16,384, 131,072 | root, bincode bytes of 2000 proofs | identical |

  16 checks, all identical. Node v24.15.0, run 2026-11-06.

  The unit tests add property tests against the real `@openzeppelin/merkle-tree` (150 random trees
  per tree type, random leaf types, values, and sizes), test vectors generated by the Rust crates for
  every tree, proof, multiproof, and codec (`serde_json` and bincode), and the Solidity verifiers
  checked with Foundry against fixtures produced by the TypeScript packages.

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  git clone https://github.com/1888-Labs/MerkleForge.git      # the Rust crates, next to it
  cd MerkleForge-ts && pnpm install && pnpm build
  cd bench && npm install
  npm run parity    # about 45 s; exits non-zero on any difference (RUST=0 skips the Rust rows)
  ```

  ### At 131,072 leaves

  | Task | MerkleForge | Others |
  | --- | --- | --- |
  | Build a Keccak-256 tree | **98 ms** with `hash-wasm`, 637 ms pure JS | merkletreejs 935 ms · Aztec StandardTree 2,668 ms · OpenZeppelin 10,493 ms |
  | Build an OpenZeppelin airdrop tree (`[address, uint256]`) | **1,708 ms** (`@merkleforge/openzeppelin`) | OpenZeppelin 21,021 ms (12x slower) |
  | Generate one proof | **2–3 µs** (`BinaryMerkleTree`) | merkletreejs 4 µs · Aztec 54–58 µs · OpenZeppelin 312 µs |
  | Verify one proof | **18 µs** with `hash-wasm`, 52 µs pure JS | merkletreejs 127 µs · Aztec StandardTree 177 µs · OpenZeppelin 292 µs |
  | Install footprint | **1–3 dependencies, about 1.1–1.4 MB** | merkletreejs 3, 1 MB · OpenZeppelin 38, 16 MB · Aztec merkle-tree 362, 395 MB · Aztec world-state 377, 1.1 GB |

  * `BinaryMerkleTree` with `@merkleforge/hash-wasm` builds a Keccak-256 tree about 10x faster than
    merkletreejs and 100x faster than OpenZeppelin, even though it hashes every leaf with a domain
    prefix (`H(0x00 ‖ leaf)`, which blocks second-preimage attacks): twice the hashing of
    merkletreejs, which uses leaves as nodes directly.
  * With pure-JavaScript Keccak it is 1.5x faster than merkletreejs.
  * `@merkleforge/openzeppelin` produces the same roots and proofs as `@openzeppelin/merkle-tree`
    while building trees 8–15x faster, generating proofs 4–6x faster, and verifying them 4–5x faster.
  * The Aztec trees use LMDB storage and, in some runs, Poseidon2, so they are not like for like.

  Measured on MerkleForge 0.2.3, Node.js 24.15, i7-1185G7, 2026-11-01. Per-size tables, the exact
  method, every caveat, and where each claim in the READMEs comes from are in
  [BENCHMARKS.md](https://github.com/1888-Labs/MerkleForge-ts/blob/main/BENCHMARKS.md).

  ### Reproduce

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  cd MerkleForge-ts
  pnpm install && pnpm build
  cd bench
  npm install        # comparison libraries, kept out of the pnpm workspace
  npm run bench          # writes results.json; SIZES, OPS, and BUILD_RUNS adjust the run
  node report.mjs        # prints the tables
  npm run bench:keccak   # Keccak-256 per call against @noble/hashes
  npm run parity         # byte-for-byte parity (needs the Rust repo cloned next to this one)
  ```
</div>
