<div className="mf-page" style={{ paddingTop: 0 }}>
  # Benchmarks

  Rust and TypeScript are measured separately: the Rust crates with Criterion on every site build,
  the TypeScript packages against other JavaScript Merkle tree libraries. Lower is better.

  ## Rust

  Criterion results from the latest site build, across tree variants, operations, hash adapters, and
  sizes. Every row links to its full Criterion report.

  <BenchmarkExplorer />

  ### MerkleForge vs `rs-merkle` vs `merkle_light`

  From the published comparison run (`cargo bench --bench comparison`, i7-1185G7, 2026-08-17):

  | Metric | MerkleForge | `rs-merkle` | `merkle_light` |
  | --- | --- | --- | --- |
  | 10K leaf construction | 22.77 ms | 3.69 ms | 2.29 ms |
  | Proof size (10K tree) | 469 bytes | 448 bytes | 512 bytes |
  | Proof verification | 1.84 µs | 6.39 µs | 1.78 µs |

  Construction uses deterministic 32-byte leaves for every library; `rs-merkle` receives pre-hashed
  SHA-256 leaves, matching its API. Proof size is each library's serialized proof byte count.

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

  ### At 131,072 leaves

  | Task | MerkleForge | Others |
  | --- | --- | --- |
  | Build a Keccak-256 tree | **485 ms** with `hash-wasm`, 2,119 ms pure JS | merkletreejs 1,544 ms · Aztec StandardTree 2,669 ms · OpenZeppelin 14,518 ms |
  | Build an OpenZeppelin airdrop tree (`[address, uint256]`) | **3,070 ms** (`@merkleforge/openzeppelin`) | OpenZeppelin 15,197 ms |
  | Generate one proof | **8–11 µs** (`BinaryMerkleTree`) | merkletreejs 37 µs · Aztec 51–53 µs · OpenZeppelin 743 µs |
  | Verify one proof | **41 µs** with `hash-wasm`, 148 µs pure JS | Aztec StandardTree 168 µs · OpenZeppelin 474 µs · merkletreejs 538 µs |
  | Install footprint | **1–2 dependencies, about 1.4 MB** | merkletreejs 3, 1 MB · OpenZeppelin 38, 16 MB · Aztec merkle-tree 362, 395 MB |

  * `@merkleforge/openzeppelin` produces the same roots and proofs as `@openzeppelin/merkle-tree`
    while building trees 4–9x faster, generating proofs 2–4x faster, and verifying about 2x faster.
  * With pure-JavaScript Keccak, `BinaryMerkleTree` builds about 1.4x slower than merkletreejs
    because it hashes every leaf with a domain prefix (`H(0x00 ‖ leaf)`), which blocks
    second-preimage attacks. With `@merkleforge/hash-wasm` it is 3.2x faster.
  * The Aztec trees use LMDB storage and, in some runs, Poseidon2, so they are not like for like.

  Per-size tables (1,024, 16,384, and 131,072 leaves), the exact method, and every caveat are in
  [BENCHMARKS.md](https://github.com/1888-Labs/MerkleForge-ts/blob/main/BENCHMARKS.md).

  ### Reproduce

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  cd MerkleForge-ts
  pnpm install && pnpm build
  cd bench
  npm install        # comparison libraries, kept out of the pnpm workspace
  npm run bench      # writes results.json; SIZES, OPS, and BUILD_RUNS adjust the run
  node report.mjs    # prints the tables
  ```
</div>
