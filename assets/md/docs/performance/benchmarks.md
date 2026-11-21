# Benchmarks

The Rust crates and the TypeScript packages are benchmarked separately, each with its own harness.
Lower latency is better; higher throughput is better. Results are hardware-specific: compare
trends across machines, not absolute values.

<LangBar />

<Rust>
  MerkleForge uses [Criterion](https://crates.io/crates/criterion) benchmarks in the unpublished
  `merkle-bench` crate to compare hash adapters, tree variants, proof generation, proof
  verification, and other Rust Merkle libraries. Criterion reports mean, median, and standard
  deviation to filter out noise. For the latest CI numbers with links to every Criterion report, see
  the [live benchmark explorer](/benchmarks).

  :::note
  The tables below are copied from [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md)
  (run date 2026-08-17). That run predates the v0.5.0 sparse-tree and Patricia-trie
  performance fixes, which cut 4,000 sequential sparse inserts from about 223 s to 0.2 s and
  4,000 Patricia inserts from about 5.9 s to 0.02 s. Sparse and Patricia numbers here are therefore
  pessimistic. Cells marked *not captured* were not completed in that run. The comparison with
  `rs-merkle` and `merkle_light` was rerun on 2026-11-06 for 0.5.2.
  :::
</Rust>

<TypeScript>
  The TypeScript packages are compared with other JavaScript Merkle tree libraries:
  [`@openzeppelin/merkle-tree`](https://github.com/OpenZeppelin/merkle-tree),
  [`merkletreejs`](https://github.com/merkletreejs/merkletreejs),
  [`@aztec/merkle-tree`](https://www.npmjs.com/package/@aztec/merkle-tree), and
  [`@aztec/world-state`](https://www.npmjs.com/package/@aztec/world-state). The suite lives in
  [`bench/`](https://github.com/1888-Labs/MerkleForge-ts/tree/main/bench) in the MerkleForge-ts
  repository. All numbers come from one run on one machine, so read them as ratios, not absolutes.
  The same results are on the [benchmarks page](/benchmarks#typescript).

  :::note
  The tables below are copied from
  [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge-ts/blob/main/BENCHMARKS.md) in the
  MerkleForge-ts repository (MerkleForge 0.2.3, run date 2026-11-01).
  :::
</TypeScript>

## Faster than what?

<Rust>
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

  Full tables and caveats: [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md#faster-than-what) in the Rust repository.
</Rust>

<TypeScript>
  Every TypeScript speed ratio names its baseline. There are three, and they are never mixed:

  | Baseline | What is compared | Headline (131,072 leaves) |
  | --- | --- | --- |
  | **Another JavaScript library** | `@merkleforge/openzeppelin` against [`@openzeppelin/merkle-tree`](https://github.com/OpenZeppelin/merkle-tree) 1.0.8, the library it replaces, on the same leaves in the same run | builds 8–15x faster across sizes; 12.3x for an `[address, uint256]` airdrop tree (1,708 ms vs 21,021 ms), with byte-for-byte identical output |
  | **The previous MerkleForge release** | 0.2.3 against 0.2.2, same harness and machine | `BinaryMerkleTree` with `@merkleforge/hash-wasm` builds 5x faster (485 ms to 98 ms) |
  | **A Keccak-256 implementation** | `keccak256` from `@merkleforge/hash` against `keccak_256` from `@noble/hashes` | about 2x faster per hash, identical digests |

  **There is no speed claim against the Rust crates.** The TypeScript packages are a port of them and
  are checked against them for identical output, not raced against them.
</TypeScript>

## Parity: same bytes first

<Rust>
  The Rust crates are the reference implementation. The TypeScript packages are checked against them
  byte for byte, live, in the TypeScript repository's CI: roots and the bincode bytes of every
  sampled proof for Keccak-256 and SHA-256 trees of up to 131,072 leaves, plus generated test vectors
  for every tree, proof, and codec. Switch to TypeScript above for the full table.
</Rust>

<TypeScript>
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
</TypeScript>

## Environment

<Rust>
  | | |
  | --- | --- |
  | CPU | 11th Gen Intel Core i7-1185G7 @ 3.00 GHz, 4 cores / 8 threads, x86_64 |
  | RAM | 15 GiB |
  | OS | Linux 7.0.0-28-generic x86_64 |
  | Rust | rustc 1.97.1 (8bab26f4f 2026-07-14) |
  | Date | 2026-08-17 |
</Rust>

<TypeScript>
  | | |
  | --- | --- |
  | CPU | 11th Gen Intel Core i7-1185G7 @ 3.00 GHz |
  | OS | Linux 7.0.0-38-generic |
  | Node.js | v24.15.0 |
  | Date | 2026-11-01 (MerkleForge 0.2.3) |
</TypeScript>

## What is measured

<Rust>
  Each table below names the Criterion benchmark that produced it. Latency is the time for one
  operation; construction builds a fresh tree from the given number of leaves.
</Rust>

<TypeScript>
  Each library builds a tree from the same random 32-byte leaves, generates proofs for the same
  2,000 random leaves, and verifies every proof (a run fails if any proof does not verify). Build
  time is the median of 3 builds; proof and verify times are the mean per operation after a warm-up.
  Database setup (Aztec) is not counted.

  Sizes are powers of two, so the fixed-depth Aztec trees are full and have the same depth as the
  other trees, except `@aztec/world-state`, whose note hash tree is always 42 levels deep.

  What differs between libraries, so the numbers are not like for like:

  * **Hash.** OpenZeppelin, merkletreejs, and one Aztec `StandardTree` run use Keccak-256 from
    `@noble/hashes`; MerkleForge uses its own pure-JS Keccak-256 (identical digests, see
    [Parity](#parity-same-bytes-first)) or WebAssembly where noted. The other Aztec runs use Poseidon2,
    a SNARK-friendly hash that costs more per call in JavaScript.
  * **Storage.** The Aztec trees keep nodes in LMDB and support commit, rollback, and snapshots.
    `@aztec/world-state` is Aztec's native C++ rollup state. The other libraries keep the tree in
    memory.
  * **Verification.** OpenZeppelin, merkletreejs, and MerkleForge verify with their own functions.
    The Aztec trees have no verifier on the tree, so their proofs are checked with Aztec's
    `computeRootFromSiblingPath` and Aztec's synchronous Poseidon2. For world-state that means 42
    Poseidon2 hashes in JavaScript per proof, which is why its verify column is high.
  * **Proof generation in the OpenZeppelin-compatible trees** re-verifies every proof before
    returning it (both libraries do this), so it is slower than in `BinaryMerkleTree`.
  * **Leaf hashing.** `BinaryMerkleTree` hashes every leaf with a domain prefix (`H(0x00 || leaf)`),
    which protects against second-preimage attacks. That is twice the hashing of merkletreejs, which
    uses leaves as nodes directly.
</TypeScript>

## Results

<Rust>
  **Hash function throughput** · `cargo bench --bench hash_throughput`

  | Algorithm | 32 B latency | 64 B latency | 1 MB throughput |
  | --- | --- | --- | --- |
  | SHA-256 | 109.60 ns | 177.40 ns | 1.28 GiB/s |
  | Keccak-256 | 1.23 µs | 660.99 ns | 291.12 MiB/s |
  | BLAKE3 | 230.70 ns | 247.83 ns | 5.99 GiB/s |

  **Binary Merkle tree** · `cargo bench --bench binary_tree`

  | Leaves | Construction (SHA-256) | Construction (BLAKE3) | Proof gen | Proof verify |
  | --- | --- | --- | --- | --- |
  | 100 | 147.19 µs | 140.96 µs | 45.22 ns | 1.09 µs |
  | 1,000 | 1.80 ms | 1.83 ms | 57.21 ns | 1.87 µs |
  | 10,000 | 30.53 ms | 39.07 ms | 74.18 ns | 2.06 µs |
  | 100,000 | 268.23 ms | 274.93 ms | 78.69 ns | 2.95 µs |

  Construction builds a fresh `BinaryMerkleTree<H>`. Proof generation uses `LeafIndex(0)`. Proof
  verification checks the proof against the recorded root and the original leaf bytes.

  **Sparse Merkle tree** · `cargo bench --bench sparse_tree` (SHA-256)

  | Existing keys | Insert | Proof gen | Proof verify | Non-membership proof | Non-membership verify |
  | --- | --- | --- | --- | --- | --- |
  | 10 | 189.85 µs | 244.18 µs | 91.43 µs | 268.20 µs | 74.38 µs |
  | 100 | 2.11 ms | 2.25 ms | 75.74 µs | 2.32 ms | 76.86 µs |
  | 1,000 | 20.24 ms | 20.32 ms | 81.58 µs | 20.12 ms | 71.28 µs |
  | 10,000 | 181.05 ms | 198.84 ms | 63.00 µs | 203.49 ms | 76.41 µs |

  **Sparse Merkle tree, batch updates**

  | Batch size | Batch insert | Sequential insert | Speedup |
  | --- | --- | --- | --- |
  | 10 | 288.97 µs | 1.53 ms | 5.30× |
  | 100 | 2.08 ms | 88.12 ms | 42.46× |
  | 1,000 | 20.33 ms | 8.69 s | 427.72× |

  **Merkle Patricia trie** · `cargo bench --bench patricia_trie` (Keccak-256)

  | Existing entries | Insert | Get | Proof gen | Proof verify |
  | --- | --- | --- | --- | --- |
  | 10 | 20.61 µs | 159.00 ns | not captured | not captured |
  | 100 | 217.63 µs | 200.07 ns | not captured | not captured |
  | 1,000 | 2.11 ms | 270.26 ns | not captured | not captured |
  | 10,000 | not captured | 387.44 ns | not captured | not captured |

  State-root timings (`patricia_trie/state_root`) were not captured in this run.

  **Energy-aware construction** · `cargo bench --bench bench_energy` builds a 100,000-leaf binary
  tree.

  | Algorithm | 100K construction time | CPU cycles | Package energy |
  | --- | --- | --- | --- |
  | SHA-256 | 463.52 ms | not captured | not captured |
  | Keccak-256 | 1.49 s | not captured | not captured |
  | BLAKE3 | 261.02 ms | not captured | not captured |

  These are Criterion wall-clock numbers only. See [Profiling & energy](/docs/performance/profiling)
  to collect RAPL counters.

  **MerkleForge, `rs-merkle`, and `merkle_light`** · `cargo bench -p merkle-bench --bench comparison` (2026-11-06)

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
</Rust>

<TypeScript>
  **bytes32 leaves, 1,024 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (MerkleForge JS) | 6.0 | 1.9 | 28.9 |
  | @merkleforge/variants BinaryMerkleTree + @merkleforge/hash-wasm | Keccak-256 (MerkleForge WASM SIMD) | 1.8 | 3.8 | 8.9 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (MerkleForge JS) | 5.6 | 30.1 | 29.3 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 70.2 | 178 | 161 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 9.0 | 1.4 | 61.2 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 23.9 | 19.1 | 105 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 38.5 | 20.3 | 218 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 16.5 | 40.8 | 1062 |

  **bytes32 leaves, 16,384 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (MerkleForge JS) | 93.3 | 2.1 | 47.0 |
  | @merkleforge/variants BinaryMerkleTree + @merkleforge/hash-wasm | Keccak-256 (MerkleForge WASM SIMD) | 13.1 | 3.0 | 13.4 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (MerkleForge JS) | 95.7 | 53.0 | 55.2 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 1435 | 308 | 271 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 141 | 2.0 | 117 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 369 | 36.4 | 147 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 639 | 43.5 | 356 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 255 | 51.9 | 1000 |

  **bytes32 leaves, 131,072 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (MerkleForge JS) | 637 | 2.3 | 52.4 |
  | @merkleforge/variants BinaryMerkleTree + @merkleforge/hash-wasm | Keccak-256 (MerkleForge WASM SIMD) | 97.6 | 3.3 | 17.9 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (MerkleForge JS) | 825 | 56.3 | 71.7 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 10493 | 312 | 292 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 935 | 4.1 | 127 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 2668 | 53.8 | 177 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 4919 | 56.7 | 393 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 2014 | 57.6 | 1010 |

  **OpenZeppelin `[address, uint256]` leaves, 1,024 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (MerkleForge JS) | 15.7 | 43.1 | 45.3 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 131 | 234 | 229 |

  **OpenZeppelin `[address, uint256]` leaves, 16,384 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (MerkleForge JS) | 196 | 53.6 | 55.1 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 1960 | 283 | 270 |

  **OpenZeppelin `[address, uint256]` leaves, 131,072 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (MerkleForge JS) | 1708 | 74.8 | 78.7 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 21021 | 347 | 344 |

  Node v24.15.0, 11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz, linux 7.0.0-38-generic. Build is the median of 3 runs; proof and verify are the mean over 2000 random leaves.

  **Keccak-256 per call**

  Plain Keccak-256 (`keccak256` from `@merkleforge/hash`, no domain prefix)
  against `keccak_256` from `@noble/hashes` 2.4.0, on the input sizes trees hash:
  32-byte leaves, 33-byte prefixed leaves, 64- and 65-byte node pairs, and one
  full 136-byte block. Both produce the same digests; the script checks this
  before timing.

  | Input | @noble/hashes keccak_256 | @merkleforge/hash keccak256 | Speedup |
  |---:|---:|---:|---:|
  | 32 B | 7.00 µs | 3.25 µs | 2.2x |
  | 33 B | 8.86 µs | 3.32 µs | 2.7x |
  | 64 B | 7.85 µs | 3.37 µs | 2.3x |
  | 65 B | 7.57 µs | 3.43 µs | 2.2x |
  | 136 B | 14.44 µs | 5.30 µs | 2.7x |

  Node v24.15.0, 11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz, linux 7.0.0-38-generic. Median of 5 rounds of 200,000 calls per size.

  **Install footprint**

  Dependencies installed with each library, not counting the library itself,
  and the total size of the library plus its dependencies.

  | Library | Dependencies | Size |
  |---|---:|---:|
  | `@merkleforge/core` + `hash` + `variants` | 1 (`@noble/hashes`) | 1.4 MB |
  | `@merkleforge/openzeppelin` | 3 (`@merkleforge/core`, `@merkleforge/hash`, `@noble/hashes`) | 1.1 MB |
  | `merkletreejs` 0.6.0 | 3 | 1 MB |
  | `@openzeppelin/merkle-tree` 1.0.8 | 38 | 16 MB |
  | `@aztec/merkle-tree` 4.4.0 | 362 | 395 MB |
  | `@aztec/world-state` 5.2.1 | 377 | 1.1 GB |

  The Aztec packages bring Aztec's prover (`bb.js`), native bindings, LMDB, an
  HTTP server, and Ethereum tooling. They are built for running an Aztec node,
  not as general-purpose tree libraries.
</TypeScript>

## Reproduce

<Rust>
  ```bash
  git clone https://github.com/1888-Labs/MerkleForge.git
  cd MerkleForge

  # Everything (long: the Patricia insert group alone can take many minutes)
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

  # Print comparative proof sizes
  MERKLEFORGE_PRINT_COMPARISON_SIZES=1 cargo bench --bench comparison -- --list
  ```

  Record the machine, kernel, CPU governor, Rust version, and command next to any numbers you
  publish.
</Rust>

<TypeScript>
  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  cd MerkleForge-ts
  pnpm install && pnpm build   # in the repo root
  cd bench
  npm install                  # comparison libraries, kept out of the pnpm workspace
  npm run bench                # writes results.json; SIZES, OPS, and BUILD_RUNS env vars adjust the run
  node report.mjs              # prints the tables above
  npm run bench:keccak         # Keccak-256 per call against @noble/hashes
  npm run parity               # byte-for-byte parity; needs the Rust repo cloned next to MerkleForge-ts
  ```

  The 0.2.2 comparison reruns the same harness at the `v0.2.2` tag; the steps are in
  [BENCHMARKS.md](https://github.com/1888-Labs/MerkleForge-ts/blob/main/BENCHMARKS.md#reproduce).

  Record the machine, Node.js version, and command next to any numbers you publish.
</TypeScript>

## Reading the tables

* **Latency:** time for one operation.
* **Throughput:** bytes or elements processed per second.
* **Proof generation:** time to produce the witness sent to a verifier.
* **Proof verification:** time for a stateless client to check the witness against a trusted root.
* **Proof size:** bytes sent over the network or stored as evidence.
