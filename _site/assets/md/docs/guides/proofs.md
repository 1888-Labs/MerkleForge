# Proofs & stateless verification

A MerkleForge proof lets a verifier check one value against a trusted root, with no access to the
tree. This is the property that makes Merkle trees useful for light clients, bridges, browsers, and
embedded devices.

## Proof types

| Tree | Proof type | Verify with |
| --- | --- | --- |
| `BinaryMerkleTree` | `MerkleProof<D>` | `BinaryMerkleTree::<H>::verify(root, leaf, &proof)` |
| `SparseMerkleTree` | `SparseMerkleProof<D>` | `verify_membership(root, key, leaf, &proof)`, `verify_non_membership(root, key, &proof)` |
| `MerklePatriciaTrie` | `MptProof<D>` | `verify_proof(root, &proof)` |

The binary proof lives in `merkle-core`, so a verifier can depend on the shared types:

```rust
use merkle_core::types::{LeafIndex, ProofSide};

/// One step on the path: a sibling hash and the side it sits on.
pub struct ProofNode<D> {
    pub hash: D,
    pub side: ProofSide,
}

/// Inclusion proof for one leaf of a binary tree.
pub struct MerkleProof<D> {
    pub leaf_index: LeafIndex,
    pub leaf_count: usize,
    pub path: Vec<ProofNode<D>>, // O(log n) siblings, leaf level first
}
```

(Shown without derives and docs; `MerkleProof` also has `depth()` and `is_trivial()`.)

## The verification algorithm

```text
1. current = H(0x00 ‖ leaf_data)
2. for each node in proof.path:
     if node.side == Left:  current = H(0x01 ‖ node.hash ‖ current)
     if node.side == Right: current = H(0x01 ‖ current ‖ node.hash)
3. return current == expected_root
```

`BinaryMerkleTree::verify` also rejects proofs whose structure is inconsistent: empty leaf data,
`leaf_index >= leaf_count`, a path length that doesn't match the padded size, or sides that don't
match the bits of `leaf_index`. It returns `false` and never panics on malformed input.

## The `ProofVerifier` trait

```rust
use merkle_core::{traits::HashFunction, types::MerkleProof};

pub trait ProofVerifier<H: HashFunction> {
    fn verify(expected_root: &H::Digest, leaf_data: &[u8], proof: &MerkleProof<H::Digest>) -> bool;
}
```

It is separate from `MerkleTree` so it can be implemented by a type with no tree state at all.
`BinaryMerkleTree<H>` implements it, and its inherent `verify` delegates to it.

## Light-client example

The server builds the tree and exports the root and a serialized proof. The client receives bytes,
decodes the proof, and verifies, with the tree long gone.

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

/// What the server publishes: a root (e.g. in a block header) and proof bytes.
struct Published {
    root: [u8; 32],
    proof_bytes: Vec<u8>,
}

fn server(transactions: &[&str], index: usize) -> Result<Published, MerkleError> {
    let tree = BinaryMerkleTree::<Sha256>::from_leaves(transactions)?;
    let root = *tree.root().ok_or(MerkleError::EmptyTree)?;
    let proof = tree.generate_proof(LeafIndex(index))?;
    Ok(Published { root, proof_bytes: proof.to_bytes()? })
    // `tree` is dropped here.
}

fn client(trusted_root: &[u8; 32], claimed_tx: &str, proof_bytes: &[u8]) -> Result<bool, MerkleError> {
    let proof = MerkleProof::<[u8; 32]>::from_bytes(proof_bytes)?;
    Ok(BinaryMerkleTree::<Sha256>::verify(trusted_root, claimed_tx.as_bytes(), &proof))
}

fn main() -> Result<(), MerkleError> {
    let txs = ["tx:alice->bob:100", "tx:bob->carol:50", "tx:carol->dave:25"];
    let published = server(&txs, 0)?;

    println!("proof is {} bytes", published.proof_bytes.len());
    assert!(client(&published.root, "tx:alice->bob:100", &published.proof_bytes)?);
    assert!(!client(&published.root, "tx:alice->bob:1000", &published.proof_bytes)?);
    Ok(())
}
```

The repository ships the same flow as a runnable example:

```bash
cargo run -p merkle-variants --example light_client
```

Try it interactively on the [live demo](/demo): change the claimed value and watch verification
fail.

## What a verifier must trust

* **The root.** A proof only shows that a leaf is consistent with *some* root. Get the root from a
  trusted source: a block header, a signed checkpoint, a contract.
* **The leaf encoding.** Verify the exact bytes that were inserted. Agree on a canonical encoding
  (field order, separators, number formats) between prover and verifier.
* **Not `leaf_count`.** The binary root doesn't commit to `MerkleProof::leaf_count`. If the number
  of leaves matters, obtain it independently.
* **The key, for sparse proofs.** Use `verify_membership`, which checks the proof's key. The
  deprecated `SparseMerkleTree::verify` does not.

## Proof sizes

From the published comparison run on a 10,000-leaf tree:

| Library | Serialized proof size |
| --- | --- |
| MerkleForge | 469 bytes |
| `rs-merkle` | 448 bytes |
| `merkle_light` | 512 bytes |

A binary proof carries `⌈log₂ n⌉` 32-byte siblings plus a side tag each, the leaf index, and the
leaf count. See [Serialization](/docs/guides/serialization) for encoding options.
