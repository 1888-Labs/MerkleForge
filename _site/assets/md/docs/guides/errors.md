# Errors

Every fallible MerkleForge API returns `Result<T, MerkleError>`. Verification functions return
`bool` instead: a bad proof is an expected outcome, not an error.

`MerkleError` lives in `merkle_core::error` (and the prelude). It implements `Debug`, `Clone`,
`PartialEq`, `Eq`, `Display`, and, with the default `std` feature, `std::error::Error`, so it works
with `?`, `Box<dyn Error>`, `anyhow`, and `thiserror`.

## Variants

| Variant | Raised when |
| --- | --- |
| `EmptyTree` | Proof generation (or binary `remove`) on a tree with no leaves |
| `IndexOutOfBounds { index, len }` | A `LeafIndex` past the last logical leaf |
| `InvalidProof` | Available to APIs that report failed verification as an error (the shipped verifiers return `false` instead) |
| `InvalidProofStructure(String)` | A proof's shape is inconsistent (e.g. sparse proof generation with a non-256 depth) |
| `EmptyLeafData` | Inserting an empty value (empty data would be indistinguishable from padding) |
| `SerializationError(String)` | `to_bytes` failed |
| `DeserializationError(String)` | `from_bytes` got malformed data |
| `HashError(String)` | An internal hashing step failed (for custom adapters; not raised by the shipped ones) |
| `UnsupportedOperation(&'static str)` | The variant can't do this, e.g. removing a missing sparse or Patricia key, or index-based Patricia proofs |
| `RlpError(String)` | Malformed RLP or hex-prefix data in the Patricia trie |

## Handling errors

`MerkleError` is `#[non_exhaustive]`: new variants may arrive in minor releases, so keep a
fallback arm.

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

fn describe(result: Result<MerkleProof<[u8; 32]>, MerkleError>) -> String {
    match result {
        Ok(proof) => format!("proof with {} siblings", proof.depth()),
        Err(MerkleError::EmptyTree) => "tree has no leaves".into(),
        Err(MerkleError::IndexOutOfBounds { index, len }) => {
            format!("leaf {index} is outside the current leaf count {len}")
        }
        Err(MerkleError::EmptyLeafData) => "empty leaves are rejected".into(),
        Err(other) => other.to_string(),
    }
}

fn main() -> Result<(), MerkleError> {
    let mut tree = BinaryMerkleTree::<Sha256>::new();
    assert_eq!(describe(tree.generate_proof(LeafIndex(0))), "tree has no leaves");

    tree.insert(b"alice")?;
    assert_eq!(
        describe(tree.generate_proof(LeafIndex(3))),
        "leaf 3 is outside the current leaf count 1",
    );
    assert_eq!(describe(tree.generate_proof(LeafIndex(0))), "proof with 0 siblings");
    Ok(())
}
```

## Display messages

```rust
use merkle_core::error::MerkleError;

fn main() {
    let err = MerkleError::IndexOutOfBounds { index: 5, len: 3 };
    assert_eq!(err.to_string(), "index 5 is out of bounds for a tree with 3 leaves");
    assert_eq!(MerkleError::EmptyLeafData.to_string(), "leaf data must not be empty");
}
```

## Using with `Box<dyn Error>`

```rust
use std::error::Error;

use merkle_variants::MerklePatriciaTrie;
use merkleforge_hash::Keccak256;

fn main() -> Result<(), Box<dyn Error>> {
    let mut trie = MerklePatriciaTrie::<Keccak256>::new();
    trie.insert(b"key", b"value")?;

    // Removing a key that isn't there is an error.
    let err = trie.remove(b"missing").unwrap_err();
    println!("{err}");
    Ok(())
}
```
