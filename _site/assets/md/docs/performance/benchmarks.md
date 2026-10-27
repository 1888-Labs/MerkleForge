# Benchmarks

MerkleForge uses [Criterion](https://crates.io/crates/criterion) benchmarks in the unpublished
`merkle-bench` crate to compare hash adapters, tree variants, proof generation, proof
verification, and other Rust Merkle libraries. Criterion reports mean, median, and standard
deviation to filter out noise.

Lower latency is better; higher throughput is better. For the latest CI numbers with links to every
Criterion report, see the [live benchmark explorer](/benchmarks).

:::note
The tables below are copied from [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md)
(run date 2026-08-17). That run predates the **Unreleased** sparse-tree and Patricia-trie
performance fixes, which cut 4,000 sequential sparse inserts from about 223 s to 0.2 s and
4,000 Patricia inserts from about 5.9 s to 0.02 s. Sparse and Patricia numbers here are therefore
pessimistic. Cells marked *not captured* were not completed in that run.
:::

## Environment

| | |
| --- | --- |
| CPU | 11th Gen Intel Core i7-1185G7 @ 3.00 GHz, 4 cores / 8 threads, x86_64 |
| RAM | 15 GiB |
| OS | Linux 7.0.0-28-generic x86_64 |
| Rust | rustc 1.97.1 (8bab26f4f 2026-07-14) |
| Date | 2026-08-17 |

## Hash function throughput

`cargo bench --bench hash_throughput`

| Algorithm | 32 B latency | 64 B latency | 1 MB throughput |
| --- | --- | --- | --- |
| SHA-256 | 109.60 ns | 177.40 ns | 1.28 GiB/s |
| Keccak-256 | 1.23 µs | 660.99 ns | 291.12 MiB/s |
| BLAKE3 | 230.70 ns | 247.83 ns | 5.99 GiB/s |

## Binary Merkle tree

`cargo bench --bench binary_tree`

| Leaves | Construction (SHA-256) | Construction (BLAKE3) | Proof gen | Proof verify |
| --- | --- | --- | --- | --- |
| 100 | 147.19 µs | 140.96 µs | 45.22 ns | 1.09 µs |
| 1,000 | 1.80 ms | 1.83 ms | 57.21 ns | 1.87 µs |
| 10,000 | 30.53 ms | 39.07 ms | 74.18 ns | 2.06 µs |
| 100,000 | 268.23 ms | 274.93 ms | 78.69 ns | 2.95 µs |

Construction builds a fresh `BinaryMerkleTree<H>`. Proof generation uses `LeafIndex(0)`. Proof
verification checks the proof against the recorded root and the original leaf bytes.

## Sparse Merkle tree

`cargo bench --bench sparse_tree` (SHA-256)

| Existing keys | Insert | Proof gen | Proof verify | Non-membership proof | Non-membership verify |
| --- | --- | --- | --- | --- | --- |
| 10 | 189.85 µs | 244.18 µs | 91.43 µs | 268.20 µs | 74.38 µs |
| 100 | 2.11 ms | 2.25 ms | 75.74 µs | 2.32 ms | 76.86 µs |
| 1,000 | 20.24 ms | 20.32 ms | 81.58 µs | 20.12 ms | 71.28 µs |
| 10,000 | 181.05 ms | 198.84 ms | 63.00 µs | 203.49 ms | 76.41 µs |

### Batch updates

| Batch size | Batch insert | Sequential insert | Speedup |
| --- | --- | --- | --- |
| 10 | 288.97 µs | 1.53 ms | 5.30× |
| 100 | 2.08 ms | 88.12 ms | 42.46× |
| 1,000 | 20.33 ms | 8.69 s | 427.72× |

## Merkle Patricia trie

`cargo bench --bench patricia_trie` (Keccak-256)

| Existing entries | Insert | Get | Proof gen | Proof verify |
| --- | --- | --- | --- | --- |
| 10 | 20.61 µs | 159.00 ns | not captured | not captured |
| 100 | 217.63 µs | 200.07 ns | not captured | not captured |
| 1,000 | 2.11 ms | 270.26 ns | not captured | not captured |
| 10,000 | not captured | 387.44 ns | not captured | not captured |

State-root timings (`patricia_trie/state_root`) were not captured in this run.

## Energy-aware construction

`cargo bench --bench bench_energy` builds a 100,000-leaf binary tree.

| Algorithm | 100K construction time | CPU cycles | Package energy |
| --- | --- | --- | --- |
| SHA-256 | 463.52 ms | not captured | not captured |
| Keccak-256 | 1.49 s | not captured | not captured |
| BLAKE3 | 261.02 ms | not captured | not captured |

These are Criterion wall-clock numbers only. See [Profiling & energy](/docs/performance/profiling)
to collect RAPL counters.

## MerkleForge vs `rs-merkle` vs `merkle_light`

`cargo bench --bench comparison`

| Metric | MerkleForge | `rs-merkle` | `merkle_light` |
| --- | --- | --- | --- |
| 10K leaf construction | 22.77 ms | 3.69 ms | 2.29 ms |
| Proof size (10K tree) | 469 bytes | 448 bytes | 512 bytes |
| Proof verification | 1.84 µs | 6.39 µs | 1.78 µs |

* Construction uses deterministic 32-byte leaves for all libraries.
* `rs-merkle` receives pre-hashed SHA-256 leaves, matching its public API.
* Proof size is the serialized proof byte count reported by each library.

## Reproduce

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
publish. Results are hardware-specific: compare trends across machines, not absolute values.

## Reading the tables

* **Latency:** time for one operation.
* **Throughput:** bytes or elements processed per second.
* **Proof generation:** time to produce the witness sent to a verifier.
* **Proof verification:** time for a stateless client to check the witness against a trusted root.
* **Proof size:** bytes sent over the network or stored as evidence.
