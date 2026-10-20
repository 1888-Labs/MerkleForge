//! Binary Merkle tree implementation.
//!
//! [`BinaryMerkleTree`] stores leaves and internal nodes in one contiguous
//! allocation and implements [`MerkleTree`] for every supported
//! [`HashFunction`].
//!
//! # Example
//!
//! ```
//! use merkle_core::{
//!     traits::{MerkleTree, ProofVerifier},
//!     types::LeafIndex,
//! };
//! use merkle_variants::BinaryMerkleTree;
//! use merkleforge_hash::Sha256;
//!
//! let mut tree = BinaryMerkleTree::<Sha256>::new();
//! tree.insert(b"alice:100")?;
//! tree.insert(b"bob:250")?;
//!
//! let proof = tree.generate_proof(LeafIndex(0))?;
//! let root = tree.root().expect("the tree contains two leaves");
//! assert!(BinaryMerkleTree::<Sha256>::verify(
//!     root,
//!     b"alice:100",
//!     &proof,
//! ));
//!
//! let metadata = tree.metadata();
//! assert_eq!(metadata.variant, "BinaryMerkleTree");
//! assert_eq!(metadata.hash_algorithm, "SHA-256");
//! # Ok::<(), merkle_core::error::MerkleError>(())
//! ```

use core::fmt;

use merkle_core::{
    error::MerkleError,
    traits::{HashFunction, MerkleTree, ProofVerifier},
    types::{LeafIndex, MerkleProof, ProofNode, ProofSide, TreeMetadata},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// One inclusion proof for several leaves of a [`BinaryMerkleTree`].
///
/// `hashes` holds only the siblings that cannot be computed from the proven
/// leaves, ordered layer by layer from the leaves up and left to right within
/// a layer. Proving `k` of `n` leaves needs at most `k * log2(n)` hashes, and
/// neighbouring leaves share siblings, so it is usually far fewer.
///
/// The serialized form (serde and [`Serializable`](merkle_core::traits::Serializable))
/// matches `BinaryMultiProof` in the TypeScript package `@merkleforge/variants`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryMultiProof<D> {
    /// Proven leaf indices, strictly increasing.
    pub leaf_indices: Vec<usize>,
    /// Number of leaves in the tree the proof was generated from.
    pub leaf_count: usize,
    /// Sibling hashes in proof order.
    pub hashes: Vec<D>,
}

/// A cache-friendly binary Merkle tree backed by a flat node array.
///
/// Leaf nodes occupy the first power-of-two-sized layer. Each following
/// layer stores the parents of the previous layer, with the root at the final
/// array position.
pub struct BinaryMerkleTree<H: HashFunction> {
    nodes: Vec<H::Digest>,
    leaf_count: usize,
    height: usize,
}

impl<H: HashFunction> BinaryMerkleTree<H> {
    /// Creates an empty binary Merkle tree without preallocated leaf slots.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            nodes: Vec::new(),
            leaf_count: 0,
            height: 0,
        }
    }

    /// Creates an empty tree with storage prepared for at least `capacity`
    /// leaves.
    ///
    /// The backing leaf layer is rounded up to the next power of two and
    /// initialized with [`HashFunction::empty`]. Reserving storage does not
    /// add logical leaves, so the returned tree remains empty.
    ///
    /// # Panics
    ///
    /// Panics if the requested capacity is too large to represent the flat
    /// node array.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        if capacity == 0 {
            return Self::new();
        }

        let leaf_capacity = Self::padded_leaf_count(capacity);
        let mut tree = Self {
            nodes: vec![H::empty(); Self::node_count(leaf_capacity)],
            leaf_count: 0,
            height: 0,
        };
        tree.recompute_root();
        tree
    }

    /// Builds a tree from many leaves in one pass.
    ///
    /// The result has the same root, indices, and proofs as inserting the
    /// leaves one at a time, but internal nodes are computed once, in
    /// `O(n)` hashes instead of `O(n log n)`.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyLeafData`] if any leaf is empty. No tree
    /// is returned in that case.
    ///
    /// # Panics
    ///
    /// Panics if the number of leaves is too large to represent the flat
    /// node array.
    pub fn from_leaves<I>(leaves: I) -> Result<Self, MerkleError>
    where
        I: IntoIterator,
        I::Item: AsRef<[u8]>,
    {
        let hashes = leaves
            .into_iter()
            .map(|leaf| {
                let data = leaf.as_ref();
                if data.is_empty() {
                    Err(MerkleError::EmptyLeafData)
                } else {
                    Ok(H::hash(data))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self::from_leaf_hashes(hashes))
    }

    /// Inserts a non-empty leaf and returns its stable leaf index.
    ///
    /// The leaf layer grows to the next power of two when its current
    /// capacity is exhausted. Internal nodes are then recomputed iteratively
    /// from the leaf layer to the root.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyLeafData`] when `data` is empty.
    pub fn insert(&mut self, data: &[u8]) -> Result<LeafIndex, MerkleError> {
        if data.is_empty() {
            return Err(MerkleError::EmptyLeafData);
        }

        let index = LeafIndex(self.leaf_count);
        self.ensure_leaf_capacity(self.leaf_count + 1);
        self.nodes[index.value()] = H::hash(data);
        self.leaf_count += 1;
        self.height = self.leaf_capacity().ilog2() as usize + 1;
        self.recompute_path(index.value());

        Ok(index)
    }

    /// Removes a leaf by replacing it with [`HashFunction::empty`].
    ///
    /// Removing trailing leaves also reduces the logical leaf count and may
    /// shrink the padded tree. Interior removals preserve all later indices.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyTree`] when the tree has no logical leaves.
    /// Returns [`MerkleError::IndexOutOfBounds`] when `index` does not refer
    /// to a logical leaf.
    pub fn remove(&mut self, index: LeafIndex) -> Result<(), MerkleError> {
        if self.is_empty() {
            return Err(MerkleError::EmptyTree);
        }
        if index.value() >= self.leaf_count {
            return Err(MerkleError::IndexOutOfBounds {
                index: index.value(),
                len: self.leaf_count,
            });
        }

        self.nodes[index.value()] = H::empty();
        self.trim_empty_trailing_leaves();

        if self.leaf_count == 0 {
            self.height = 0;
            self.recompute_root();
            return Ok(());
        }

        let required_capacity = Self::padded_leaf_count(self.leaf_count);
        if required_capacity < self.leaf_capacity() {
            self.resize_leaf_layer(required_capacity);
        } else {
            self.recompute_path(index.value());
        }
        self.height = required_capacity.ilog2() as usize + 1;

        Ok(())
    }

    /// Returns the current root digest, or `None` when no leaves are stored.
    #[must_use]
    pub fn root(&self) -> Option<&H::Digest> {
        if self.is_empty() {
            None
        } else {
            self.nodes.last()
        }
    }

    /// Returns the number of logical leaves currently stored.
    #[must_use]
    pub const fn leaf_count(&self) -> usize {
        self.leaf_count
    }

    /// Returns `true` when the tree contains no logical leaves.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.leaf_count == 0
    }

    /// Returns the tree height, where an empty tree has height zero.
    #[must_use]
    pub const fn height(&self) -> usize {
        self.height
    }

    /// Generates an inclusion proof for the leaf at `index`.
    ///
    /// Sibling hashes are collected from the leaf layer upward. Each proof
    /// node records whether the sibling belongs on the left or right when
    /// reconstructing the root.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyTree`] when the tree has no logical leaves.
    /// Returns [`MerkleError::IndexOutOfBounds`] when `index` is not a logical
    /// leaf index.
    pub fn generate_proof(&self, index: LeafIndex) -> Result<MerkleProof<H::Digest>, MerkleError> {
        if self.is_empty() {
            return Err(MerkleError::EmptyTree);
        }
        if index.value() >= self.leaf_count {
            return Err(MerkleError::IndexOutOfBounds {
                index: index.value(),
                len: self.leaf_count,
            });
        }

        let mut path = Vec::with_capacity(self.height.saturating_sub(1));
        let mut layer_start = 0;
        let mut layer_width = self.leaf_capacity();
        let mut offset = index.value();

        while layer_width > 1 {
            let (sibling_offset, side) = if offset.is_multiple_of(2) {
                (offset + 1, ProofSide::Right)
            } else {
                (offset - 1, ProofSide::Left)
            };

            path.push(ProofNode {
                hash: self.nodes[layer_start + sibling_offset].clone(),
                side,
            });

            offset /= 2;
            layer_start += layer_width;
            layer_width /= 2;
        }

        Ok(MerkleProof {
            leaf_index: index,
            leaf_count: self.leaf_count,
            path,
        })
    }

    /// Verifies an inclusion proof without accessing a tree instance.
    ///
    /// Returns `false` when the proof structure is inconsistent with its
    /// claimed leaf index or leaf count.
    #[must_use]
    pub fn verify(
        expected_root: &H::Digest,
        leaf_data: &[u8],
        proof: &MerkleProof<H::Digest>,
    ) -> bool {
        <Self as ProofVerifier<H>>::verify(expected_root, leaf_data, proof)
    }

    /// Builds a tree whose leaf layer is exactly `hashes`, dropping trailing
    /// empty slots the same way [`Self::remove`] does.
    /// Generates one proof for several leaves.
    ///
    /// Indices may be in any order and may repeat; the proof lists them
    /// sorted and de-duplicated in [`BinaryMultiProof::leaf_indices`].
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyTree`] for an empty tree,
    /// [`MerkleError::InvalidProofStructure`] when `indices` is empty, and
    /// [`MerkleError::IndexOutOfBounds`] for an index past the last leaf.
    pub fn generate_multi_proof(
        &self,
        indices: &[LeafIndex],
    ) -> Result<BinaryMultiProof<H::Digest>, MerkleError> {
        if self.is_empty() {
            return Err(MerkleError::EmptyTree);
        }
        let mut leaf_indices: Vec<usize> = indices.iter().map(|index| index.value()).collect();
        leaf_indices.sort_unstable();
        leaf_indices.dedup();
        if leaf_indices.is_empty() {
            return Err(MerkleError::InvalidProofStructure(
                "a multiproof needs at least one leaf".to_string(),
            ));
        }
        if let Some(&index) = leaf_indices.iter().find(|&&index| index >= self.leaf_count) {
            return Err(MerkleError::IndexOutOfBounds {
                index,
                len: self.leaf_count,
            });
        }

        let mut hashes = Vec::new();
        let mut known = leaf_indices.clone();
        let mut layer_start = 0;
        let mut layer_width = self.leaf_capacity();
        while layer_width > 1 {
            let mut next = Vec::with_capacity(known.len());
            let mut i = 0;
            while i < known.len() {
                let offset = known[i];
                if offset.is_multiple_of(2) && known.get(i + 1) == Some(&(offset + 1)) {
                    i += 1;
                } else {
                    hashes.push(self.nodes[layer_start + (offset ^ 1)].clone());
                }
                next.push(offset / 2);
                i += 1;
            }
            known = next;
            layer_start += layer_width;
            layer_width /= 2;
        }

        Ok(BinaryMultiProof {
            leaf_indices,
            leaf_count: self.leaf_count,
            hashes,
        })
    }

    /// Verifies a multiproof for `leaves`, the raw leaf data in
    /// [`BinaryMultiProof::leaf_indices`] order.
    ///
    /// Returns `false` for any malformed proof: unsorted or repeated indices,
    /// an index past `leaf_count`, a leaf count that does not match the
    /// number of indices, empty leaf data, or unused or missing hashes.
    #[must_use]
    pub fn verify_multi_proof<L: AsRef<[u8]>>(
        expected_root: &H::Digest,
        leaves: &[L],
        proof: &BinaryMultiProof<H::Digest>,
    ) -> bool {
        let indices = &proof.leaf_indices;
        if proof.leaf_count == 0
            || indices.is_empty()
            || indices.len() != leaves.len()
            || indices.windows(2).any(|pair| pair[0] >= pair[1])
            || indices.last().is_some_and(|&last| last >= proof.leaf_count)
            || leaves.iter().any(|leaf| leaf.as_ref().is_empty())
        {
            return false;
        }
        let Some(mut layer_width) = proof.leaf_count.checked_next_power_of_two() else {
            return false;
        };

        let mut known: Vec<(usize, H::Digest)> = indices
            .iter()
            .zip(leaves)
            .map(|(&index, leaf)| (index, H::hash(leaf.as_ref())))
            .collect();
        let mut hashes = proof.hashes.iter();
        while layer_width > 1 {
            let mut next = Vec::with_capacity(known.len());
            let mut i = 0;
            while i < known.len() {
                let (offset, ref hash) = known[i];
                let parent = match known.get(i + 1) {
                    Some((right, right_hash))
                        if offset.is_multiple_of(2) && *right == offset + 1 =>
                    {
                        i += 1;
                        H::hash_nodes(hash, right_hash)
                    }
                    _ => {
                        let Some(sibling) = hashes.next() else {
                            return false;
                        };
                        if offset.is_multiple_of(2) {
                            H::hash_nodes(hash, sibling)
                        } else {
                            H::hash_nodes(sibling, hash)
                        }
                    }
                };
                next.push((offset / 2, parent));
                i += 1;
            }
            known = next;
            layer_width /= 2;
        }

        hashes.next().is_none() && matches!(known.as_slice(), [(_, root)] if root == expected_root)
    }

    fn from_leaf_hashes(hashes: Vec<H::Digest>) -> Self {
        let mut tree = Self::new();
        if hashes.is_empty() {
            return tree;
        }

        let leaf_capacity = Self::padded_leaf_count(hashes.len());
        tree.leaf_count = hashes.len();
        tree.nodes = hashes;
        tree.nodes
            .resize(Self::node_count(leaf_capacity), H::empty());
        tree.trim_empty_trailing_leaves();

        if tree.leaf_count == 0 {
            return Self::new();
        }

        let required_capacity = Self::padded_leaf_count(tree.leaf_count);
        if required_capacity < leaf_capacity {
            tree.resize_leaf_layer(required_capacity);
        } else {
            tree.recompute_root();
        }
        tree.height = required_capacity.ilog2() as usize + 1;

        tree
    }

    const fn padded_leaf_count(leaf_count: usize) -> usize {
        leaf_count
            .checked_next_power_of_two()
            .expect("binary Merkle tree capacity exceeds usize")
    }

    fn node_count(leaf_capacity: usize) -> usize {
        leaf_capacity
            .checked_mul(2)
            .and_then(|count| count.checked_sub(1))
            .expect("binary Merkle tree node count exceeds usize")
    }

    const fn leaf_capacity(&self) -> usize {
        self.nodes.len().div_ceil(2)
    }

    fn ensure_leaf_capacity(&mut self, required: usize) {
        if required > self.leaf_capacity() {
            self.resize_leaf_layer(Self::padded_leaf_count(required));
        }
    }

    fn resize_leaf_layer(&mut self, new_capacity: usize) {
        let leaves_to_copy = self.leaf_count.min(new_capacity);
        let mut nodes = vec![H::empty(); Self::node_count(new_capacity)];

        if leaves_to_copy > 0 {
            nodes[..leaves_to_copy].clone_from_slice(&self.nodes[..leaves_to_copy]);
        }

        self.nodes = nodes;
        self.recompute_root();
    }

    fn trim_empty_trailing_leaves(&mut self) {
        let empty = H::empty();
        while self.leaf_count > 0 && self.nodes[self.leaf_count - 1] == empty {
            self.leaf_count -= 1;
        }
    }

    fn recompute_root(&mut self) {
        let leaf_capacity = self.leaf_capacity();
        if leaf_capacity == 0 {
            return;
        }

        let mut layer_start = 0;
        let mut layer_width = leaf_capacity;
        let mut parent_start = leaf_capacity;

        while layer_width > 1 {
            for offset in 0..(layer_width / 2) {
                let left = self.nodes[layer_start + offset * 2].clone();
                let right = self.nodes[layer_start + offset * 2 + 1].clone();
                self.nodes[parent_start + offset] = H::hash_nodes(&left, &right);
            }

            layer_start = parent_start;
            layer_width /= 2;
            parent_start += layer_width;
        }
    }

    fn recompute_path(&mut self, leaf_index: usize) {
        let mut layer_start = 0;
        let mut layer_width = self.leaf_capacity();
        let mut parent_start = layer_width;
        let mut offset = leaf_index;

        while layer_width > 1 {
            let pair_offset = offset & !1;
            let left = self.nodes[layer_start + pair_offset].clone();
            let right = self.nodes[layer_start + pair_offset + 1].clone();
            self.nodes[parent_start + offset / 2] = H::hash_nodes(&left, &right);

            offset /= 2;
            layer_start = parent_start;
            layer_width /= 2;
            parent_start += layer_width;
        }
    }
}

impl<H: HashFunction> MerkleTree<H> for BinaryMerkleTree<H> {
    fn insert(&mut self, data: &[u8]) -> Result<LeafIndex, MerkleError> {
        Self::insert(self, data)
    }

    fn remove(&mut self, index: LeafIndex) -> Result<(), MerkleError> {
        Self::remove(self, index)
    }

    fn root(&self) -> Option<&H::Digest> {
        Self::root(self)
    }

    fn leaf_count(&self) -> usize {
        Self::leaf_count(self)
    }

    fn is_empty(&self) -> bool {
        Self::is_empty(self)
    }

    fn height(&self) -> usize {
        Self::height(self)
    }

    fn generate_proof(&self, index: LeafIndex) -> Result<MerkleProof<H::Digest>, MerkleError> {
        Self::generate_proof(self, index)
    }

    fn metadata(&self) -> TreeMetadata {
        TreeMetadata {
            leaf_count: self.leaf_count,
            height: self.height,
            node_count: self.nodes.len(),
            hash_algorithm: H::algorithm_name(),
            variant: "BinaryMerkleTree",
        }
    }
}

impl<H: HashFunction> ProofVerifier<H> for BinaryMerkleTree<H> {
    fn verify(expected_root: &H::Digest, leaf_data: &[u8], proof: &MerkleProof<H::Digest>) -> bool {
        if leaf_data.is_empty()
            || proof.leaf_count == 0
            || proof.leaf_index.value() >= proof.leaf_count
        {
            return false;
        }

        let Some(leaf_capacity) = proof.leaf_count.checked_next_power_of_two() else {
            return false;
        };
        let expected_depth = leaf_capacity.ilog2() as usize;
        if proof.path.len() != expected_depth {
            return false;
        }

        let mut current = H::hash(leaf_data);
        let mut offset = proof.leaf_index.value();

        for node in &proof.path {
            let expected_side = if offset.is_multiple_of(2) {
                ProofSide::Right
            } else {
                ProofSide::Left
            };
            if node.side != expected_side {
                return false;
            }

            current = match node.side {
                ProofSide::Left => H::hash_nodes(&node.hash, &current),
                ProofSide::Right => H::hash_nodes(&current, &node.hash),
            };
            offset /= 2;
        }

        current == *expected_root
    }
}

impl<H: HashFunction> Default for BinaryMerkleTree<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: HashFunction> Clone for BinaryMerkleTree<H> {
    fn clone(&self) -> Self {
        Self {
            nodes: self.nodes.clone(),
            leaf_count: self.leaf_count,
            height: self.height,
        }
    }
}

impl<H: HashFunction> fmt::Debug for BinaryMerkleTree<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BinaryMerkleTree")
            .field("hash_algorithm", &H::algorithm_name())
            .field("leaf_count", &self.leaf_count)
            .field("height", &self.height)
            .field("root", &self.root())
            .finish_non_exhaustive()
    }
}

#[derive(Serialize)]
struct BinaryTreeReprRef<'a, D> {
    hash_algorithm: &'a str,
    leaves: &'a [D],
}

#[derive(Deserialize)]
struct BinaryTreeRepr<D> {
    hash_algorithm: String,
    leaves: Vec<D>,
}

/// Serializes the hash algorithm name and the leaf layer.
///
/// Internal nodes are not stored; deserialization recomputes them, so a
/// decoded tree is always internally consistent.
impl<H: HashFunction> Serialize for BinaryMerkleTree<H>
where
    H::Digest: Serialize,
{
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        BinaryTreeReprRef {
            hash_algorithm: H::algorithm_name(),
            leaves: &self.nodes[..self.leaf_count],
        }
        .serialize(serializer)
    }
}

impl<'de, H: HashFunction> Deserialize<'de> for BinaryMerkleTree<H>
where
    H::Digest: Deserialize<'de>,
{
    fn deserialize<De: Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        let repr = BinaryTreeRepr::<H::Digest>::deserialize(deserializer)?;
        if repr.hash_algorithm != H::algorithm_name() {
            return Err(serde::de::Error::custom(format!(
                "tree was built with {}, expected {}",
                repr.hash_algorithm,
                H::algorithm_name()
            )));
        }
        if repr.leaves.len().checked_next_power_of_two().is_none() {
            return Err(serde::de::Error::custom("too many leaves"));
        }

        Ok(Self::from_leaf_hashes(repr.leaves))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use merkleforge_hash::Sha256;

    #[test]
    fn new_tree_is_empty() {
        let tree = BinaryMerkleTree::<Sha256>::new();

        assert!(tree.is_empty());
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.root(), None);
        assert!(tree.nodes.is_empty());
    }

    #[test]
    fn zero_capacity_matches_new() {
        let tree = BinaryMerkleTree::<Sha256>::with_capacity(0);

        assert!(tree.is_empty());
        assert_eq!(tree.height(), 0);
        assert!(tree.nodes.is_empty());
    }

    #[test]
    fn capacity_is_padded_to_next_power_of_two() {
        let tree = BinaryMerkleTree::<Sha256>::with_capacity(3);

        assert!(tree.is_empty());
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.root(), None);
        assert_eq!(tree.nodes.len(), 7);
        assert!(tree.nodes[..4].iter().all(|node| node == &Sha256::empty()));
    }

    #[test]
    fn empty_internal_nodes_are_built_bottom_up() {
        let tree = BinaryMerkleTree::<Sha256>::with_capacity(4);
        let empty = Sha256::empty();
        let parent = Sha256::hash_nodes(&empty, &empty);
        let expected_root = Sha256::hash_nodes(&parent, &parent);

        assert_eq!(tree.nodes[4], parent);
        assert_eq!(tree.nodes[5], parent);
        assert_eq!(tree.nodes[6], expected_root);
    }

    #[test]
    fn inserting_one_leaf_produces_root() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        let index = tree.insert(b"alice").unwrap();

        assert_eq!(index, LeafIndex(0));
        assert_eq!(tree.root(), Some(&Sha256::hash(b"alice")));
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.height(), 1);
        assert!(!tree.is_empty());
    }

    #[test]
    fn insertion_resizes_and_recomputes_root() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        tree.insert(b"carol").unwrap();

        let alice = Sha256::hash(b"alice");
        let bob = Sha256::hash(b"bob");
        let carol = Sha256::hash(b"carol");
        let left = Sha256::hash_nodes(&alice, &bob);
        let right = Sha256::hash_nodes(&carol, &Sha256::empty());
        let expected = Sha256::hash_nodes(&left, &right);

        assert_eq!(tree.root(), Some(&expected));
        assert_eq!(tree.leaf_count(), 3);
        assert_eq!(tree.height(), 3);
        assert_eq!(tree.nodes.len(), 7);
    }

    #[test]
    fn empty_leaf_data_is_rejected() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();

        assert_eq!(tree.insert(b""), Err(MerkleError::EmptyLeafData));
        assert!(tree.is_empty());
    }

    #[test]
    fn removing_only_leaf_returns_tree_to_empty_state() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        let index = tree.insert(b"alice").unwrap();
        tree.remove(index).unwrap();

        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
        assert_eq!(tree.leaf_count(), 0);
        assert_eq!(tree.height(), 0);
        assert_eq!(tree.nodes[0], Sha256::empty());
    }

    #[test]
    fn removing_interior_leaf_preserves_later_indices() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        tree.insert(b"carol").unwrap();
        tree.remove(LeafIndex(1)).unwrap();

        let alice = Sha256::hash(b"alice");
        let carol = Sha256::hash(b"carol");
        let left = Sha256::hash_nodes(&alice, &Sha256::empty());
        let right = Sha256::hash_nodes(&carol, &Sha256::empty());
        let expected = Sha256::hash_nodes(&left, &right);

        assert_eq!(tree.root(), Some(&expected));
        assert_eq!(tree.leaf_count(), 3);
    }

    #[test]
    fn removing_trailing_leaf_shrinks_tree() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        tree.remove(LeafIndex(1)).unwrap();

        assert_eq!(tree.root(), Some(&Sha256::hash(b"alice")));
        assert_eq!(tree.leaf_count(), 1);
        assert_eq!(tree.height(), 1);
        assert_eq!(tree.nodes.len(), 1);
    }

    #[test]
    fn out_of_bounds_remove_is_rejected() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();

        assert_eq!(
            tree.remove(LeafIndex(1)),
            Err(MerkleError::IndexOutOfBounds { index: 1, len: 1 })
        );
    }

    #[test]
    fn removing_from_empty_tree_is_rejected() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();

        assert_eq!(tree.remove(LeafIndex(0)), Err(MerkleError::EmptyTree));
    }

    #[test]
    fn trait_methods_delegate_to_binary_tree() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        let index =
            <BinaryMerkleTree<Sha256> as MerkleTree<Sha256>>::insert(&mut tree, b"trait leaf")
                .unwrap();

        assert_eq!(index, LeafIndex(0));
        assert!(<BinaryMerkleTree<Sha256> as MerkleTree<Sha256>>::root(&tree).is_some());

        let metadata = tree.metadata();
        assert_eq!(metadata.leaf_count, 1);
        assert_eq!(metadata.height, 1);
        assert_eq!(metadata.node_count, 1);
        assert_eq!(metadata.hash_algorithm, "SHA-256");
        assert_eq!(metadata.variant, "BinaryMerkleTree");
    }

    #[test]
    fn proof_generation_rejects_empty_tree() {
        let tree = BinaryMerkleTree::<Sha256>::new();

        assert_eq!(
            tree.generate_proof(LeafIndex(0)),
            Err(MerkleError::EmptyTree)
        );
    }

    #[test]
    fn proof_generation_rejects_out_of_bounds_index() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();

        assert_eq!(
            tree.generate_proof(LeafIndex(1)),
            Err(MerkleError::IndexOutOfBounds { index: 1, len: 1 })
        );
    }

    #[test]
    fn single_leaf_proof_is_trivial() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();

        let proof = tree.generate_proof(LeafIndex(0)).unwrap();

        assert!(proof.is_trivial());
        assert_eq!(proof.depth(), 0);
        assert_eq!(proof.leaf_index, LeafIndex(0));
        assert_eq!(proof.leaf_count, 1);
    }

    #[test]
    fn proof_path_has_one_node_per_non_leaf_level() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        for data in [b"alice".as_slice(), b"bob", b"carol"] {
            tree.insert(data).unwrap();
        }

        for index in 0..tree.leaf_count() {
            let proof = tree.generate_proof(LeafIndex(index)).unwrap();
            assert_eq!(proof.depth(), tree.height() - 1);
        }
    }

    #[test]
    fn proof_records_correct_siblings_and_sides() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        for data in [b"alice".as_slice(), b"bob", b"carol", b"dave"] {
            tree.insert(data).unwrap();
        }

        let alice = Sha256::hash(b"alice");
        let bob = Sha256::hash(b"bob");
        let carol = Sha256::hash(b"carol");
        let dave = Sha256::hash(b"dave");
        let left_parent = Sha256::hash_nodes(&alice, &bob);
        let right_parent = Sha256::hash_nodes(&carol, &dave);

        let left_proof = tree.generate_proof(LeafIndex(0)).unwrap();
        assert_eq!(
            left_proof.path,
            vec![
                ProofNode {
                    hash: bob,
                    side: ProofSide::Right,
                },
                ProofNode {
                    hash: right_parent,
                    side: ProofSide::Right,
                },
            ]
        );

        let right_proof = tree.generate_proof(LeafIndex(3)).unwrap();
        assert_eq!(
            right_proof.path,
            vec![
                ProofNode {
                    hash: carol,
                    side: ProofSide::Left,
                },
                ProofNode {
                    hash: left_parent,
                    side: ProofSide::Left,
                },
            ]
        );
    }

    #[test]
    fn every_valid_index_produces_a_root_reconstructing_proof() {
        let data = [b"alice".as_slice(), b"bob", b"carol", b"dave", b"eve"];
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        for leaf in data {
            tree.insert(leaf).unwrap();
        }

        for (index, leaf) in data.iter().enumerate() {
            let proof = tree.generate_proof(LeafIndex(index)).unwrap();
            let reconstructed =
                proof
                    .path
                    .iter()
                    .fold(Sha256::hash(leaf), |current, node| match node.side {
                        ProofSide::Left => Sha256::hash_nodes(&node.hash, &current),
                        ProofSide::Right => Sha256::hash_nodes(&current, &node.hash),
                    });

            assert_eq!(Some(&reconstructed), tree.root());
        }
    }

    #[test]
    fn valid_proof_verifies() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        for data in [b"alice".as_slice(), b"bob", b"carol"] {
            tree.insert(data).unwrap();
        }
        let proof = tree.generate_proof(LeafIndex(1)).unwrap();

        assert!(BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"bob",
            &proof
        ));
    }

    #[test]
    fn wrong_leaf_data_fails_verification() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        let proof = tree.generate_proof(LeafIndex(0)).unwrap();

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"mallory",
            &proof
        ));
    }

    #[test]
    fn tampered_sibling_fails_verification() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        let mut proof = tree.generate_proof(LeafIndex(0)).unwrap();
        proof.path[0].hash = Sha256::hash(b"tampered");

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"alice",
            &proof
        ));
    }

    #[test]
    fn proof_from_different_tree_fails_verification() {
        let mut first = BinaryMerkleTree::<Sha256>::new();
        first.insert(b"alice").unwrap();
        first.insert(b"bob").unwrap();
        let proof = first.generate_proof(LeafIndex(0)).unwrap();

        let mut second = BinaryMerkleTree::<Sha256>::new();
        second.insert(b"alice").unwrap();
        second.insert(b"carol").unwrap();

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            second.root().unwrap(),
            b"alice",
            &proof
        ));
    }

    #[test]
    fn trivial_proof_verifies() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        let proof = tree.generate_proof(LeafIndex(0)).unwrap();

        assert!(proof.is_trivial());
        assert!(BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"alice",
            &proof
        ));
    }

    #[test]
    fn invalid_proof_index_returns_false() {
        let proof = MerkleProof {
            leaf_index: LeafIndex(1),
            leaf_count: 1,
            path: Vec::new(),
        };

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            &Sha256::hash(b"alice"),
            b"alice",
            &proof
        ));
    }

    #[test]
    fn invalid_path_length_returns_false() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        let mut proof = tree.generate_proof(LeafIndex(0)).unwrap();
        proof.path.clear();

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"alice",
            &proof
        ));
    }

    fn sequential(leaves: &[&[u8]]) -> BinaryMerkleTree<Sha256> {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        for leaf in leaves {
            tree.insert(leaf).unwrap();
        }
        tree
    }

    #[test]
    fn from_leaves_matches_sequential_inserts() {
        let leaves: Vec<Vec<u8>> = (0_u32..37).map(|i| i.to_be_bytes().to_vec()).collect();
        for len in 0..leaves.len() {
            let slices: Vec<&[u8]> = leaves[..len].iter().map(Vec::as_slice).collect();
            let bulk = BinaryMerkleTree::<Sha256>::from_leaves(&slices).unwrap();
            let expected = sequential(&slices);

            assert_eq!(bulk.root(), expected.root(), "len {len}");
            assert_eq!(bulk.leaf_count(), expected.leaf_count());
            assert_eq!(bulk.height(), expected.height());
            assert_eq!(bulk.nodes, expected.nodes);
        }
    }

    #[test]
    fn from_leaves_rejects_empty_leaf() {
        let result = BinaryMerkleTree::<Sha256>::from_leaves([b"alice".as_slice(), b""]);

        assert_eq!(result.err(), Some(MerkleError::EmptyLeafData));
    }

    #[test]
    fn clone_is_independent() {
        let original = sequential(&[b"alice", b"bob"]);
        let mut copy = original.clone();
        copy.insert(b"carol").unwrap();

        assert_eq!(original.leaf_count(), 2);
        assert_ne!(original.root(), copy.root());
    }

    #[test]
    fn debug_summarizes_tree() {
        let tree = sequential(&[b"alice"]);
        let debug = format!("{tree:?}");

        assert!(debug.contains("BinaryMerkleTree"));
        assert!(debug.contains("leaf_count: 1"));
    }

    #[test]
    fn serialization_round_trips_with_removed_leaves() {
        use merkle_core::traits::Serializable;

        let mut tree = sequential(&[b"alice", b"bob", b"carol", b"dave", b"eve"]);
        tree.remove(LeafIndex(1)).unwrap();

        let bytes = tree.to_bytes().unwrap();
        let decoded = BinaryMerkleTree::<Sha256>::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.root(), tree.root());
        assert_eq!(decoded.leaf_count(), tree.leaf_count());
        assert_eq!(decoded.height(), tree.height());
        assert_eq!(decoded.nodes, tree.nodes);
    }

    #[test]
    fn deserialization_rejects_other_hash_algorithm() {
        use merkle_core::traits::Serializable;
        use merkleforge_hash::Blake3;

        let bytes = sequential(&[b"alice"]).to_bytes().unwrap();

        assert!(BinaryMerkleTree::<Blake3>::from_bytes(&bytes).is_err());
    }

    #[test]
    fn deserialization_drops_trailing_empty_leaves() {
        use merkle_core::traits::Serializable;

        let repr = (
            "SHA-256".to_string(),
            vec![Sha256::hash(b"alice"), Sha256::empty(), Sha256::empty()],
        );
        let bytes = repr.to_bytes().unwrap();
        let decoded = BinaryMerkleTree::<Sha256>::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.root(), sequential(&[b"alice"]).root());
        assert_eq!(decoded.leaf_count(), 1);
    }

    #[test]
    fn invalid_sibling_side_returns_false() {
        let mut tree = BinaryMerkleTree::<Sha256>::new();
        tree.insert(b"alice").unwrap();
        tree.insert(b"bob").unwrap();
        let mut proof = tree.generate_proof(LeafIndex(0)).unwrap();
        proof.path[0].side = ProofSide::Left;

        assert!(!BinaryMerkleTree::<Sha256>::verify(
            tree.root().unwrap(),
            b"alice",
            &proof
        ));
    }

    fn multiproof_tree(n: usize) -> (Vec<Vec<u8>>, BinaryMerkleTree<Sha256>) {
        let leaves: Vec<Vec<u8>> = (0..n).map(|i| format!("leaf-{i}").into_bytes()).collect();
        let tree = BinaryMerkleTree::<Sha256>::from_leaves(&leaves).unwrap();
        (leaves, tree)
    }

    fn picked<'a>(leaves: &'a [Vec<u8>], proof: &BinaryMultiProof<[u8; 32]>) -> Vec<&'a [u8]> {
        proof
            .leaf_indices
            .iter()
            .map(|&i| leaves[i].as_slice())
            .collect()
    }

    #[test]
    fn multiproof_verifies_any_subset() {
        let (leaves, tree) = multiproof_tree(11);
        let root = *tree.root().unwrap();
        for subset in [
            vec![0],
            vec![10],
            vec![3, 4],
            vec![0, 5, 10],
            (0..11).collect(),
        ] {
            let indices: Vec<LeafIndex> = subset.into_iter().map(LeafIndex).collect();
            let proof = tree.generate_multi_proof(&indices).unwrap();
            assert!(BinaryMerkleTree::<Sha256>::verify_multi_proof(
                &root,
                &picked(&leaves, &proof),
                &proof
            ));
        }
    }

    #[test]
    fn multiproof_for_one_leaf_matches_the_single_proof() {
        let (_, tree) = multiproof_tree(11);
        let multi = tree.generate_multi_proof(&[LeafIndex(6)]).unwrap();
        let single = tree.generate_proof(LeafIndex(6)).unwrap();
        let path: Vec<[u8; 32]> = single.path.into_iter().map(|node| node.hash).collect();
        assert_eq!(multi.hashes, path);
    }

    #[test]
    fn multiproof_shares_siblings_and_normalizes_indices() {
        let (_, tree) = multiproof_tree(11);
        let proof = tree
            .generate_multi_proof(&[
                LeafIndex(7),
                LeafIndex(4),
                LeafIndex(6),
                LeafIndex(5),
                LeafIndex(7),
            ])
            .unwrap();
        assert_eq!(proof.leaf_indices, vec![4, 5, 6, 7]);
        assert_eq!(proof.hashes.len(), 2);
        let all: Vec<LeafIndex> = (0..8).map(LeafIndex).collect();
        let full =
            BinaryMerkleTree::<Sha256>::from_leaves((0..8).map(|i| format!("leaf-{i}"))).unwrap();
        assert!(full.generate_multi_proof(&all).unwrap().hashes.is_empty());
    }

    #[test]
    fn multiproof_errors() {
        let (_, tree) = multiproof_tree(11);
        assert!(matches!(
            tree.generate_multi_proof(&[LeafIndex(11)]),
            Err(MerkleError::IndexOutOfBounds { index: 11, len: 11 })
        ));
        assert!(matches!(
            tree.generate_multi_proof(&[]),
            Err(MerkleError::InvalidProofStructure(_))
        ));
        assert_eq!(
            BinaryMerkleTree::<Sha256>::new().generate_multi_proof(&[LeafIndex(0)]),
            Err(MerkleError::EmptyTree)
        );
    }

    #[test]
    fn multiproof_rejects_tampering() {
        let (leaves, tree) = multiproof_tree(11);
        let root = *tree.root().unwrap();
        let proof = tree
            .generate_multi_proof(&[LeafIndex(1), LeafIndex(4), LeafIndex(9)])
            .unwrap();
        let items = picked(&leaves, &proof);
        let verify = |p: &BinaryMultiProof<[u8; 32]>, l: &[&[u8]]| {
            BinaryMerkleTree::<Sha256>::verify_multi_proof(&root, l, p)
        };
        assert!(verify(&proof, &items));

        let mut forged = items.clone();
        forged[0] = b"forged";
        assert!(!verify(&proof, &forged));
        assert!(!verify(&proof, &items[1..]));

        let mut flipped = proof.clone();
        flipped.hashes[0][0] ^= 1;
        assert!(!verify(&flipped, &items));

        let mut missing = proof.clone();
        missing.hashes.pop();
        assert!(!verify(&missing, &items));

        let mut extra = proof.clone();
        extra.hashes.push(root);
        assert!(!verify(&extra, &items));

        let mut unsorted = proof.clone();
        unsorted.leaf_indices = vec![4, 1, 9];
        assert!(!verify(&unsorted, &items));

        let mut deeper = proof.clone();
        deeper.leaf_count = 17;
        assert!(!verify(&deeper, &items));

        let mut empty = proof;
        empty.leaf_count = 0;
        assert!(!verify(&empty, &items));
    }
}
