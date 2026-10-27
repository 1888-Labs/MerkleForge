# Getting started

This page takes you from an empty project to a verified inclusion proof in a few minutes.

## Requirements

MerkleForge targets **Rust 1.97.1** or newer (edition 2024), the `rust-version` declared by every
crate. Check your toolchain with:

```bash
rustc --version
```

## Install

::::steps
### Create a project

```bash
cargo new merkle-hello
cd merkle-hello
```

### Add the crates

:::code-group
```bash [cargo add]
cargo add merkle-core merkleforge-hash merkle-variants
```

```toml [Cargo.toml]
[dependencies]
merkle-core = "0.4.1"
merkleforge-hash = "0.4.1"
merkle-variants = "0.4.1"
```
:::

Most applications want all three:

* `merkle-core` provides the traits (`MerkleTree`, `ProofVerifier`, `HashFunction`), the proof
  types, and `MerkleError`.
* `merkleforge-hash` provides the `Sha256`, `Keccak256`, and `Blake3` adapters.
* `merkle-variants` provides the trees.

Depend on `merkle-core` alone if you are writing your own tree or hash adapter.

### Write your first proof

Replace `src/main.rs` with:

```rust [src/main.rs]
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

fn main() -> Result<(), MerkleError> {
    let mut tree = BinaryMerkleTree::<Sha256>::new();

    tree.insert(b"alice:100")?;
    tree.insert(b"bob:250")?;
    tree.insert(b"carol:75")?;

    let root = tree.root().expect("tree is non-empty");
    println!("root: {root:02x?}");

    // Generate an inclusion proof for leaf 0 (alice).
    let proof = tree.generate_proof(LeafIndex(0))?;
    println!("proof depth: {}", proof.depth());

    // Stateless verification: no tree needed, only the root, leaf bytes, and proof.
    let valid = BinaryMerkleTree::<Sha256>::verify(root, b"alice:100", &proof);
    assert!(valid);

    // A different value for the same leaf is rejected.
    assert!(!BinaryMerkleTree::<Sha256>::verify(root, b"alice:999", &proof));
    Ok(())
}
```

### Run it

```bash
cargo run
```

You should see the root, then `proof depth: 2` (three leaves are padded to four, so the proof has
two sibling hashes).
::::

## What just happened?

1. Each `insert` hashed the raw bytes into a leaf, `H(0x00 ‖ data)`, and recomputed the path to
   the root.
2. `root()` returned the 32-byte commitment to all three leaves.
3. `generate_proof` collected the sibling hashes from leaf 0 up to the root.
4. `verify` rebuilt the root from `b"alice:100"` and those siblings, and compared it with the
   trusted root. It never touched the tree.

That last step is the **light-client pattern**: whoever holds a trusted root can check one value
without the dataset. The [Mental model](/docs/mental-model) page explains each piece.

## Build from many leaves at once

When you already have the full leaf set, `from_leaves` builds the tree in one `O(n)` pass. The
root, indices, and proofs are identical to sequential inserts.

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

fn main() -> Result<(), MerkleError> {
    let rows = ["alice:100", "bob:250", "carol:75"];
    let batch = BinaryMerkleTree::<Sha256>::from_leaves(rows)?;

    let mut sequential = BinaryMerkleTree::<Sha256>::new();
    for row in rows {
        sequential.insert(row.as_bytes())?;
    }

    assert_eq!(batch.root(), sequential.root());
    Ok(())
}
```

:::note
`BinaryMerkleTree::from_leaves` is new since 0.4.1 and listed under **Unreleased** in the
changelog. Use sequential `insert` calls if you depend on the published 0.4.1 crates.
:::

## Swap the hash function

Every tree is generic over its hash. Change one type parameter and nothing else:

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::{Blake3, Keccak256};

fn main() -> Result<(), MerkleError> {
    let mut fast = BinaryMerkleTree::<Blake3>::new();
    let mut evm = BinaryMerkleTree::<Keccak256>::new();

    fast.insert(b"alice:100")?;
    evm.insert(b"alice:100")?;

    // Different algorithms, different roots.
    assert_ne!(fast.root(), evm.root());
    Ok(())
}
```

See [Hash functions](/docs/guides/hash-functions) for when to pick each one.

## Try the light-client example

The repository ships a runnable server/client split:

```bash
git clone https://github.com/1888-Labs/MerkleForge.git
cd MerkleForge
cargo run -p merkle-variants --example light_client
```

```text
server: built tree and exported root + proof
client: tree dropped before verification
Stateless verification: true
```

The same flow runs in your browser on the [live demo](/demo).

## Next steps

* [Choosing a tree](/docs/guides/choosing-a-tree) when your data is a key-value map rather than a list.
* [Proofs & stateless verification](/docs/guides/proofs) to ship proofs to clients.
* [Serialization](/docs/guides/serialization) to store proofs and trees.
