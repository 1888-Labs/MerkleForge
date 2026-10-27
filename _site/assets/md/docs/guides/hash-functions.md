# Hash functions

Every MerkleForge tree is generic over `H: HashFunction`. The hash decides the digest type, how
leaves and internal nodes are hashed, and what an empty slot looks like. Swapping it is a
type-level change: Rust monomorphises the generics, so there is no runtime dispatch.

## The adapters

`merkleforge-hash` ships three adapters. All produce `[u8; 32]` digests.

| Type | Algorithm | `algorithm_name()` | Best for |
| --- | --- | --- | --- |
| `Sha256` | SHA-256 | `"SHA-256"` | Interoperable binary and sparse trees; hardware SHA extensions |
| `Keccak256` | Keccak-256 | `"Keccak-256"` | Ethereum-compatible Patricia roots |
| `Blake3` | BLAKE3 | `"BLAKE3"` | Highest software throughput when Ethereum compatibility isn't needed |

```rust
use merkle_core::traits::HashFunction;
use merkleforge_hash::{Blake3, Keccak256, Sha256};

fn main() {
    let sha = Sha256::hash(b"alice:100");
    let keccak = Keccak256::hash(b"alice:100");
    let blake = Blake3::hash(b"alice:100");

    assert_eq!(sha.len(), 32);
    assert_ne!(sha, keccak);
    assert_ne!(sha, blake);

    assert_eq!(Blake3::algorithm_name(), "BLAKE3");
    assert_eq!(Blake3::digest_size(), 32);

    // Combine two child digests into a parent.
    let left = Blake3::hash(b"left leaf");
    let right = Blake3::hash(b"right leaf");
    let _parent = Blake3::hash_nodes(&left, &right);
}
```

`merkleforge_hash` re-exports `HashFunction`, so `use merkleforge_hash::HashFunction;` also works.

## Domain separation

| Adapter | Leaf: `hash(data)` | Internal node: `hash_nodes(l, r)` | Encoded node: `hash_encoded_node(rlp)` |
| --- | --- | --- | --- |
| `Sha256` | `SHA-256(0x00 ‖ data)` | `SHA-256(0x01 ‖ l ‖ r)` | `SHA-256(0x00 ‖ rlp)` (default) |
| `Keccak256` | `Keccak-256(0x00 ‖ data)` | `Keccak-256(0x01 ‖ l ‖ r)` | `Keccak-256(rlp)`, plain, Ethereum-compatible |
| `Blake3` | `derive_key("MerkleForge 2026 leaf v1", data)` | `derive_key("MerkleForge 2026 internal-node v1", l ‖ r)` | Same as leaf (default) |

`empty()` is the leaf hash of an empty slice, `hash(b"")`, for all three adapters. Binary trees pad
with it and sparse trees use it for vacant slots.

Why it matters: see [Mental model → Domain separation](/docs/mental-model#domain-separation-0x00-and-0x01).

```rust
use merkle_core::traits::HashFunction;
use merkleforge_hash::Sha256;
use sha2::{Digest, Sha256 as RawSha256};

fn main() {
    // The adapter's leaf hash is SHA-256 over 0x00 ‖ data.
    let mut raw = RawSha256::new();
    raw.update([0x00]);
    raw.update(b"alice:100");
    let expected: [u8; 32] = raw.finalize().into();

    assert_eq!(Sha256::hash(b"alice:100"), expected);
    assert_eq!(Sha256::empty(), Sha256::hash(b""));
}
```

(This snippet uses the [`sha2`](https://crates.io/crates/sha2) crate directly to show the
pre-image. You don't need it to use MerkleForge.)

## Choosing one

* **Ethereum interop:** `Keccak256`. Required for Patricia roots that match Ethereum.
* **Broad interoperability:** `Sha256`. Ubiquitous, and fast on CPUs with SHA extensions.
* **Raw speed:** `Blake3`. In the published run it hashed 1 MB at 5.99 GiB/s versus 1.28 GiB/s for
  SHA-256 and 291 MiB/s for Keccak-256. See [Benchmarks](/docs/performance/benchmarks).

A tree built with one hash can't be read with another: serialized trees store the algorithm
name and deserialization rejects a mismatch.

## The `HashFunction` trait

```rust
use core::fmt::Debug;

pub trait HashFunction: Send + Sync + 'static {
    type Digest: AsRef<[u8]> + Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    /// Leaf hash. Must apply the leaf domain, e.g. H(0x00 ‖ data).
    fn hash(data: &[u8]) -> Self::Digest;

    /// Parent of two children. Default: hash(0x01 ‖ left ‖ right).
    fn hash_nodes(left: &Self::Digest, right: &Self::Digest) -> Self::Digest {
        let mut buf = Vec::with_capacity(1 + left.as_ref().len() + right.as_ref().len());
        buf.push(0x01);
        buf.extend_from_slice(left.as_ref());
        buf.extend_from_slice(right.as_ref());
        Self::hash(&buf)
    }

    /// Hash of a self-describing encoding such as an RLP Patricia node. Default: hash(encoded).
    fn hash_encoded_node(encoded: &[u8]) -> Self::Digest {
        Self::hash(encoded)
    }

    /// Digest of an empty slot. Default: hash(&[]).
    fn empty() -> Self::Digest {
        Self::hash(&[])
    }

    fn algorithm_name() -> &'static str;
    fn digest_size() -> usize;
}
```

This mirrors the definition in `merkle_core::traits::hash_function` (doc comments shortened).

## Writing a custom adapter

Implement `hash`, `algorithm_name`, and `digest_size`. Apply the `0x00` leaf prefix in `hash`; the
default `hash_nodes` applies `0x01`. This example wraps the `sha2` crate's SHA-512/256:

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use sha2::{Digest, Sha512_256};

pub struct Sha512Trunc;

impl HashFunction for Sha512Trunc {
    type Digest = [u8; 32];

    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha512_256::new();
        hasher.update([0x00]); // leaf domain
        hasher.update(data);
        hasher.finalize().into()
    }

    // hash_nodes, hash_encoded_node, and empty use the defaults.

    fn algorithm_name() -> &'static str {
        "SHA-512/256"
    }

    fn digest_size() -> usize {
        32
    }
}

fn main() -> Result<(), MerkleError> {
    let tree = BinaryMerkleTree::<Sha512Trunc>::from_leaves(["a", "b", "c"])?;
    let root = tree.root().expect("non-empty");
    let proof = tree.generate_proof(LeafIndex(2))?;
    assert!(BinaryMerkleTree::<Sha512Trunc>::verify(root, b"c", &proof));
    assert_eq!(tree.metadata().hash_algorithm, "SHA-512/256");
    Ok(())
}
```

Checklist for a custom adapter:

1. **Separate the domains.** `hash` must not collide with `hash_nodes` for different structures.
   Prefix leaves with `0x00`, or override both methods with your own separation (as `Blake3`
   does with context strings).
2. **Override for speed if you like.** `hash_nodes` can hash `0x01 ‖ l ‖ r` without the
   intermediate `Vec`, as the shipped adapters do. Keep the same output.
3. **Ethereum roots** need `hash_encoded_node` to return the plain hash of the RLP bytes.
4. **Use a unique `algorithm_name`.** Serialized trees store it to reject mismatched hashes.

:::warning
Since **Unreleased**, the default `hash_nodes` hashes `0x01 ‖ left ‖ right` instead of
`left ‖ right`. Custom adapters that relied on the old default now produce different roots.
The shipped adapters override `hash_nodes` and are unaffected.
:::
