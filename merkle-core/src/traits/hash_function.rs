//! # merkle-core :: traits :: `hash_function`
//!
//! The `HashFunction` trait is the **pluggable crypto abstraction** at the
//! heart of the library.  By parameterising every tree type over
//! `H: HashFunction`, callers can swap SHA-256 for BLAKE3 with zero changes
//! to tree logic and zero runtime cost (Rust monomorphises the generics).
//!
//! ## Implementing `HashFunction`
//!
//! ```rust,ignore
//! use merkle_core::traits::HashFunction;
//!
//! pub struct MyHash;
//!
//! impl HashFunction for MyHash {
//!     type Digest = [u8; 32];
//!
//!     // Leaves must be domain-separated from internal nodes: prefix the
//!     // leaf pre-image with 0x00 (the default `hash_nodes` uses 0x01).
//!     fn hash(data: &[u8]) -> Self::Digest { /* MyHash(0x00 || data) */ }
//!
//!     fn algorithm_name() -> &'static str { "MyHash" }
//!
//!     fn digest_size() -> usize { 32 }
//! }
//! ```
//!
//! ## Domain separation
//!
//! Every tree hashes leaves with [`HashFunction::hash`] and internal nodes
//! with [`HashFunction::hash_nodes`]. The two must never produce the same
//! digest for different structures, or an internal node could be presented
//! as a leaf (a second-preimage attack). The convention is:
//!
//! - `hash(data)` = `H(0x00 || data)`
//! - `hash_nodes(left, right)` = `H(0x01 || left || right)`
//!
//! The default `hash_nodes` applies the `0x01` prefix, so implementors only
//! need to apply the `0x00` prefix in `hash`.

use alloc::vec::Vec;
use core::fmt::Debug;

/// Abstraction over a cryptographic hash function used by Merkle trees.
///
/// # Type parameter `Digest`
/// The associated type `Digest` represents a fixed-size hash output.  It
/// must be:
/// - `Clone` — nodes are cloned when building the tree.
/// - `PartialEq + Eq` — so roots and proof hashes can be compared.
/// - `AsRef<[u8]>` — for serialisation and concatenation before hashing.
/// - `Debug` — for test output and diagnostic messages.
/// - `Send + Sync` — so trees can be used across thread boundaries.
///
/// # Zero-cost abstractions
/// All methods are `#[inline]` by convention in implementors.  Rust's
/// monomorphisation ensures that no vtable dispatch overhead occurs when
/// calling through `H: HashFunction` bounds.
pub trait HashFunction: Send + Sync + 'static {
    /// The concrete digest type produced by this hash function.
    type Digest: AsRef<[u8]> + Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

    // ── Core hashing ──────────────────────────────────────────────────────

    /// Hash arbitrary bytes and return the digest.
    #[must_use]
    fn hash(data: &[u8]) -> Self::Digest;

    /// Hash two child digests together to produce a parent digest.
    ///
    /// The default implementation hashes `0x01 || left || right` with
    /// [`Self::hash`], which keeps internal nodes domain-separated from
    /// leaves as long as `hash` prefixes leaf data with `0x00`. Override this
    /// only to use a faster or algorithm-specific construction that keeps the
    /// same separation.
    #[must_use]
    fn hash_nodes(left: &Self::Digest, right: &Self::Digest) -> Self::Digest {
        let mut combined = Vec::with_capacity(1 + left.as_ref().len() + right.as_ref().len());
        combined.push(0x01);
        combined.extend_from_slice(left.as_ref());
        combined.extend_from_slice(right.as_ref());
        Self::hash(&combined)
    }

    /// Hash an encoded node whose encoding already identifies its node type,
    /// such as an RLP-encoded Merkle Patricia Trie node.
    ///
    /// Structures with self-describing encodings use this instead of
    /// [`Self::hash`]. The default delegates to [`Self::hash`]. Adapters that
    /// must match an external format override it; for example, the
    /// Keccak-256 adapter returns plain `Keccak-256(encoded)` so Patricia
    /// roots match Ethereum.
    #[must_use]
    fn hash_encoded_node(encoded: &[u8]) -> Self::Digest {
        Self::hash(encoded)
    }

    /// Return the canonical "empty" digest used to pad trees to a
    /// power-of-two size, or to represent vacant slots in sparse trees.
    ///
    /// The default is the hash of a zero-length byte slice.
    #[must_use]
    fn empty() -> Self::Digest {
        Self::hash(&[])
    }

    // ── Metadata ──────────────────────────────────────────────────────────

    /// Human-readable name of the algorithm, e.g. `"SHA-256"` or `"BLAKE3"`.
    #[must_use]
    fn algorithm_name() -> &'static str;

    /// Output size of the digest in bytes.
    #[must_use]
    fn digest_size() -> usize;
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    /// A minimal stub `HashFunction` used only in unit tests so that
    /// `merkle-core` has no dependency on `merkle-hash` at test time.
    use super::*;

    struct Xor8;

    impl HashFunction for Xor8 {
        type Digest = [u8; 1];

        fn hash(data: &[u8]) -> [u8; 1] {
            [data.iter().fold(0u8, |acc, &b| acc ^ b)]
        }

        fn algorithm_name() -> &'static str {
            "XOR8"
        }

        fn digest_size() -> usize {
            1
        }
    }

    #[test]
    fn hash_deterministic() {
        let a = Xor8::hash(b"hello");
        let b = Xor8::hash(b"hello");
        assert_eq!(a, b);
    }

    #[test]
    fn default_hash_nodes_prefixes_internal_node_domain() {
        let l = Xor8::hash(b"left");
        let r = Xor8::hash(b"right");
        let parent = Xor8::hash_nodes(&l, &r);
        // XOR8(0x01 || l || r) = 0x01 ^ l ^ r for single-byte digests.
        let expected = [0x01 ^ l[0] ^ r[0]];
        assert_eq!(parent, expected);
    }

    #[test]
    fn default_hash_encoded_node_delegates_to_hash() {
        assert_eq!(Xor8::hash_encoded_node(b"node"), Xor8::hash(b"node"));
    }

    #[test]
    fn empty_is_hash_of_empty_slice() {
        assert_eq!(Xor8::empty(), Xor8::hash(&[]));
    }

    #[test]
    fn metadata() {
        assert_eq!(Xor8::algorithm_name(), "XOR8");
        assert_eq!(Xor8::digest_size(), 1);
    }
}
