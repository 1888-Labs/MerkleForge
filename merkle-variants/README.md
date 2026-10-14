<p align="center">
  <a href="https://github.com/1888-Labs/MerkleForge">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/1888-Labs/MerkleForge/main/assets/brand/svg/mark/merkleforge-mark-white.svg">
      <img alt="MerkleForge" src="https://raw.githubusercontent.com/1888-Labs/MerkleForge/main/assets/brand/svg/mark/merkleforge-mark-black.svg" width="72">
    </picture>
  </a>
</p>

<h1 align="center">merkle-variants</h1>

<p align="center">Binary, Sparse, and Ethereum-compatible Patricia Merkle trees<br>for MerkleForge, generic over any hash adapter.</p>

<p align="center">
  <a href="https://crates.io/crates/merkle-variants"><img alt="crates.io" src="https://img.shields.io/crates/v/merkle-variants.svg?style=flat-square&label=crates.io&color=000000"></a>
  <a href="https://docs.rs/merkle-variants"><img alt="docs.rs" src="https://img.shields.io/docsrs/merkle-variants?style=flat-square&label=docs.rs&color=000000"></a>
  <a href="#license"><img alt="license" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-000000?style=flat-square"></a>
</p>

`merkle-variants` contains the concrete tree implementations built on top of
`merkle-core`. Every tree is generic over `H: HashFunction`, so you can choose
SHA-256, Keccak-256, BLAKE3, or your own adapter without changing the tree API.

## Installation

```toml
[dependencies]
merkle-core = "0.5.0"
merkleforge-hash = "0.5.0"
merkle-variants = "0.5.0"
```

## Which Tree Should I Use?

| Variant | Use it for | Key capability |
| --- | --- | --- |
| `BinaryMerkleTree<H>` | Ordered lists, transaction batches, file chunks, logs | Compact inclusion proofs by leaf index |
| `SparseMerkleTree<H>` | Huge key spaces with mostly empty slots, account/state maps, rollups | Membership and non-membership proofs by 32-byte key |
| `MerklePatriciaTrie<H>` | Ethereum-compatible state, receipts, storage, and witnesses | RLP encoded nodes and Keccak-compatible roots |

## Binary Merkle Tree

```rust
use merkle_core::{
    prelude::*,
    types::LeafIndex,
};
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

fn main() -> Result<(), MerkleError> {
    let mut tree = BinaryMerkleTree::<Sha256>::new();
    tree.insert(b"tx:alice->bob:100")?;
    tree.insert(b"tx:bob->carol:50")?;

    let proof = tree.generate_proof(LeafIndex(0))?;
    let root = tree.root().expect("tree is not empty");

    assert!(BinaryMerkleTree::<Sha256>::verify(
        root,
        b"tx:alice->bob:100",
        &proof,
    ));

    Ok(())
}
```

Highlights:

- flat, cache-friendly storage;
- automatic power-of-two padding;
- stable proof generation by `LeafIndex`;
- generic `MerkleTree<H>` and `ProofVerifier<H>` support.

## Sparse Merkle Tree

```rust
use merkle_core::prelude::*;
use merkle_variants::SparseMerkleTree;
use merkleforge_hash::Sha256;

fn main() -> Result<(), MerkleError> {
    let mut tree = SparseMerkleTree::<Sha256>::new();
    let key = [7u8; 32];

    tree.insert(key, b"account:alice:100")?;

    let proof = tree.generate_membership_proof(key)?;
    let root = tree.root().expect("tree is not empty");

    assert!(SparseMerkleTree::<Sha256>::verify_membership(
        root,
        key,
        b"account:alice:100",
        &proof,
    ));

    Ok(())
}
```

Highlights:

- fixed 256-bit key space;
- precomputed empty-hash cache;
- shortcut-node compression for sparse data;
- batched terminal subtrees;
- membership and non-membership proofs;
- batch insert/remove APIs for grouped state updates.

## Merkle Patricia Trie

```rust
use merkle_core::prelude::*;
use merkle_variants::MerklePatriciaTrie;
use merkleforge_hash::Keccak256;

fn main() -> Result<(), MerkleError> {
    let mut trie = MerklePatriciaTrie::<Keccak256>::new();

    trie.insert(b"alice", b"100")?;
    trie.insert(b"bob", b"250")?;

    assert_eq!(trie.get(b"alice"), Some(&b"100"[..]));

    let proof = trie.generate_proof(b"alice")?;
    let root = trie.root().expect("trie is not empty");

    assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(
        root,
        &proof,
    ));

    Ok(())
}
```

Highlights:

- Ethereum-style leaf, extension, branch, and empty nodes;
- hex-prefix path encoding;
- Recursive Length Prefix node encoding;
- inline/hash node references following Ethereum rules;
- key-value insert, lookup, remove, and canonical node collapsing;
- EIP-1186-style membership and non-membership witnesses;
- validation against Ethereum trie vectors.

## Testing and Benchmarks

The crate is covered by unit tests, integration tests, property-based tests,
Ethereum vector tests, and Criterion benchmarks.

```bash
cargo test -p merkle-variants
cargo bench --bench binary_tree
cargo bench --bench sparse_tree
cargo bench --bench patricia_trie
```

Benchmark summaries live in [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md).

## Links

- API docs: <https://docs.rs/merkle-variants>
- Repository: <https://github.com/1888-Labs/MerkleForge>
- Website: <https://1888-labs.github.io/MerkleForge/>
- Hash adapters: <https://crates.io/crates/merkleforge-hash>
- Core traits: <https://crates.io/crates/merkle-core>

## Part of MerkleForge

| Crate | Role |
| --- | --- |
| [`merkle-core`](https://crates.io/crates/merkle-core) | Shared traits, proof types, serialization, and errors |
| [`merkleforge-hash`](https://crates.io/crates/merkleforge-hash) | SHA-256, Keccak-256, and BLAKE3 adapters |
| [`merkle-variants`](https://crates.io/crates/merkle-variants) | Binary, Sparse, and Patricia tree implementations |

## Safety

`merkle-variants` uses `#![forbid(unsafe_code)]`. The crate contains no unsafe
blocks.

Security note: MerkleForge Framework has not yet completed an independent
security audit. Review the code and threat model before using it in
security-critical environments.

## License

Licensed under either of:

- [MIT License](https://github.com/1888-Labs/MerkleForge/blob/main/LICENSE)
- [Apache License, Version 2.0](https://github.com/1888-Labs/MerkleForge/blob/main/LICENSE-APACHE)

at your option.
