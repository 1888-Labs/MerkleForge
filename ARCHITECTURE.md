# MerkleForge Architecture

How the workspace fits together, and where to make a change. For usage, see
the [documentation site](https://1888-labs.github.io/MerkleForge/docs/getting-started).

## Crates

The framework is split into a small core and the pieces that build on it:

| Crate (folder) | Published as | Role |
|----------------|--------------|------|
| `merkle-core/` | `merkle-core` | Traits, shared types, proofs, and `MerkleError`. No tree logic. |
| `merkle-hash/` | `merkleforge-hash` | `Sha256`, `Keccak256`, and `Blake3` adapters implementing `HashFunction`. |
| `merkle-variants/` | `merkle-variants` | `BinaryMerkleTree`, `SparseMerkleTree`, and `MerklePatriciaTrie`. |
| `merkle-bench/` | not published | Criterion benchmarks, isolated from the library crates. |

`merkle-hash` is published as `merkleforge-hash` because the original name was
already taken on crates.io.

### Module organization

<img width="1698" height="1252" alt="MerkleForge module organization" src="https://github.com/user-attachments/assets/7c3e4c98-8e61-479b-aee8-510ae2b231a1" />

### Trait hierarchy

<img width="1253" height="1158" alt="MerkleForge trait hierarchy" src="https://github.com/user-attachments/assets/cd1afa6c-1aaf-4d43-bb72-18467ce73726" />

Every tree implements `MerkleTree<H: HashFunction>` and `ProofVerifier`, so
integrations can switch variants or hashes by changing a type parameter.
Generics are monomorphised at compile time, so there is no runtime dispatch
cost. The whole workspace is `#![forbid(unsafe_code)]`.

## Workspace layout

```
MerkleForge/
├── Cargo.toml                    # workspace manifest + shared dependency versions
├── merkle-core/src/
│   ├── error.rs                  # MerkleError (non_exhaustive)
│   ├── traits/
│   │   ├── hash_function.rs      # HashFunction
│   │   ├── merkle_tree.rs        # MerkleTree + ProofVerifier
│   │   └── serializable.rs       # blanket serde/bincode impl
│   └── types/common.rs           # NodeIndex, LeafIndex, MerkleProof, ...
├── merkle-hash/src/              # sha256.rs, keccak256.rs, blake3.rs
├── merkle-variants/
│   ├── src/                      # binary.rs, sparse.rs, patricia.rs
│   ├── tests/                    # property tests, trait contract, Ethereum vectors
│   └── examples/light_client.rs  # stateless verification demo
├── merkle-bench/benches/         # one Criterion file per benchmark group
├── docs/                         # Vocs website: docs, examples, demo, benchmarks
├── scripts/                      # site build, flamegraph, and energy helpers
├── profiling/                    # profiling guide + reference flamegraph
├── assets/brand/                 # official logos, icons, banners, favicons
└── .github/workflows/            # CI, website, release, notifications
```

## Hashing and domain separation

Leaves and internal nodes are hashed in separate domains:

- Leaf: `H(0x00 || data)`
- Internal node: `H(0x01 || left || right)`

This prevents second-preimage attacks where an internal node is passed off as
a leaf. SHA-256 and Keccak-256 use the byte prefixes above. BLAKE3 uses its
native `derive_key` mode with context strings instead.

## Tree variants

- **Binary Merkle tree**: balanced, power-of-two tree built bottom-up
  iteratively. Best when the full leaf set is known up front. Proofs are
  `O(log n)`.
- **Sparse Merkle tree**: 256-bit key space with shortcut nodes (Buterin,
  2018), precomputed empty-subtree hashes, node batching (Ouvrard), and
  one-phase batch updates (Ma et al.) for rollup workloads.
- **Merkle Patricia trie**: Ethereum-compatible trie (branch, extension, leaf,
  empty nodes) with RLP encoding. Its state roots are checked against the
  official Ethereum test vectors.

## Proofs and stateless verification

A `MerkleProof<D>` carries the sibling hashes from a leaf up to the root:

```rust
pub struct MerkleProof<D> {
    pub leaf_index: LeafIndex,
    pub leaf_count: usize,
    pub path:       Vec<ProofNode<D>>,   // O(log n) siblings
}
```

A verifier needs only a trusted root, the leaf bytes, and the proof:

```
1. current = H(0x00 || leaf_data)
2. for each ProofNode in proof.path:
     if side == Left:  current = H(0x01 || sibling || current)
     if side == Right: current = H(0x01 || current || sibling)
3. return current == expected_root
```

Proofs are `serde`-serialisable through the blanket `Serializable` impl.
`cargo run -p merkle-variants --example light_client` shows the full
server/client split.

## Where to make a change

| You want to... | Start in |
|----------------|----------|
| Add or change a public trait or shared type | `merkle-core/src/traits/`, `merkle-core/src/types/` (open an issue first) |
| Add a hash function | a new file in `merkle-hash/src/`, implementing `HashFunction` |
| Fix or optimise a tree | `merkle-variants/src/<variant>.rs` and its tests in `merkle-variants/tests/` |
| Add test vectors | `merkle-variants/tests/fixtures/` |
| Add a benchmark | `merkle-bench/benches/`, then record results in [`BENCHMARKS.md`](BENCHMARKS.md) |
| Improve the docs or website | `docs/pages/` (MDX) and `docs/components/` |
| Change CI or releases | `.github/workflows/` |
