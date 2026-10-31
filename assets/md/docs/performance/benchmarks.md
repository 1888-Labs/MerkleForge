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
  pessimistic. Cells marked *not captured* were not completed in that run.
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
  MerkleForge-ts repository (run date 2026-10-18).
  :::
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
  | Date | 2026-10-18 |
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

  * **Hash.** MerkleForge, OpenZeppelin, merkletreejs, and one Aztec `StandardTree` run use
    Keccak-256 from `@noble/hashes` (or WebAssembly where noted). The other Aztec runs use Poseidon2,
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

  **MerkleForge, `rs-merkle`, and `merkle_light`** · `cargo bench --bench comparison`

  | Metric | MerkleForge | `rs-merkle` | `merkle_light` |
  | --- | --- | --- | --- |
  | 10K leaf construction | 22.77 ms | 3.69 ms | 2.29 ms |
  | Proof size (10K tree) | 469 bytes | 448 bytes | 512 bytes |
  | Proof verification | 1.84 µs | 6.39 µs | 1.78 µs |

  * Construction uses deterministic 32-byte leaves for all libraries.
  * `rs-merkle` receives pre-hashed SHA-256 leaves, matching its public API.
  * Proof size is the serialized proof byte count reported by each library.
</Rust>

<TypeScript>
  **bytes32 leaves, 1,024 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (noble) | 27.9 | 3.3 | 100 |
  | @merkleforge/variants BinaryMerkleTree + hash-wasm | Keccak-256 (WASM) | 6.9 | 2.2 | 27.3 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (noble) | 15.8 | 94.7 | 114 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 104 | 263 | 235 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 12.8 | 2.1 | 98.5 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 35.3 | 27.4 | 129 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 66.3 | 32.9 | 328 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 21.7 | 66.9 | 1257 |

  **bytes32 leaves, 16,384 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (noble) | 276 | 9.3 | 128 |
  | @merkleforge/variants BinaryMerkleTree + hash-wasm | Keccak-256 (WASM) | 65.3 | 5.5 | 31.7 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (noble) | 173 | 124 | 174 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 1631 | 328 | 306 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 144 | 3.1 | 120 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 429 | 44.4 | 165 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 701 | 50.9 | 403 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 313 | 74.2 | 1251 |

  **bytes32 leaves, 131,072 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/variants BinaryMerkleTree | Keccak-256 (noble) | 2119 | 10.8 | 148 |
  | @merkleforge/variants BinaryMerkleTree + hash-wasm | Keccak-256 (WASM) | 485 | 8.2 | 40.8 |
  | @merkleforge/openzeppelin SimpleMerkleTree | Keccak-256 (noble) | 1561 | 176 | 204 |
  | @openzeppelin/merkle-tree SimpleMerkleTree | Keccak-256 (noble) | 14518 | 743 | 474 |
  | merkletreejs MerkleTree | Keccak-256 (noble) | 1544 | 36.9 | 538 |
  | @aztec/merkle-tree StandardTree | Keccak-256 (noble) | 2669 | 51.9 | 167 |
  | @aztec/merkle-tree StandardTree | Poseidon2 (bb.js WASM) | 4672 | 50.7 | 392 |
  | @aztec/world-state note hash tree (native) | Poseidon2 (native C++), depth 42 | 2007 | 52.9 | 988 |

  **OpenZeppelin `[address, uint256]` leaves, 1,024 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (noble) | 28.4 | 85.8 | 103 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 125 | 230 | 220 |

  **OpenZeppelin `[address, uint256]` leaves, 16,384 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (noble) | 386 | 115 | 131 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 1874 | 289 | 271 |

  **OpenZeppelin `[address, uint256]` leaves, 131,072 leaves**

  | Library | Hash | Build (ms) | Proof (µs) | Verify (µs) |
  |---|---|---:|---:|---:|
  | @merkleforge/openzeppelin StandardMerkleTree | Keccak-256 (noble) | 3070 | 141 | 171 |
  | @openzeppelin/merkle-tree StandardMerkleTree | Keccak-256 (noble) | 15197 | 333 | 320 |

  Build is the median of 3 runs; proof and verify are the mean over 2000 random leaves.

  **Install footprint**

  Dependencies installed with each library, not counting the library itself, and the total size of
  the library plus its dependencies.

  | Library | Dependencies | Size |
  |---|---:|---:|
  | `@merkleforge/core` + `hash` + `variants` | 1 (`@noble/hashes`) | 1.4 MB |
  | `@merkleforge/openzeppelin` | 2 (`@merkleforge/core`, `@noble/hashes`) | 1.1 MB |
  | `merkletreejs` 0.6.0 | 3 | 1 MB |
  | `@openzeppelin/merkle-tree` 1.0.8 | 38 | 16 MB |
  | `@aztec/merkle-tree` 4.4.0 | 362 | 395 MB |
  | `@aztec/world-state` 5.2.1 | 377 | 1.1 GB |

  The Aztec packages bring Aztec's prover (`bb.js`), native bindings, LMDB, an HTTP server, and
  Ethereum tooling. They are built for running an Aztec node, not as general-purpose tree libraries.
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
  ```

  Record the machine, Node.js version, and command next to any numbers you publish.
</TypeScript>

## Reading the tables

* **Latency:** time for one operation.
* **Throughput:** bytes or elements processed per second.
* **Proof generation:** time to produce the witness sent to a verifier.
* **Proof verification:** time for a stateless client to check the witness against a trusted root.
* **Proof size:** bytes sent over the network or stored as evidence.
