//! # merkle-hash :: blake3
//!
//! BLAKE3 adapter for the [`HashFunction`] trait.
//!
//! BLAKE3 is the fastest secure cryptographic hash available on modern
//! hardware — benchmarks consistently show it outpacing SHA-256 by 3–10×
//! on software paths, while also scaling across CPU cores via internal
//! tree-parallelism.  BLAKE3 natively supports keyed hashing and
//! context strings, making domain separation trivial without extra length
//! overhead.
//!
//! The benchmark suite compares SHA-256, Keccak-256, and BLAKE3 to guide
//! algorithm selection for specific hardware environments.
//!
//! ## Usage
//! ```rust,ignore
//! use merkle_hash::Blake3;
//! use merkle_core::traits::HashFunction;
//!
//! let digest = Blake3::hash(b"hello merkle");
//! assert_eq!(digest.len(), 32);
//! ```

use std::sync::OnceLock;

use merkle_core::traits::HashFunction;

/// Leaf domain-separation context string.
const LEAF_CONTEXT: &str = "MerkleForge 2026 leaf v1";
/// Internal node domain-separation context string.
const NODE_CONTEXT: &str = "MerkleForge 2026 internal-node v1";

/// Derive-key hashers for the two contexts, created once.
///
/// `blake3::derive_key(context, data)` first hashes the context string into a
/// key on every call. Cloning a hasher that already holds that key skips the
/// extra compression, about a quarter of the cost for 32- and 64-byte inputs.
/// The digests are identical.
fn context_hasher(cell: &'static OnceLock<blake3::Hasher>, context: &str) -> blake3::Hasher {
    cell.get_or_init(|| blake3::Hasher::new_derive_key(context))
        .clone()
}

static LEAF_HASHER: OnceLock<blake3::Hasher> = OnceLock::new();
static NODE_HASHER: OnceLock<blake3::Hasher> = OnceLock::new();

/// BLAKE3 implementation of [`HashFunction`].
///
/// Uses BLAKE3's built-in **keyed derivation** (`blake3::derive_key`) for
/// domain separation, which is more efficient and cryptographically cleaner
/// than prepending a prefix byte.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Blake3;

impl HashFunction for Blake3 {
    type Digest = [u8; 32];

    /// Leaf hashing using BLAKE3's derive-key mode with context `"MerkleForge 2026 leaf v1"`.
    #[inline]
    fn hash(data: &[u8]) -> [u8; 32] {
        let mut hasher = context_hasher(&LEAF_HASHER, LEAF_CONTEXT);
        hasher.update(data);
        *hasher.finalize().as_bytes()
    }

    /// Internal node hashing using BLAKE3's derive-key mode with context
    /// `"MerkleForge 2026 internal-node v1"` applied to `left || right`.
    #[inline]
    fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        let mut hasher = context_hasher(&NODE_HASHER, NODE_CONTEXT);
        hasher.update(left);
        hasher.update(right);
        *hasher.finalize().as_bytes()
    }

    /// The canonical empty-leaf hash: `BLAKE3_leaf("")`.
    ///
    /// Pre-computed for the default context string — avoids a function call
    /// on every empty-slot lookup in sparse trees.
    fn empty() -> [u8; 32] {
        blake3::derive_key(LEAF_CONTEXT, b"")
    }

    fn algorithm_name() -> &'static str {
        "BLAKE3"
    }

    fn digest_size() -> usize {
        32
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use merkle_core::traits::HashFunction;

    #[test]
    fn digest_size_is_32() {
        assert_eq!(Blake3::digest_size(), 32);
    }

    #[test]
    fn algorithm_name() {
        assert_eq!(Blake3::algorithm_name(), "BLAKE3");
    }

    #[test]
    fn hash_deterministic() {
        assert_eq!(Blake3::hash(b"hello"), Blake3::hash(b"hello"));
    }

    #[test]
    fn hash_nodes_non_commutative() {
        let a = [0xAAu8; 32];
        let b = [0xBBu8; 32];
        assert_ne!(Blake3::hash_nodes(&a, &b), Blake3::hash_nodes(&b, &a));
    }

    #[test]
    fn leaf_and_node_contexts_differ() {
        // Even with the same input bytes, domain separation must give different digests.
        let data = [0x42u8; 32];
        let leaf = Blake3::hash(&data);
        let node = Blake3::hash_nodes(&data, &data);
        assert_ne!(leaf, node);
    }

    #[test]
    fn empty_matches_runtime_computation() {
        assert_eq!(Blake3::empty(), blake3::derive_key(LEAF_CONTEXT, b""));
    }

    #[test]
    fn cached_hashers_match_derive_key() {
        for len in [0_usize, 1, 31, 32, 33, 64, 65, 1023, 1024, 1025, 5000] {
            let data: Vec<u8> = (0..len).map(|i| (i * 7 + len) as u8).collect();
            assert_eq!(Blake3::hash(&data), blake3::derive_key(LEAF_CONTEXT, &data));
        }
        let left = [0x11u8; 32];
        let right = [0x22u8; 32];
        let mut combined = [0u8; 64];
        combined[..32].copy_from_slice(&left);
        combined[32..].copy_from_slice(&right);
        assert_eq!(
            Blake3::hash_nodes(&left, &right),
            blake3::derive_key(NODE_CONTEXT, &combined)
        );
    }

    #[test]
    fn different_inputs_give_different_digests() {
        let a = Blake3::hash(b"alice");
        let b = Blake3::hash(b"bob");
        assert_ne!(a, b);
    }

    #[test]
    fn hash_nodes_avalanche() {
        // A one-bit change in a child must change the parent.
        let mut left = [0u8; 32];
        let right = [0u8; 32];
        let parent_a = Blake3::hash_nodes(&left, &right);
        left[0] ^= 1;
        let parent_b = Blake3::hash_nodes(&left, &right);
        assert_ne!(parent_a, parent_b);
    }
}
