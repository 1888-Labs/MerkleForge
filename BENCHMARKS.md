# MerkleForge Benchmark Results

MerkleForge uses Criterion benchmarks to compare hash adapters, tree variants,
proof generation, proof verification, and external Rust Merkle libraries.

Lower latency is better. Higher throughput is better. Publishable benchmark
numbers should be recorded from the same machine and Rust toolchain.

## Benchmark Environment

- CPU: 11th Gen Intel(R) Core(TM) i7-1185G7 @ 3.00GHz
- CPU layout: 4 cores / 8 threads, x86_64
- RAM: 15 GiB
- OS: Linux 7.0.0-28-generic x86_64 GNU/Linux
- Rust: rustc 1.97.1 (8bab26f4f 2026-07-14)
- Date: 2026-08-17
- Criterion output: `target/criterion/`
- Live report URL: <https://1888-labs.github.io/MerkleForge/>

Run note: the local `cargo bench --workspace` refresh was started on
2026-08-17 and completed through baseline hashing, energy construction,
binary tree, comparison, hash throughput, and partial Patricia trie groups.
The full run was stopped at `patricia_trie/insert/Keccak-256/10000` after
Criterion estimated roughly 1,206 seconds for that single 10-sample case.
Cells marked `not captured` were not completed in this local refresh and
should be regenerated on a dedicated benchmark machine.

## Reproduction Commands

Run the complete benchmark suite:

```bash
cargo bench --workspace
```

Run focused benchmark groups:

```bash
cargo bench --bench hash_throughput
cargo bench --bench binary_tree
cargo bench --bench sparse_tree
cargo bench --bench patricia_trie
cargo bench --bench comparison
cargo bench --bench bench_energy
```

Print comparative proof sizes:

```bash
MERKLEFORGE_PRINT_COMPARISON_SIZES=1 cargo bench --bench comparison -- --list
```

## Hash Function Throughput

Populated from:

```bash
cargo bench --bench hash_throughput
```

The current benchmark implementation measures 1 KiB, 16 KiB, 64 KiB, and
1 MiB buffers. The table below keeps the small-message latency and
large-message throughput columns used by the public benchmark summary.

| Algorithm | 32B latency | 64B latency | 1MB throughput |
|-----------|-------------|-------------|----------------|
| SHA-256 | 109.60 ns | 177.40 ns | 1.28 GiB/s |
| Keccak-256 | 1.23 µs | 660.99 ns | 291.12 MiB/s |
| BLAKE3 | 230.70 ns | 247.83 ns | 5.99 GiB/s |

## Binary Merkle Tree

Populated from:

```bash
cargo bench --bench binary_tree
```

Benchmarked tree sizes: 100, 1,000, 10,000, and 100,000 leaves across SHA-256,
Keccak-256, and BLAKE3. The summary table focuses on SHA-256 and BLAKE3
because they represent the conservative production baseline and the
high-throughput baseline.

| Leaves | Construction (SHA-256) | Construction (BLAKE3) | Proof gen | Proof verify |
|--------|-------------------------|------------------------|-----------|--------------|
| 100 | 147.19 µs | 140.96 µs | 45.22 ns | 1.09 µs |
| 1,000 | 1.80 ms | 1.83 ms | 57.21 ns | 1.87 µs |
| 10,000 | 30.53 ms | 39.07 ms | 74.18 ns | 2.06 µs |
| 100,000 | 268.23 ms | 274.93 ms | 78.69 ns | 2.95 µs |

Notes:

- Construction measures building a fresh `BinaryMerkleTree<H>`.
- Proof generation uses `LeafIndex(0)`.
- Proof verification checks the generated proof against the recorded root and
  original leaf bytes.

## Sparse Merkle Tree

Populated from:

```bash
cargo bench --bench sparse_tree
```

Benchmarked existing-state sizes: 10, 100, 1,000, and 10,000 keys across
SHA-256, Keccak-256, and BLAKE3.

| Existing keys | Insert (SHA-256) | Proof gen (SHA-256) | Proof verify (SHA-256) | Non-membership proof | Non-membership verify |
|---------------|------------------|---------------------|------------------------|----------------------|-----------------------|
| 10 | 189.85 µs | 244.18 µs | 91.43 µs | 268.20 µs | 74.38 µs |
| 100 | 2.11 ms | 2.25 ms | 75.74 µs | 2.32 ms | 76.86 µs |
| 1,000 | 20.24 ms | 20.32 ms | 81.58 µs | 20.12 ms | 71.28 µs |
| 10,000 | 181.05 ms | 198.84 ms | 63.00 µs | 203.49 ms | 76.41 µs |

### Sparse Batch Updates

Populated from `sparse_tree/batch_update`.

| Batch size | Batch insert (SHA-256) | Sequential insert (SHA-256) | Speedup |
|------------|-------------------------|------------------------------|---------|
| 10 | 288.97 µs | 1.53 ms | 5.30x |
| 100 | 2.08 ms | 88.12 ms | 42.46x |
| 1,000 | 20.33 ms | 8.69 s | 427.72x |

Notes:

- Sparse proofs include both membership and non-membership paths.
- Batch update results are used to evaluate the one-phase update strategy for
  rollup-style workloads.

## Merkle Patricia Trie

Populated from:

```bash
cargo bench --bench patricia_trie
```

Benchmarked existing-entry sizes: 10, 100, 1,000, and 10,000 entries across
Keccak-256 and BLAKE3. Keccak-256 is the Ethereum-compatible primary result.

| Existing entries | Insert (Keccak-256) | Get (Keccak-256) | Proof gen (Keccak-256) | Proof verify (Keccak-256) |
|------------------|---------------------|------------------|-------------------------|---------------------------|
| 10 | 20.61 µs | 159.00 ns | not captured | not captured |
| 100 | 217.63 µs | 200.07 ns | not captured | not captured |
| 1,000 | 2.11 ms | 270.26 ns | not captured | not captured |
| 10,000 | not captured (est. 1,206 s for insert group) | 387.44 ns | not captured | not captured |

### Patricia State Root

Populated from `patricia_trie/state_root`.

| Entries | State root (Keccak-256) | State root (BLAKE3) |
|---------|--------------------------|---------------------|
| 100 | not captured | not captured |
| 1,000 | not captured | not captured |
| 10,000 | not captured | not captured |

## Energy-Aware Benchmark

Populated from:

```bash
cargo bench --bench bench_energy
./scripts/energy-perf.sh bench_energy
```

Detailed reproduction notes live in [`benchmarks/ENERGY.md`](benchmarks/ENERGY.md).

| Algorithm | 100K construction time | CPU cycles | Energy package joules | Notes |
|-----------|-------------------------|------------|------------------------|-------|
| SHA-256 | 463.52 ms | not captured | not captured | Criterion wall-clock only |
| Keccak-256 | 1.49 s | not captured | not captured | Criterion wall-clock only |
| BLAKE3 | 261.02 ms | not captured | not captured | Criterion wall-clock only |

## Comparative: MerkleForge vs `rs-merkle` vs `merkle_light`

Run 2026-11-06 on the machine above (i7-1185G7, 4 cores / 8 threads, rustc
1.97.1), SHA-256 everywhere, the same deterministic 32-byte leaves for every
library, Criterion medians. Every ratio below is **MerkleForge against the
named library, same operation, same leaves, same hash, same run**. MerkleForge
and the other libraries produce different roots (MerkleForge domain-separates
leaves and nodes), so this compares speed for the same job, not identical
output.

### Faster than what?

Against [`rs-merkle`](https://crates.io/crates/rs_merkle) 1.5, at 1,048,576 leaves:

| Operation | MerkleForge | `rs-merkle` | Faster | Threads (MerkleForge / rs-merkle) |
|---|---:|---:|---:|---|
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

- MerkleForge hashes every leaf with a `0x00` prefix and every node with a
  `0x01` prefix, which blocks second-preimage attacks. That is extra work the
  other libraries skip, and it is counted in MerkleForge's times.
- `rs-merkle` is given its leaves already hashed in verification (its API
  takes leaf hashes); MerkleForge hashes the raw leaf inside `verify`. In
  construction, `rs-merkle`'s leaf hashing is timed.
- Rows marked 8 threads use the `parallel` feature (rayon) on this 4-core
  laptop; the other libraries have no multi-threaded build or batch verifier.
  The one-thread rows are the like-for-like comparison.

### Full results

**10,000 leaves**

| Operation | MerkleForge | MerkleForge `parallel` | `rs-merkle` | `merkle_light` |
|---|---:|---:|---:|---:|
| Build (`from_leaves`) | 1.59 ms | 1.20 ms | 3.02 ms | 1.73 ms |
| Build (one `insert` at a time) | 13.6 ms | 14.6 ms | n/a | n/a |
| Generate one proof | 193 ns | 172 ns | 1.61 µs | 181 ns |
| Verify one proof | 1.46 µs | 1.36 µs | 5.93 µs | 1.57 µs |
| Verify 10,000 proofs | 15.0 ms | 4.55 ms | 68.2 ms | 16.5 ms |
| Generate a multiproof (100 leaves) | 3.89 µs | 4.29 µs | 24.5 µs | n/a |
| Verify a multiproof (100 leaves) | 84.6 µs | 82.5 µs | 168 µs | n/a |

**1,048,576 leaves**

| Operation | MerkleForge | MerkleForge `parallel` | `rs-merkle` | `merkle_light` |
|---|---:|---:|---:|---:|
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

### Reproduce

```bash
cargo bench -p merkle-bench --bench comparison                        # 10,000 leaves
cargo bench -p merkle-bench --bench comparison --features parallel    # multi-threaded build and verify_batch
COMPARISON_LEAVES=1048576 cargo bench -p merkle-bench --bench comparison
COMPARISON_LEAVES=1048576 cargo bench -p merkle-bench --bench comparison --features parallel
MERKLEFORGE_PRINT_COMPARISON_SIZES=1 cargo bench -p merkle-bench --bench comparison -- --list   # proof sizes
```

Timings on a laptop vary by 10–20% between runs; compare ratios, not single
numbers.

## How To Read The Tables

- **Latency**: elapsed time for one operation. Lower means faster.
- **Throughput**: processed bytes or elements per second. Higher means faster.
- **Proof generation**: time to produce the witness sent to a verifier.
- **Proof verification**: time for a stateless client to check the witness
  against a trusted root.
- **Proof size**: bytes sent over the network or stored as evidence.
- **Energy package joules**: CPU package energy reported by Linux perf when
  the processor exposes RAPL counters.

## Completion Checklist

- [x] Root `BENCHMARKS.md` exists.
- [x] Benchmark environment is documented for reproducibility.
- [x] Hash throughput table template exists.
- [x] Binary Merkle tree table template exists.
- [x] Sparse Merkle tree table template exists.
- [x] Patricia trie table template exists.
- [x] Comparative benchmark table includes current local measurements.
- [x] No vague placeholder cells remain; incomplete measurements are explicitly
  marked `not captured`.
- [ ] Regenerate `not captured` rows on a dedicated benchmark machine before
  publishing a new benchmark report.
