//! # merkle-hash :: sha256
//!
//! SHA-256 adapter for the [`HashFunction`] trait.
//!
//! SHA-256 is the most widely deployed hash function in blockchain systems.
//! It benefits from hardware acceleration (Intel SHA Extensions, ARM SHA)
//! which can yield a ~50% speed boost over software implementations on
//! supported CPUs (Drake, 2019).  This makes it a strong default for
//! production deployments that run on modern server hardware.
//!
//! ## Usage
//! ```rust,ignore
//! use merkle_hash::Sha256;
//! use merkle_core::traits::HashFunction;
//!
//! let digest = Sha256::hash(b"hello merkle");
//! assert_eq!(digest.len(), 32);
//! println!("SHA-256 name: {}", Sha256::algorithm_name());
//! ```

use merkle_core::traits::HashFunction;
use sha2::digest::generic_array::GenericArray;
use sha2::{Digest as Sha2Digest, Sha256 as Sha2_256};

/// SHA-256 initial state (FIPS 180-4, section 5.3.3).
const IV: [u32; 8] = [
    0x6a09_e667,
    0xbb67_ae85,
    0x3c6e_f372,
    0xa54f_f53a,
    0x510e_527f,
    0x9b05_688c,
    0x1f83_d9ab,
    0x5be0_cd19,
];

/// Big-endian bytes of a SHA-256 state.
#[inline]
fn state_bytes(state: &[u32; 8]) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (chunk, word) in out.chunks_exact_mut(4).zip(state) {
        chunk.copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// `SHA-256(0x01 || left || right)` with the two padded blocks laid out
/// directly and handed to the compression function, skipping the streaming
/// hasher's buffering. The 65-byte message always needs exactly two blocks:
/// the second holds the last byte of `right`, the `0x80` pad, and the bit
/// length (520). Same digest as the streaming hasher.
#[inline]
fn node_digest(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
    let mut blocks = [GenericArray::<u8, sha2::digest::consts::U64>::default(); 2];
    blocks[0][0] = 0x01;
    blocks[0][1..33].copy_from_slice(left);
    blocks[0][33..64].copy_from_slice(&right[..31]);
    blocks[1][0] = right[31];
    blocks[1][1] = 0x80;
    blocks[1][56..64].copy_from_slice(&520u64.to_be_bytes());
    let mut state = IV;
    sha2::compress256(&mut state, &blocks);
    state_bytes(&state)
}

/// `SHA-256(0x00 || data)` for data of at most 54 bytes, which fits one
/// padded block (1 prefix byte + data + `0x80` + 8 length bytes <= 64).
#[inline]
fn short_leaf_digest(data: &[u8]) -> [u8; 32] {
    debug_assert!(data.len() <= 54);
    let mut block = [GenericArray::<u8, sha2::digest::consts::U64>::default(); 1];
    block[0][1..=data.len()].copy_from_slice(data);
    block[0][data.len() + 1] = 0x80;
    block[0][56..64].copy_from_slice(&(((data.len() as u64) + 1) * 8).to_be_bytes());
    let mut state = IV;
    sha2::compress256(&mut state, &block);
    state_bytes(&state)
}

/// SHA-256 implementation of [`HashFunction`].
///
/// Produces 32-byte digests.  Hardware-accelerated on x86-64 CPUs with the
/// SHA or AVX2 extension (detected at compile time by the `sha2` crate).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sha256;

impl HashFunction for Sha256 {
    /// 32-byte fixed-size digest.
    type Digest = [u8; 32];

    /// Domain-separated node hashing: `SHA-256(0x01 || left || right)`.
    ///
    /// The `0x01` prefix distinguishes internal nodes from leaf hashes
    /// (`0x00` prefix), preventing second-preimage attacks where an attacker
    /// could substitute an internal node for a leaf.
    #[inline]
    fn hash_nodes(left: &[u8; 32], right: &[u8; 32]) -> [u8; 32] {
        node_digest(left, right) // 0x01 internal-node domain separator
    }

    /// Leaf hashing: `SHA-256(0x00 || data)`.
    ///
    /// The `0x00` prefix distinguishes leaf hashes from internal-node hashes.
    /// Callers that hash leaf data themselves should use this for correctness,
    /// but the tree implementations call it internally so most users don't
    /// need to know about it.
    #[inline]
    fn hash(data: &[u8]) -> [u8; 32] {
        if data.len() <= 54 {
            return short_leaf_digest(data);
        }
        let mut hasher = Sha2_256::new();
        hasher.update([0x00]); // Leaf domain separator
        hasher.update(data);
        hasher.finalize().into()
    }

    #[inline]
    fn empty() -> [u8; 32] {
        // Pre-computed: SHA-256(0x00) — the canonical empty-leaf hash.
        // This avoids a runtime hash call every time we need the sentinel.
        [
            0x6e, 0x34, 0x0b, 0x9c, 0xff, 0xb3, 0x7a, 0x98, 0x9c, 0xa5, 0x44, 0xe6, 0xbb, 0x78,
            0x0a, 0x2c, 0x78, 0x90, 0x1d, 0x3f, 0xb3, 0x37, 0x38, 0x76, 0x85, 0x11, 0xa3, 0x06,
            0x17, 0xaf, 0xa0, 0x1d,
        ]
    }

    fn algorithm_name() -> &'static str {
        "SHA-256"
    }

    fn digest_size() -> usize {
        32
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn streaming(parts: &[&[u8]]) -> [u8; 32] {
        let mut hasher = Sha2_256::new();
        for part in parts {
            hasher.update(part);
        }
        hasher.finalize().into()
    }

    #[test]
    fn direct_compression_matches_the_streaming_hasher() {
        // Every length across the one-block cutoff (54/55) and several blocks.
        for len in 0..300usize {
            let data: Vec<u8> = (0..len).map(|i| (i * 131 + len) as u8).collect();
            assert_eq!(
                Sha256::hash(&data),
                streaming(&[&[0x00], &data]),
                "leaf, {len} bytes"
            );
        }
        let mut left = [0u8; 32];
        let mut right = [0u8; 32];
        for round in 0..1000u32 {
            for (i, (l, r)) in left.iter_mut().zip(right.iter_mut()).enumerate() {
                *l = (round as usize * 7 + i * 13) as u8;
                *r = (round as usize * 11 + i * 29) as u8 ^ 0xa5;
            }
            assert_eq!(
                Sha256::hash_nodes(&left, &right),
                streaming(&[&[0x01], &left, &right]),
                "node, round {round}"
            );
        }
    }

    /// SHA-256(0x00) — The domain-separated empty leaf hash for MerkleForge.
    const MERKLE_EMPTY_SENTINEL: [u8; 32] = [
        0x6e, 0x34, 0x0b, 0x9c, 0xff, 0xb3, 0x7a, 0x98, 0x9c, 0xa5, 0x44, 0xe6, 0xbb, 0x78, 0x0a,
        0x2c, 0x78, 0x90, 0x1d, 0x3f, 0xb3, 0x37, 0x38, 0x76, 0x85, 0x11, 0xa3, 0x06, 0x17, 0xaf,
        0xa0, 0x1d,
    ];

    #[test]
    fn digest_size_is_32() {
        assert_eq!(Sha256::digest_size(), 32);
    }

    #[test]
    fn algorithm_name() {
        assert_eq!(Sha256::algorithm_name(), "SHA-256");
    }

    #[test]
    fn hash_is_deterministic() {
        let input = b"merkleforge-test";
        assert_eq!(Sha256::hash(input), Sha256::hash(input));
    }

    #[test]
    fn empty_matches_precomputed() {
        // This ensures the library's precomputed constant matches a fresh runtime hash of the 0x00 prefix.
        let mut h = Sha2_256::new();
        h.update([0x00u8]);
        let manual_compute: [u8; 32] = h.finalize().into();

        assert_eq!(
            Sha256::empty(),
            manual_compute,
            "Precomputed empty() must match SHA-256(0x00)"
        );
        assert_eq!(Sha256::empty(), MERKLE_EMPTY_SENTINEL);
    }

    #[test]
    fn hash_nodes_non_commutative() {
        let a = [0x11u8; 32];
        let b = [0x22u8; 32];
        assert_ne!(Sha256::hash_nodes(&a, &b), Sha256::hash_nodes(&b, &a));
    }

    #[test]
    fn leaf_and_node_domain_separation() {
        let data = [0xAAu8; 32];
        let dummy = [0x00u8; 32];

        // hash(data) uses 0x00 prefix; hash_nodes(data, dummy) uses 0x01 prefix.
        // Even if the input starts similarly, the prefixes ensure the digests differ.
        let leaf_h = Sha256::hash(&data);
        let node_h = Sha256::hash_nodes(&data, &dummy);

        assert_ne!(
            leaf_h, node_h,
            "Leaf and Internal Node hashes must be domain-separated"
        );
    }

    #[test]
    fn different_inputs_different_hashes() {
        assert_ne!(Sha256::hash(b"leaf1"), Sha256::hash(b"leaf2"));
    }
}
