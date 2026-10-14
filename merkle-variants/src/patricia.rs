//! Merkle Patricia trie implementation.
//!
//! [`MerklePatriciaTrie`] models Ethereum's nibble-addressed Merkle Patricia
//! Trie shape. The implementation includes nibble-path utilities, RLP/HP
//! encoding, Ethereum-style node references, root hashing, and basic
//! key-value mutation.

use core::{fmt, marker::PhantomData};

use merkle_core::{
    error::MerkleError,
    traits::{HashFunction, MerkleTree},
    types::{LeafIndex, MerkleProof, TreeMetadata},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const EMPTY_RLP: u8 = 0x80;
const SHORT_STRING_OFFSET: u8 = 0x80;
const SHORT_LIST_OFFSET: u8 = 0xC0;
const LONG_STRING_OFFSET: u8 = 0xB7;
const LONG_LIST_OFFSET: u8 = 0xF7;

/// A sequence of 4-bit nibbles extracted from a byte key.
///
/// Each stored nibble is guaranteed to be in the range `0x0..=0xF` when
/// constructed through [`Self::from_key`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NibblePath {
    nibbles: Vec<u8>,
}

impl NibblePath {
    /// Converts a byte key into a high-nibble-first path.
    ///
    /// Every input byte contributes two nibbles. For example, `b"key"` has
    /// three bytes and therefore produces six nibbles.
    #[must_use]
    pub fn from_key(key: &[u8]) -> Self {
        let mut nibbles = Vec::with_capacity(key.len() * 2);
        for byte in key {
            nibbles.push(byte >> 4);
            nibbles.push(byte & 0x0F);
        }

        Self { nibbles }
    }

    fn from_nibbles(nibbles: Vec<u8>) -> Result<Self, MerkleError> {
        if let Some(invalid) = nibbles.iter().find(|nibble| **nibble > 0x0F) {
            return Err(MerkleError::RlpError(format!(
                "invalid nibble value {invalid:#x}"
            )));
        }

        Ok(Self { nibbles })
    }

    /// Returns the number of nibbles in this path.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.nibbles.len()
    }

    /// Returns `true` when this path contains no nibbles.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.nibbles.is_empty()
    }

    /// Returns the nibble at `idx`.
    ///
    /// # Panics
    ///
    /// Panics when `idx` is outside this path.
    #[must_use]
    pub fn get(&self, idx: usize) -> u8 {
        self.nibbles[idx]
    }

    /// Returns a path beginning at nibble `start`.
    ///
    /// # Panics
    ///
    /// Panics when `start` is greater than [`Self::len`].
    #[must_use]
    pub fn slice(&self, start: usize) -> Self {
        Self {
            nibbles: self.nibbles[start..].to_vec(),
        }
    }

    /// Returns the shared prefix length with `other`, measured in nibbles.
    #[must_use]
    pub fn common_prefix_len(&self, other: &Self) -> usize {
        self.nibbles
            .iter()
            .zip(&other.nibbles)
            .take_while(|(left, right)| left == right)
            .count()
    }
}

/// Hex-prefix encodes an MPT nibble path.
///
/// Leaf paths use prefix nibble `0x2` for even paths and `0x3` for odd paths.
/// Extension paths use `0x0` for even paths and `0x1` for odd paths.
#[must_use]
pub fn hp_encode(path: &NibblePath, is_leaf: bool) -> Vec<u8> {
    let odd = path.len() % 2 == 1;
    let mut prefixed = Vec::with_capacity(path.len() + 2);
    let prefix = match (is_leaf, odd) {
        (false, false) => 0x0,
        (false, true) => 0x1,
        (true, false) => 0x2,
        (true, true) => 0x3,
    };

    prefixed.push(prefix);
    if odd {
    } else {
        prefixed.push(0);
    }
    prefixed.extend_from_slice(&path.nibbles);

    nibbles_to_bytes(&prefixed)
}

/// Decodes a hex-prefix encoded MPT path.
///
/// Returns the decoded path and whether the path belongs to a leaf. Malformed
/// input returns an empty extension path instead of panicking; callers that
/// need strict validation should use [`rlp_decode`].
#[must_use]
pub fn hp_decode(bytes: &[u8]) -> (NibblePath, bool) {
    hp_decode_checked(bytes).unwrap_or_default()
}

/// RLP-encodes an MPT node.
#[must_use]
pub fn rlp_encode<D: AsRef<[u8]>>(node: &MptNode<D>) -> Vec<u8> {
    match node {
        MptNode::Empty => vec![EMPTY_RLP],
        MptNode::Leaf { key_suffix, value } => rlp_encode_list(&[
            rlp_encode_bytes(&hp_encode(key_suffix, true)),
            rlp_encode_bytes(value),
        ]),
        MptNode::Extension {
            shared_prefix,
            child,
        } => rlp_encode_list(&[
            rlp_encode_bytes(&hp_encode(shared_prefix, false)),
            rlp_encode_node_ref(child),
        ]),
        MptNode::Branch { children, value } => {
            let mut items = Vec::with_capacity(17);
            items.extend(children.iter().map(rlp_encode_node_ref));
            items.push(
                value
                    .as_deref()
                    .map_or_else(|| vec![EMPTY_RLP], rlp_encode_bytes),
            );

            rlp_encode_list(&items)
        }
    }
}

/// Decodes an RLP-encoded MPT node.
///
/// # Errors
///
/// Returns [`MerkleError::RlpError`] when the input is malformed or does not
/// describe one of the supported MPT node shapes.
pub fn rlp_decode<D>(bytes: &[u8]) -> Result<MptNode<D>, MerkleError>
where
    D: TryFrom<Vec<u8>>,
{
    let (item, consumed) = parse_rlp_item(bytes)?;
    if consumed != bytes.len() {
        return Err(MerkleError::RlpError(
            "trailing bytes after RLP item".to_string(),
        ));
    }

    decode_node_item(&item)
}

/// A lazily materialized reference to an MPT node.
///
/// Ethereum embeds small node RLP directly in the parent and stores only the
/// hash for larger nodes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeRef<D> {
    /// Hash of a large node's RLP encoding.
    Hash(D),
    /// Raw RLP encoding of a small node.
    Inline(Vec<u8>),
}

/// A Merkle Patricia Trie witness for one key.
///
/// The proof nodes are RLP-encoded MPT nodes ordered from root to the terminal
/// node reached by the key path. `value = None` represents a non-membership
/// proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MptProof<D> {
    /// Raw key bytes this proof is for.
    pub key: Vec<u8>,
    /// Stored value for membership proofs, or `None` for non-membership.
    pub value: Option<Vec<u8>>,
    /// RLP-encoded witness nodes, ordered root first.
    pub proof_nodes: Vec<Vec<u8>>,
    #[serde(skip)]
    _marker: PhantomData<D>,
}

impl<D> MptProof<D> {
    /// Creates a proof from its key, optional value, and RLP witness nodes.
    #[must_use]
    pub const fn new(key: Vec<u8>, value: Option<Vec<u8>>, proof_nodes: Vec<Vec<u8>>) -> Self {
        Self {
            key,
            value,
            proof_nodes,
            _marker: PhantomData,
        }
    }

    /// Returns `true` when this proof contains a concrete value.
    #[must_use]
    pub const fn is_membership(&self) -> bool {
        self.value.is_some()
    }

    /// Returns `true` when this proof proves absence for the key.
    #[must_use]
    pub const fn is_non_membership(&self) -> bool {
        self.value.is_none()
    }
}

/// Returns the Ethereum-style node reference for `node`.
///
/// Nodes whose RLP encoding is at least 32 bytes are represented by
/// `H(RLP(node))`; smaller nodes are represented by their raw RLP bytes.
#[must_use]
pub fn hash_node<H>(node: &MptNode<H::Digest>) -> NodeRef<H::Digest>
where
    H: HashFunction,
{
    let rlp = rlp_encode(node);
    if rlp.len() >= 32 {
        NodeRef::Hash(hash_mpt_bytes::<H>(&rlp))
    } else {
        NodeRef::Inline(rlp)
    }
}

/// Computes a full root hash for an MPT root node.
///
/// Unlike child references, the root is always returned as a digest, even when
/// the root node's RLP encoding is short enough to be inlined by a parent.
#[must_use]
pub fn compute_root_hash<H>(root: &MptNode<H::Digest>) -> H::Digest
where
    H: HashFunction,
{
    hash_mpt_bytes::<H>(&rlp_encode(root))
}

/// A Merkle Patricia Trie node.
///
/// The public variants mirror Ethereum's four logical MPT node types: empty,
/// leaf, extension, and branch.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MptNode<D> {
    /// Empty slot with no data at this path.
    Empty,

    /// Leaf node storing the remaining key suffix and value bytes.
    Leaf {
        /// Remaining nibble path after the branch point.
        key_suffix: NibblePath,
        /// Value stored at the complete key.
        value: Vec<u8>,
    },

    /// Extension node compressing a shared nibble prefix.
    Extension {
        /// Shared prefix consumed before following `child`.
        shared_prefix: NibblePath,
        /// Next trie node reference after the compressed prefix.
        child: NodeRef<D>,
    },

    /// Branch node with one child slot per nibble plus an optional value.
    Branch {
        /// Children indexed by nibble `0x0..=0xF`.
        children: Box<[NodeRef<D>; 16]>,
        /// Value stored exactly at this branch path, if any.
        value: Option<Vec<u8>>,
    },
}

impl<D> MptNode<D> {
    #[cfg(test)]
    fn empty_children() -> Box<[NodeRef<D>; 16]> {
        Box::new(std::array::from_fn(|_| NodeRef::Inline(vec![EMPTY_RLP])))
    }
}

/// Ethereum-style Merkle Patricia Trie over nibble-addressed keys.
///
/// The trie supports key-value insertion, removal, lookup, RLP witness
/// generation, and Ethereum-compatible root hashing when used with
/// `Keccak-256`.
///
/// Nodes form an owned tree, and every node caches its Ethereum node
/// reference (inline RLP or hash). A mutation therefore re-encodes and
/// re-hashes only the nodes on the changed key path.
pub struct MerklePatriciaTrie<H: HashFunction> {
    root: Option<Box<TrieNode<H::Digest>>>,
    root_hash: Option<H::Digest>,
    len: usize,
    next_index_hint: usize,
}

/// A trie node together with values cached from its subtree.
#[derive(Clone)]
struct TrieNode<D> {
    kind: TrieNodeKind<D>,
    /// How a parent embeds this node: inline RLP, or the hash of its RLP.
    reference: NodeRef<D>,
    /// Number of nodes in this subtree, including this one.
    node_count: usize,
    /// Longest node path from this node to a leaf, including this node.
    height: usize,
}

#[derive(Clone)]
enum TrieNodeKind<D> {
    Leaf {
        path: Vec<u8>,
        value: Vec<u8>,
    },
    Extension {
        path: Vec<u8>,
        child: Box<TrieNode<D>>,
    },
    Branch {
        children: [Option<Box<TrieNode<D>>>; 16],
        value: Option<Vec<u8>>,
    },
}

impl<D: Clone> TrieNodeKind<D> {
    /// Converts this node into its public encoding form.
    fn to_mpt(&self) -> MptNode<D> {
        match self {
            Self::Leaf { path, value } => MptNode::Leaf {
                key_suffix: NibblePath {
                    nibbles: path.clone(),
                },
                value: value.clone(),
            },
            Self::Extension { path, child } => MptNode::Extension {
                shared_prefix: NibblePath {
                    nibbles: path.clone(),
                },
                child: child.reference.clone(),
            },
            Self::Branch { children, value } => MptNode::Branch {
                children: Box::new(std::array::from_fn(|slot| {
                    children[slot].as_ref().map_or_else(
                        || NodeRef::Inline(vec![EMPTY_RLP]),
                        |child| child.reference.clone(),
                    )
                })),
                value: value.clone(),
            },
        }
    }
}

type Children<D> = [Option<Box<TrieNode<D>>>; 16];

impl<H: HashFunction> MerklePatriciaTrie<H> {
    /// Creates an empty Merkle Patricia Trie.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            root: None,
            root_hash: None,
            len: 0,
            next_index_hint: 0,
        }
    }

    /// Inserts or updates `value` at `key`.
    ///
    /// Only the nodes on the key's path are re-encoded and re-hashed.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyLeafData`] when `value` is empty.
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), MerkleError> {
        if value.is_empty() {
            return Err(MerkleError::EmptyLeafData);
        }

        if self.get(key).is_none() {
            self.len += 1;
        }
        let path = bytes_to_nibbles(key);
        self.root = Some(Self::insert_at(self.root.take(), &path, value.to_vec()));
        self.refresh_root_hash();

        Ok(())
    }

    /// Removes the value stored at `key`.
    ///
    /// Branch and extension nodes that are no longer needed are collapsed, so
    /// the trie keeps the same canonical shape (and root) as a trie built from
    /// the remaining entries.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::UnsupportedOperation`] when `key` is not present.
    pub fn remove(&mut self, key: &[u8]) -> Result<(), MerkleError> {
        if self.get(key).is_none() {
            return Err(MerkleError::UnsupportedOperation(
                "remove missing Patricia key",
            ));
        }
        let Some(root) = self.root.take() else {
            return Err(MerkleError::EmptyTree);
        };

        let path = bytes_to_nibbles(key);
        self.root = Self::remove_at(*root, &path);
        self.len -= 1;
        self.refresh_root_hash();

        Ok(())
    }

    /// Returns the value stored at `key`, if present.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        let path = bytes_to_nibbles(key);
        let mut remaining = path.as_slice();
        let mut node = self.root.as_deref()?;

        loop {
            match &node.kind {
                TrieNodeKind::Leaf { path, value } => {
                    return (path.as_slice() == remaining).then_some(value.as_slice());
                }
                TrieNodeKind::Extension { path, child } => {
                    remaining = remaining.strip_prefix(path.as_slice())?;
                    node = child;
                }
                TrieNodeKind::Branch { children, value } => match remaining.split_first() {
                    None => return value.as_deref(),
                    Some((nibble, rest)) => {
                        node = children[usize::from(*nibble)].as_deref()?;
                        remaining = rest;
                    }
                },
            }
        }
    }

    /// Generates an MPT witness for `key`.
    ///
    /// The returned proof contains RLP-encoded nodes from the root to the
    /// terminal node reached by the key path. Missing keys produce a
    /// non-membership proof with `value = None`.
    ///
    /// # Errors
    ///
    /// Returns [`MerkleError::EmptyTree`] when the trie is empty.
    pub fn generate_proof(&self, key: &[u8]) -> Result<MptProof<H::Digest>, MerkleError> {
        let Some(root) = self.root.as_deref() else {
            return Err(MerkleError::EmptyTree);
        };

        let path = bytes_to_nibbles(key);
        let mut remaining = path.as_slice();
        let mut node = root;
        let mut proof_nodes = Vec::new();

        let value = loop {
            proof_nodes.push(rlp_encode(&node.kind.to_mpt()));

            match &node.kind {
                TrieNodeKind::Leaf { path, value } => {
                    break (path.as_slice() == remaining).then(|| value.clone());
                }
                TrieNodeKind::Extension { path, child } => {
                    let Some(rest) = remaining.strip_prefix(path.as_slice()) else {
                        break None;
                    };
                    remaining = rest;
                    node = child;
                }
                TrieNodeKind::Branch { children, value } => {
                    let Some((nibble, rest)) = remaining.split_first() else {
                        break value.clone();
                    };
                    let Some(child) = children[usize::from(*nibble)].as_deref() else {
                        break None;
                    };
                    remaining = rest;
                    node = child;
                }
            }
        };

        Ok(MptProof::new(key.to_vec(), value, proof_nodes))
    }

    /// Verifies an MPT proof against a trusted root hash.
    ///
    /// Membership proofs require the final node to contain `proof.value`.
    /// Non-membership proofs require the path to terminate at an empty child,
    /// a missing branch value, or a divergent leaf/extension.
    #[must_use]
    pub fn verify_proof(expected_root: &H::Digest, proof: &MptProof<H::Digest>) -> bool
    where
        H::Digest: TryFrom<Vec<u8>>,
    {
        let Some(root_node) = proof.proof_nodes.first() else {
            return false;
        };
        if hash_mpt_bytes::<H>(root_node) != *expected_root {
            return false;
        }

        let key_path = NibblePath::from_key(&proof.key);
        Self::verify_proof_at(&key_path, proof.value.as_ref(), &proof.proof_nodes, 0)
    }

    /// Returns the current root digest, or `None` for an empty trie.
    #[must_use]
    pub const fn root(&self) -> Option<&H::Digest> {
        self.root_hash.as_ref()
    }

    /// Returns `true` when the trie contains no entries.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the number of key-value entries stored in the trie.
    #[must_use]
    pub const fn leaf_count(&self) -> usize {
        self.len
    }

    /// Returns the number of materialized trie nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.node_count)
    }

    /// Returns the longest node path from the root to any leaf.
    #[must_use]
    pub fn height(&self) -> usize {
        self.root.as_ref().map_or(0, |root| root.height)
    }

    /// Returns every `(key, value)` entry in ascending key order.
    fn entries(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        fn walk<D>(node: &TrieNode<D>, prefix: &mut Vec<u8>, out: &mut Vec<(Vec<u8>, Vec<u8>)>) {
            let start = prefix.len();
            match &node.kind {
                TrieNodeKind::Leaf { path, value } => {
                    prefix.extend_from_slice(path);
                    out.push((nibbles_to_bytes(prefix), value.clone()));
                }
                TrieNodeKind::Extension { path, child } => {
                    prefix.extend_from_slice(path);
                    walk(child, prefix, out);
                }
                TrieNodeKind::Branch { children, value } => {
                    if let Some(value) = value {
                        out.push((nibbles_to_bytes(prefix), value.clone()));
                    }
                    for (nibble, child) in (0_u8..).zip(children) {
                        if let Some(child) = child {
                            prefix.push(nibble);
                            walk(child, prefix, out);
                            prefix.pop();
                        }
                    }
                }
            }
            prefix.truncate(start);
        }

        let mut out = Vec::with_capacity(self.len);
        if let Some(root) = &self.root {
            walk(root, &mut Vec::new(), &mut out);
        }
        out
    }

    fn refresh_root_hash(&mut self) {
        self.root_hash = self.root.as_ref().map(|root| match &root.reference {
            NodeRef::Hash(hash) => hash.clone(),
            // The root is always hashed, even when a parent would inline it.
            NodeRef::Inline(rlp) => hash_mpt_bytes::<H>(rlp),
        });
    }

    fn make(kind: TrieNodeKind<H::Digest>) -> Box<TrieNode<H::Digest>> {
        let reference = hash_node::<H>(&kind.to_mpt());
        let (node_count, height) = match &kind {
            TrieNodeKind::Leaf { .. } => (1, 1),
            TrieNodeKind::Extension { child, .. } => (1 + child.node_count, 1 + child.height),
            TrieNodeKind::Branch { children, .. } => children
                .iter()
                .flatten()
                .fold((1, 1), |(count, height), child| {
                    (count + child.node_count, height.max(1 + child.height))
                }),
        };

        Box::new(TrieNode {
            kind,
            reference,
            node_count,
            height,
        })
    }

    fn leaf(path: Vec<u8>, value: Vec<u8>) -> Box<TrieNode<H::Digest>> {
        Self::make(TrieNodeKind::Leaf { path, value })
    }

    fn empty_children() -> Children<H::Digest> {
        std::array::from_fn(|_| None)
    }

    /// Wraps `node` in an extension for `prefix`, unless `prefix` is empty.
    fn with_prefix(prefix: &[u8], node: Box<TrieNode<H::Digest>>) -> Box<TrieNode<H::Digest>> {
        if prefix.is_empty() {
            node
        } else {
            Self::make(TrieNodeKind::Extension {
                path: prefix.to_vec(),
                child: node,
            })
        }
    }

    /// Puts `value` at `remaining` below a new branch: in the branch value
    /// slot when `remaining` is empty, otherwise in a leaf child.
    fn place(
        children: &mut Children<H::Digest>,
        branch_value: &mut Option<Vec<u8>>,
        remaining: &[u8],
        value: Vec<u8>,
    ) {
        match remaining.split_first() {
            None => *branch_value = Some(value),
            Some((nibble, rest)) => {
                children[usize::from(*nibble)] = Some(Self::leaf(rest.to_vec(), value));
            }
        }
    }

    fn insert_at(
        node: Option<Box<TrieNode<H::Digest>>>,
        path: &[u8],
        value: Vec<u8>,
    ) -> Box<TrieNode<H::Digest>> {
        let Some(node) = node else {
            return Self::leaf(path.to_vec(), value);
        };

        match node.kind {
            TrieNodeKind::Leaf {
                path: leaf_path,
                value: leaf_value,
            } => {
                if leaf_path == path {
                    return Self::leaf(leaf_path, value);
                }

                let common = common_prefix_len(&leaf_path, path);
                let mut children = Self::empty_children();
                let mut branch_value = None;
                Self::place(
                    &mut children,
                    &mut branch_value,
                    &leaf_path[common..],
                    leaf_value,
                );
                Self::place(&mut children, &mut branch_value, &path[common..], value);

                let branch = Self::make(TrieNodeKind::Branch {
                    children,
                    value: branch_value,
                });
                Self::with_prefix(&path[..common], branch)
            }
            TrieNodeKind::Extension {
                path: ext_path,
                child,
            } => {
                let common = common_prefix_len(&ext_path, path);
                if common == ext_path.len() {
                    let child = Self::insert_at(Some(child), &path[common..], value);
                    return Self::make(TrieNodeKind::Extension {
                        path: ext_path,
                        child,
                    });
                }

                let mut children = Self::empty_children();
                let mut branch_value = None;
                let rest = &ext_path[common + 1..];
                children[usize::from(ext_path[common])] = Some(Self::with_prefix(rest, child));
                Self::place(&mut children, &mut branch_value, &path[common..], value);

                let branch = Self::make(TrieNodeKind::Branch {
                    children,
                    value: branch_value,
                });
                Self::with_prefix(&ext_path[..common], branch)
            }
            TrieNodeKind::Branch {
                mut children,
                value: branch_value,
            } => match path.split_first() {
                None => Self::make(TrieNodeKind::Branch {
                    children,
                    value: Some(value),
                }),
                Some((nibble, rest)) => {
                    let slot = usize::from(*nibble);
                    children[slot] = Some(Self::insert_at(children[slot].take(), rest, value));
                    Self::make(TrieNodeKind::Branch {
                        children,
                        value: branch_value,
                    })
                }
            },
        }
    }

    /// Removes the entry at `path`, which must exist below `node`.
    fn remove_at(node: TrieNode<H::Digest>, path: &[u8]) -> Option<Box<TrieNode<H::Digest>>> {
        match node.kind {
            TrieNodeKind::Leaf { .. } => None,
            TrieNodeKind::Extension {
                path: ext_path,
                child,
            } => {
                let child = Self::remove_at(*child, &path[ext_path.len()..])?;
                Some(Self::merge_prefix(ext_path, child))
            }
            TrieNodeKind::Branch {
                mut children,
                mut value,
            } => {
                match path.split_first() {
                    None => value = None,
                    Some((nibble, rest)) => {
                        let slot = usize::from(*nibble);
                        let child = children[slot].take().expect("key path exists");
                        children[slot] = Self::remove_at(*child, rest);
                    }
                }
                Self::normalize_branch(children, value)
            }
        }
    }

    /// Prepends a non-empty `prefix` to `node`, merging it into a leaf or
    /// extension so no extension ever points at another extension or a leaf.
    fn merge_prefix(
        mut prefix: Vec<u8>,
        node: Box<TrieNode<H::Digest>>,
    ) -> Box<TrieNode<H::Digest>> {
        if matches!(node.kind, TrieNodeKind::Branch { .. }) {
            return Self::make(TrieNodeKind::Extension {
                path: prefix,
                child: node,
            });
        }

        match node.kind {
            TrieNodeKind::Leaf { path, value } => {
                prefix.extend_from_slice(&path);
                Self::leaf(prefix, value)
            }
            TrieNodeKind::Extension { path, child } => {
                prefix.extend_from_slice(&path);
                Self::make(TrieNodeKind::Extension {
                    path: prefix,
                    child,
                })
            }
            TrieNodeKind::Branch { .. } => unreachable!("branches are handled above"),
        }
    }

    /// Rebuilds a branch after a removal, collapsing it when it no longer
    /// has at least two occupied slots (children plus value).
    fn normalize_branch(
        mut children: Children<H::Digest>,
        value: Option<Vec<u8>>,
    ) -> Option<Box<TrieNode<H::Digest>>> {
        let mut occupied = children
            .iter()
            .enumerate()
            .filter(|(_, child)| child.is_some());
        let first = occupied.next().map(|(slot, _)| slot);
        let has_second = occupied.next().is_some();

        match (first, has_second, value) {
            (None, _, None) => None,
            (None, _, Some(value)) => Some(Self::leaf(Vec::new(), value)),
            (Some(slot), false, None) => {
                let child = children[slot].take().expect("slot is occupied");
                let nibble = u8::try_from(slot).expect("branch slots are nibbles");
                Some(Self::merge_prefix(vec![nibble], child))
            }
            (_, _, value) => Some(Self::make(TrieNodeKind::Branch { children, value })),
        }
    }

    fn verify_proof_at(
        path: &NibblePath,
        proof_value: Option<&Vec<u8>>,
        proof_nodes: &[Vec<u8>],
        index: usize,
    ) -> bool
    where
        H::Digest: TryFrom<Vec<u8>>,
    {
        let Some(encoded_node) = proof_nodes.get(index) else {
            return false;
        };
        let Ok(node) = rlp_decode::<H::Digest>(encoded_node) else {
            return false;
        };

        match node {
            MptNode::Empty => proof_value.is_none() && index + 1 == proof_nodes.len(),
            MptNode::Leaf { key_suffix, value } => {
                if key_suffix == *path {
                    proof_value == Some(&value) && index + 1 == proof_nodes.len()
                } else {
                    proof_value.is_none() && index + 1 == proof_nodes.len()
                }
            }
            MptNode::Extension {
                shared_prefix,
                child,
            } => {
                if path.common_prefix_len(&shared_prefix) != shared_prefix.len() {
                    return proof_value.is_none() && index + 1 == proof_nodes.len();
                }

                let Some(next_node) = proof_nodes.get(index + 1) else {
                    return false;
                };
                if !Self::node_ref_matches(&child, next_node) {
                    return false;
                }

                Self::verify_proof_at(
                    &path.slice(shared_prefix.len()),
                    proof_value,
                    proof_nodes,
                    index + 1,
                )
            }
            MptNode::Branch { children, value } => {
                if path.is_empty() {
                    return proof_value == value.as_ref() && index + 1 == proof_nodes.len();
                }

                let child_ref = &children[usize::from(path.get(0))];
                if matches!(child_ref, NodeRef::Inline(rlp) if is_rlp_empty(rlp)) {
                    return proof_value.is_none() && index + 1 == proof_nodes.len();
                }

                let Some(next_node) = proof_nodes.get(index + 1) else {
                    return false;
                };
                if !Self::node_ref_matches(child_ref, next_node) {
                    return false;
                }

                Self::verify_proof_at(&path.slice(1), proof_value, proof_nodes, index + 1)
            }
        }
    }

    fn node_ref_matches(node_ref: &NodeRef<H::Digest>, encoded_node: &[u8]) -> bool {
        match node_ref {
            NodeRef::Hash(hash) => hash_mpt_bytes::<H>(encoded_node) == *hash,
            NodeRef::Inline(rlp) => rlp == encoded_node,
        }
    }

    fn key_for_leaf_index(index: LeafIndex) -> Vec<u8> {
        let mut key = b"merkleforge:mpt:trait-leaf:".to_vec();
        key.extend_from_slice(&(index.value() as u64).to_be_bytes());
        key
    }

    /// Returns the key of an occupied trait-addressed entry.
    fn existing_index_key(&self, index: LeafIndex) -> Result<Vec<u8>, MerkleError> {
        if self.is_empty() {
            return Err(MerkleError::EmptyTree);
        }

        let key = Self::key_for_leaf_index(index);
        if self.get(&key).is_some() {
            Ok(key)
        } else {
            Err(MerkleError::IndexOutOfBounds {
                index: index.value(),
                len: self.len,
            })
        }
    }
}

fn common_prefix_len(left: &[u8], right: &[u8]) -> usize {
    left.iter()
        .zip(right)
        .take_while(|(left, right)| left == right)
        .count()
}

impl<H: HashFunction> Default for MerklePatriciaTrie<H> {
    fn default() -> Self {
        Self::new()
    }
}

impl<H: HashFunction> Clone for MerklePatriciaTrie<H> {
    fn clone(&self) -> Self {
        Self {
            root: self.root.clone(),
            root_hash: self.root_hash.clone(),
            len: self.len,
            next_index_hint: self.next_index_hint,
        }
    }
}

impl<H: HashFunction> fmt::Debug for MerklePatriciaTrie<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MerklePatriciaTrie")
            .field("hash_algorithm", &H::algorithm_name())
            .field("leaf_count", &self.len)
            .field("node_count", &self.node_count())
            .field("root", &self.root())
            .finish_non_exhaustive()
    }
}

#[derive(Serialize, Deserialize)]
struct PatriciaTrieRepr {
    hash_algorithm: String,
    entries: Vec<(Vec<u8>, Vec<u8>)>,
}

/// Serializes the hash algorithm name and the `(key, value)` entries in key
/// order.
///
/// Nodes are not serialized; deserialization re-inserts the entries, so a
/// decoded trie is always canonical.
impl<H: HashFunction> Serialize for MerklePatriciaTrie<H> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        PatriciaTrieRepr {
            hash_algorithm: H::algorithm_name().to_string(),
            entries: self.entries(),
        }
        .serialize(serializer)
    }
}

impl<'de, H: HashFunction> Deserialize<'de> for MerklePatriciaTrie<H> {
    fn deserialize<De: Deserializer<'de>>(deserializer: De) -> Result<Self, De::Error> {
        let repr = PatriciaTrieRepr::deserialize(deserializer)?;
        if repr.hash_algorithm != H::algorithm_name() {
            return Err(serde::de::Error::custom(format!(
                "trie was built with {}, expected {}",
                repr.hash_algorithm,
                H::algorithm_name()
            )));
        }

        let mut trie = Self::new();
        for (key, value) in repr.entries {
            if trie.get(&key).is_some() {
                return Err(serde::de::Error::custom("duplicate Patricia key"));
            }
            trie.insert(&key, &value)
                .map_err(serde::de::Error::custom)?;
        }

        Ok(trie)
    }
}

/// Index-addressed adapter over the key-addressed trie.
///
/// Leaf index `i` is stored under the key
/// `b"merkleforge:mpt:trait-leaf:" || i` (8 bytes, big-endian).
/// [`MerkleTree::insert`] returns the first unoccupied index at or above the
/// leaf count, so it never overwrites an existing entry, even after
/// removals. Operations on an index with no entry return
/// [`MerkleError::IndexOutOfBounds`].
impl<H: HashFunction> MerkleTree<H> for MerklePatriciaTrie<H> {
    fn insert(&mut self, data: &[u8]) -> Result<LeafIndex, MerkleError> {
        let mut index = self.len.max(self.next_index_hint);
        while self
            .get(&Self::key_for_leaf_index(LeafIndex(index)))
            .is_some()
        {
            index += 1;
        }

        Self::insert(self, &Self::key_for_leaf_index(LeafIndex(index)), data)?;
        self.next_index_hint = index + 1;

        Ok(LeafIndex(index))
    }

    fn remove(&mut self, index: LeafIndex) -> Result<(), MerkleError> {
        let key = self.existing_index_key(index)?;

        Self::remove(self, &key)
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
        self.existing_index_key(index)?;

        Err(MerkleError::UnsupportedOperation(
            "Patricia trie proofs are key-addressed; use MerklePatriciaTrie::generate_proof",
        ))
    }

    fn metadata(&self) -> TreeMetadata {
        TreeMetadata {
            leaf_count: self.len,
            height: self.height(),
            node_count: self.node_count(),
            hash_algorithm: H::algorithm_name(),
            variant: "MerklePatriciaTrie",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum RlpItem {
    Bytes(Vec<u8>),
    List(Vec<Vec<u8>>),
}

fn nibbles_to_bytes(nibbles: &[u8]) -> Vec<u8> {
    nibbles
        .chunks(2)
        .map(|chunk| {
            let high = chunk[0] << 4;
            let low = chunk.get(1).copied().unwrap_or(0);
            high | low
        })
        .collect()
}

fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    bytes
        .iter()
        .flat_map(|byte| [byte >> 4, byte & 0x0F])
        .collect()
}

fn hp_decode_checked(bytes: &[u8]) -> Result<(NibblePath, bool), MerkleError> {
    let mut nibbles = bytes_to_nibbles(bytes);
    let Some(prefix) = nibbles.first().copied() else {
        return Err(MerkleError::RlpError("empty hex-prefix path".to_string()));
    };

    let (is_leaf, odd) = match prefix {
        0x0 => (false, false),
        0x1 => (false, true),
        0x2 => (true, false),
        0x3 => (true, true),
        _ => {
            return Err(MerkleError::RlpError(format!(
                "invalid hex-prefix nibble {prefix:#x}"
            )));
        }
    };

    let path_nibbles = if odd {
        nibbles.split_off(1)
    } else {
        if nibbles.get(1).copied() != Some(0) {
            return Err(MerkleError::RlpError(
                "even hex-prefix path must include zero padding nibble".to_string(),
            ));
        }
        nibbles.split_off(2)
    };

    Ok((NibblePath::from_nibbles(path_nibbles)?, is_leaf))
}

fn rlp_encode_bytes(bytes: &[u8]) -> Vec<u8> {
    if bytes.len() == 1 && bytes[0] < 0x80 {
        return bytes.to_vec();
    }

    if bytes.len() <= 55 {
        let mut encoded = Vec::with_capacity(bytes.len() + 1);
        encoded.push(SHORT_STRING_OFFSET + u8::try_from(bytes.len()).expect("length <= 55"));
        encoded.extend_from_slice(bytes);
        return encoded;
    }

    let length_bytes = usize_to_be_bytes(bytes.len());
    let mut encoded = Vec::with_capacity(1 + length_bytes.len() + bytes.len());
    encoded.push(LONG_STRING_OFFSET + u8::try_from(length_bytes.len()).expect("usize is short"));
    encoded.extend_from_slice(&length_bytes);
    encoded.extend_from_slice(bytes);
    encoded
}

fn rlp_encode_list(items: &[Vec<u8>]) -> Vec<u8> {
    let payload_len = items.iter().map(Vec::len).sum::<usize>();
    let mut payload = Vec::with_capacity(payload_len);
    for item in items {
        payload.extend_from_slice(item);
    }

    if payload.len() <= 55 {
        let mut encoded = Vec::with_capacity(payload.len() + 1);
        encoded.push(SHORT_LIST_OFFSET + u8::try_from(payload.len()).expect("length <= 55"));
        encoded.extend_from_slice(&payload);
        return encoded;
    }

    let length_bytes = usize_to_be_bytes(payload.len());
    let mut encoded = Vec::with_capacity(1 + length_bytes.len() + payload.len());
    encoded.push(LONG_LIST_OFFSET + u8::try_from(length_bytes.len()).expect("usize is short"));
    encoded.extend_from_slice(&length_bytes);
    encoded.extend_from_slice(&payload);
    encoded
}

fn rlp_encode_node_ref<D: AsRef<[u8]>>(node_ref: &NodeRef<D>) -> Vec<u8> {
    match node_ref {
        NodeRef::Hash(hash) => rlp_encode_bytes(hash.as_ref()),
        NodeRef::Inline(rlp) => rlp.clone(),
    }
}

fn hash_mpt_bytes<H>(bytes: &[u8]) -> H::Digest
where
    H: HashFunction,
{
    H::hash_encoded_node(bytes)
}

fn usize_to_be_bytes(value: usize) -> Vec<u8> {
    let bytes = value.to_be_bytes();
    bytes
        .iter()
        .skip_while(|byte| **byte == 0)
        .copied()
        .collect()
}

fn parse_rlp_item(bytes: &[u8]) -> Result<(RlpItem, usize), MerkleError> {
    let Some(&prefix) = bytes.first() else {
        return Err(MerkleError::RlpError("empty RLP input".to_string()));
    };

    match prefix {
        0x00..=0x7F => Ok((RlpItem::Bytes(vec![prefix]), 1)),
        0x80..=0xB7 => {
            let len = usize::from(prefix - SHORT_STRING_OFFSET);
            let payload = read_payload(bytes, 1, len)?;
            Ok((RlpItem::Bytes(payload.to_vec()), 1 + len))
        }
        0xB8..=0xBF => {
            let len_of_len = usize::from(prefix - LONG_STRING_OFFSET);
            let (len, offset) = read_long_length(bytes, len_of_len)?;
            let payload = read_payload(bytes, offset, len)?;
            Ok((RlpItem::Bytes(payload.to_vec()), offset + len))
        }
        0xC0..=0xF7 => {
            let len = usize::from(prefix - SHORT_LIST_OFFSET);
            let payload = read_payload(bytes, 1, len)?;
            Ok((RlpItem::List(parse_rlp_list_payload(payload)?), 1 + len))
        }
        0xF8..=0xFF => {
            let len_of_len = usize::from(prefix - LONG_LIST_OFFSET);
            let (len, offset) = read_long_length(bytes, len_of_len)?;
            let payload = read_payload(bytes, offset, len)?;
            Ok((
                RlpItem::List(parse_rlp_list_payload(payload)?),
                offset + len,
            ))
        }
    }
}

fn read_payload(bytes: &[u8], offset: usize, len: usize) -> Result<&[u8], MerkleError> {
    bytes
        .get(offset..offset + len)
        .ok_or_else(|| MerkleError::RlpError("RLP payload length exceeds input".to_string()))
}

fn read_long_length(bytes: &[u8], len_of_len: usize) -> Result<(usize, usize), MerkleError> {
    if len_of_len == 0 {
        return Err(MerkleError::RlpError(
            "RLP long length must not be empty".to_string(),
        ));
    }

    let length_bytes = read_payload(bytes, 1, len_of_len)?;
    if length_bytes.first().copied() == Some(0) {
        return Err(MerkleError::RlpError(
            "RLP length has leading zero".to_string(),
        ));
    }

    let mut len = 0_usize;
    for byte in length_bytes {
        len = len
            .checked_mul(256)
            .and_then(|value| value.checked_add(usize::from(*byte)))
            .ok_or_else(|| MerkleError::RlpError("RLP length overflows usize".to_string()))?;
    }

    Ok((len, 1 + len_of_len))
}

fn parse_rlp_list_payload(payload: &[u8]) -> Result<Vec<Vec<u8>>, MerkleError> {
    let mut items = Vec::new();
    let mut cursor = 0_usize;

    while cursor < payload.len() {
        let (_, consumed) = parse_rlp_item(&payload[cursor..])?;
        items.push(payload[cursor..cursor + consumed].to_vec());
        cursor += consumed;
    }

    Ok(items)
}

fn decode_node_item<D>(item: &RlpItem) -> Result<MptNode<D>, MerkleError>
where
    D: TryFrom<Vec<u8>>,
{
    match item {
        RlpItem::Bytes(bytes) if bytes.is_empty() => Ok(MptNode::Empty),
        RlpItem::Bytes(_) => Err(MerkleError::RlpError(
            "top-level MPT node cannot be a non-empty byte string".to_string(),
        )),
        RlpItem::List(items) if items.len() == 2 => decode_leaf_or_extension(items),
        RlpItem::List(items) if items.len() == 17 => decode_branch(items),
        RlpItem::List(items) => Err(MerkleError::RlpError(format!(
            "invalid MPT list length {}; expected 2 or 17",
            items.len()
        ))),
    }
}

fn decode_leaf_or_extension<D>(items: &[Vec<u8>]) -> Result<MptNode<D>, MerkleError>
where
    D: TryFrom<Vec<u8>>,
{
    let encoded_path = decode_rlp_bytes(&items[0])?;
    let (path, is_leaf) = hp_decode_checked(&encoded_path)?;

    if is_leaf {
        return Ok(MptNode::Leaf {
            key_suffix: path,
            value: decode_rlp_bytes(&items[1])?,
        });
    }

    Ok(MptNode::Extension {
        shared_prefix: path,
        child: decode_node_ref(&items[1]),
    })
}

fn decode_node_ref<D>(item: &[u8]) -> NodeRef<D>
where
    D: TryFrom<Vec<u8>>,
{
    if is_rlp_empty(item) {
        NodeRef::Inline(vec![EMPTY_RLP])
    } else if let Ok(hash) = decode_rlp_hash(item) {
        NodeRef::Hash(hash)
    } else {
        NodeRef::Inline(item.to_vec())
    }
}

fn decode_branch<D>(items: &[Vec<u8>]) -> Result<MptNode<D>, MerkleError>
where
    D: TryFrom<Vec<u8>>,
{
    let mut children = Vec::with_capacity(16);
    for item in &items[..16] {
        children.push(decode_node_ref(item));
    }
    let children = children.try_into().map_err(|_| {
        MerkleError::RlpError("branch node must contain exactly 16 children".to_string())
    })?;
    let value = decode_rlp_bytes(&items[16])?;
    let value = if value.is_empty() { None } else { Some(value) };

    Ok(MptNode::Branch {
        children: Box::new(children),
        value,
    })
}

fn decode_rlp_hash<D>(bytes: &[u8]) -> Result<D, MerkleError>
where
    D: TryFrom<Vec<u8>>,
{
    let value = decode_rlp_bytes(bytes)?;
    if value.len() != 32 {
        return Err(MerkleError::RlpError(
            "branch child hash must be 32 bytes".to_string(),
        ));
    }

    D::try_from(value).map_err(|_| {
        MerkleError::RlpError("failed to convert branch child hash digest".to_string())
    })
}

fn decode_rlp_bytes(bytes: &[u8]) -> Result<Vec<u8>, MerkleError> {
    let (item, consumed) = parse_rlp_item(bytes)?;
    if consumed != bytes.len() {
        return Err(MerkleError::RlpError(
            "trailing bytes after RLP byte string".to_string(),
        ));
    }

    match item {
        RlpItem::Bytes(value) => Ok(value),
        RlpItem::List(_) => Err(MerkleError::RlpError(
            "expected RLP byte string, found list".to_string(),
        )),
    }
}

fn is_rlp_empty(bytes: &[u8]) -> bool {
    bytes == [EMPTY_RLP]
}

#[cfg(test)]
mod tests {
    use merkle_core::error::MerkleError;
    use merkle_core::traits::{MerkleTree, Serializable};
    use merkle_core::types::LeafIndex;
    use merkleforge_hash::{Blake3, Keccak256};

    use super::{
        EMPTY_RLP, MerklePatriciaTrie, MptNode, MptProof, NibblePath, NodeRef, compute_root_hash,
        hash_mpt_bytes, hash_node, hp_decode, hp_encode, rlp_decode, rlp_encode,
    };

    #[test]
    fn new_trie_is_empty() {
        let tree = MerklePatriciaTrie::<Keccak256>::new();

        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
        assert_eq!(tree.node_count(), 0);
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn nibble_path_from_key_splits_each_byte() {
        let path = NibblePath::from_key(b"key");

        assert_eq!(path.len(), 6);
        assert_eq!(path.get(0), 0x6);
        assert_eq!(path.get(1), 0xB);
        assert_eq!(path.get(2), 0x6);
        assert_eq!(path.get(3), 0x5);
        assert_eq!(path.get(4), 0x7);
        assert_eq!(path.get(5), 0x9);
    }

    #[test]
    fn nibble_path_slice_keeps_suffix() {
        let path = NibblePath::from_key(&[0xAB, 0xCD]);
        let suffix = path.slice(2);

        assert_eq!(suffix.len(), 2);
        assert_eq!(suffix.get(0), 0xC);
        assert_eq!(suffix.get(1), 0xD);
    }

    #[test]
    fn nibble_path_common_prefix_len_counts_matching_nibbles() {
        let first = NibblePath::from_key(&[0xAB, 0xCD]);
        let second = NibblePath::from_key(&[0xAB, 0xEF]);

        assert_eq!(first.common_prefix_len(&second), 2);
    }

    #[test]
    fn all_four_node_variants_compile() {
        let leaf = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(b"leaf"),
            value: b"value".to_vec(),
        };
        let extension = MptNode::<[u8; 32]>::Extension {
            shared_prefix: NibblePath::from_key(b"shared"),
            child: NodeRef::Inline(vec![EMPTY_RLP]),
        };
        let branch = MptNode::<[u8; 32]>::Branch {
            children: MptNode::empty_children(),
            value: Some(b"branch value".to_vec()),
        };

        assert!(matches!(MptNode::<[u8; 32]>::Empty, MptNode::Empty));
        assert!(matches!(leaf, MptNode::Leaf { .. }));
        assert!(matches!(extension, MptNode::Extension { .. }));
        assert!(matches!(branch, MptNode::Branch { .. }));
    }

    #[test]
    fn node_height_tracks_longest_path_to_leaf() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();
        tree.insert(&[0xA0, 0xBC], b"value").unwrap();

        assert_eq!(tree.height(), 1);
    }

    #[test]
    fn rlp_encoding_of_empty_node_is_empty_string() {
        let encoded = rlp_encode(&MptNode::<[u8; 32]>::Empty);

        assert_eq!(encoded, vec![0x80]);
        assert_eq!(rlp_decode::<[u8; 32]>(&encoded).unwrap(), MptNode::Empty);
    }

    #[test]
    fn hp_encode_marks_even_and_odd_leaf_paths() {
        let even_path = NibblePath::from_key(&[0xAB]);
        let odd_path = even_path.slice(1);

        let even_encoded = hp_encode(&even_path, true);
        let odd_encoded = hp_encode(&odd_path, true);

        assert_eq!(even_encoded[0] >> 4, 0x2);
        assert_eq!(odd_encoded[0] >> 4, 0x3);
        assert_eq!(hp_decode(&even_encoded), (even_path, true));
        assert_eq!(hp_decode(&odd_encoded), (odd_path, true));
    }

    #[test]
    fn rlp_round_trips_leaf_node() {
        let node = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(b"leaf"),
            value: b"value".to_vec(),
        };

        assert_eq!(rlp_decode(&rlp_encode(&node)).unwrap(), node);
    }

    #[test]
    fn rlp_round_trips_extension_node() {
        let child = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(&[0xCD]),
            value: b"value".to_vec(),
        };
        let node = MptNode::<[u8; 32]>::Extension {
            shared_prefix: NibblePath::from_key(&[0xAB]),
            child: NodeRef::Inline(rlp_encode(&child)),
        };

        assert_eq!(rlp_decode(&rlp_encode(&node)).unwrap(), node);
    }

    #[test]
    fn rlp_round_trips_branch_node() {
        let mut children = MptNode::<[u8; 32]>::empty_children();
        let child = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(&[0xBC]),
            value: b"child".to_vec(),
        };
        children[10] = NodeRef::Inline(rlp_encode(&child));
        let node = MptNode::<[u8; 32]>::Branch {
            children,
            value: Some(b"branch".to_vec()),
        };

        assert_eq!(rlp_decode(&rlp_encode(&node)).unwrap(), node);
    }

    #[test]
    fn malformed_rlp_returns_rlp_error() {
        let result = rlp_decode::<[u8; 32]>(&[0xC1, 0x80]);

        assert!(matches!(result, Err(MerkleError::RlpError(_))));
    }

    #[test]
    fn malformed_hp_prefix_inside_rlp_returns_rlp_error() {
        let malformed_leaf = super::rlp_encode_list(&[
            super::rlp_encode_bytes(&[0x40]),
            super::rlp_encode_bytes(b"value"),
        ]);
        let result = rlp_decode::<[u8; 32]>(&malformed_leaf);

        assert!(matches!(result, Err(MerkleError::RlpError(_))));
    }

    #[test]
    fn empty_trie_root_matches_ethereum_known_hash() {
        let root = compute_root_hash::<Keccak256>(&MptNode::Empty);
        let expected = [
            0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0,
            0xf8, 0x6e, 0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5,
            0xe3, 0x63, 0xb4, 0x21,
        ];

        assert_eq!(root, expected);
    }

    #[test]
    fn hash_node_inlines_small_rlp_nodes() {
        let node = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(&[0x01]),
            value: b"x".to_vec(),
        };
        let rlp = rlp_encode(&node);

        assert!(rlp.len() < 32);
        assert_eq!(hash_node::<Keccak256>(&node), NodeRef::Inline(rlp));
    }

    #[test]
    fn hash_node_hashes_large_rlp_nodes() {
        let node = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(b"large"),
            value: vec![0xAB; 64],
        };

        assert!(rlp_encode(&node).len() >= 32);
        assert!(matches!(hash_node::<Keccak256>(&node), NodeRef::Hash(_)));
    }

    #[test]
    fn root_hash_computation_is_deterministic() {
        let node = MptNode::<[u8; 32]>::Leaf {
            key_suffix: NibblePath::from_key(b"deterministic"),
            value: b"value".to_vec(),
        };

        assert_eq!(
            compute_root_hash::<Blake3>(&node),
            compute_root_hash::<Blake3>(&node)
        );
    }

    #[test]
    fn insert_then_get_returns_value() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();

        assert_eq!(tree.get(b"key"), Some(&b"value"[..]));
        assert!(!tree.is_empty());
        assert!(tree.root().is_some());
    }

    #[test]
    fn get_missing_key_returns_none() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();
        tree.insert(b"key", b"value").unwrap();

        assert_eq!(tree.get(b"missing"), None);
    }

    #[test]
    fn shared_prefix_keys_are_retrievable_independently() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"keyboard", b"instrument").unwrap();

        assert_eq!(tree.get(b"key"), Some(&b"value"[..]));
        assert_eq!(tree.get(b"keyboard"), Some(&b"instrument"[..]));
    }

    #[test]
    fn one_hundred_inserted_pairs_are_retrievable() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        for index in 0_u8..100 {
            let key = [b'k', index];
            let value = [b'v', index];
            tree.insert(&key, &value).unwrap();
        }

        for index in 0_u8..100 {
            let key = [b'k', index];
            let value = [b'v', index];
            assert_eq!(tree.get(&key), Some(&value[..]));
        }
    }

    #[test]
    fn inserting_same_key_updates_value() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"key", b"updated").unwrap();

        assert_eq!(tree.get(b"key"), Some(&b"updated"[..]));
    }

    #[test]
    fn insert_rejects_empty_value_without_mutating() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        assert_eq!(tree.insert(b"key", b""), Err(MerkleError::EmptyLeafData));
        assert!(tree.is_empty());
        assert_eq!(tree.get(b"key"), None);
    }

    #[test]
    fn trait_methods_delegate_to_patricia_trie() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        let index =
            <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::insert(&mut tree, b"value")
                .unwrap();

        assert_eq!(index, LeafIndex(0));
        assert_eq!(
            <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::leaf_count(&tree),
            1
        );
        assert!(<MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::root(&tree).is_some());

        let metadata = <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::metadata(&tree);
        assert_eq!(metadata.leaf_count, 1);
        assert_eq!(metadata.height, tree.height());
        assert_eq!(metadata.node_count, tree.node_count());
        assert_eq!(metadata.hash_algorithm, "Keccak-256");
        assert_eq!(metadata.variant, "MerklePatriciaTrie");
    }

    #[test]
    fn trait_remove_uses_deterministic_index_key() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::insert(&mut tree, b"value")
            .unwrap();
        <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::remove(&mut tree, LeafIndex(0))
            .unwrap();

        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn trait_generate_proof_points_to_key_addressed_mpt_proofs() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::insert(&mut tree, b"value")
            .unwrap();

        assert_eq!(
            <MerklePatriciaTrie<Keccak256> as MerkleTree<Keccak256>>::generate_proof(
                &tree,
                LeafIndex(0),
            ),
            Err(MerkleError::UnsupportedOperation(
                "Patricia trie proofs are key-addressed; use MerklePatriciaTrie::generate_proof",
            ))
        );
    }

    #[test]
    fn root_hash_changes_on_distinct_inserts() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"first", b"value-1").unwrap();
        let first_root = *tree.root().unwrap();
        tree.insert(b"second", b"value-2").unwrap();
        let second_root = *tree.root().unwrap();

        assert_ne!(first_root, second_root);
    }

    #[test]
    fn remove_deletes_existing_key() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.remove(b"key").unwrap();

        assert_eq!(tree.get(b"key"), None);
        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
    }

    #[test]
    fn remove_missing_key_returns_error_without_mutating() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();
        tree.insert(b"key", b"value").unwrap();
        let root_before = *tree.root().unwrap();

        let result = tree.remove(b"missing");

        assert_eq!(
            result,
            Err(MerkleError::UnsupportedOperation(
                "remove missing Patricia key"
            ))
        );
        assert_eq!(tree.get(b"key"), Some(&b"value"[..]));
        assert_eq!(tree.root(), Some(&root_before));
    }

    #[test]
    fn removing_all_keys_returns_to_empty_trie() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"alpha", b"one").unwrap();
        tree.insert(b"beta", b"two").unwrap();
        tree.remove(b"alpha").unwrap();
        tree.remove(b"beta").unwrap();

        assert!(tree.is_empty());
        assert_eq!(tree.root(), None);
        assert_eq!(tree.node_count(), 0);
        assert_eq!(tree.height(), 0);
    }

    #[test]
    fn remove_collapses_to_canonical_remaining_subtree() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();
        let mut singleton = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"keyboard", b"instrument").unwrap();
        tree.remove(b"keyboard").unwrap();
        singleton.insert(b"key", b"value").unwrap();

        assert_eq!(tree.get(b"keyboard"), None);
        assert_eq!(tree.get(b"key"), Some(&b"value"[..]));
        assert_eq!(tree.root(), singleton.root());
        assert_eq!(tree.node_count(), singleton.node_count());
        assert_eq!(tree.height(), singleton.height());
    }

    #[test]
    fn membership_proof_for_inserted_key_verifies() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"keyboard", b"instrument").unwrap();
        let proof = tree.generate_proof(b"keyboard").unwrap();

        assert!(proof.is_membership());
        assert_eq!(proof.value, Some(b"instrument".to_vec()));
        assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(
            tree.root().unwrap(),
            &proof
        ));
    }

    #[test]
    fn proof_root_node_hashes_to_trusted_root() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        let proof = tree.generate_proof(b"key").unwrap();

        assert_eq!(
            hash_mpt_bytes::<Keccak256>(&proof.proof_nodes[0]),
            *tree.root().unwrap()
        );
    }

    #[test]
    fn stale_proof_fails_after_key_is_removed() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"keyboard", b"instrument").unwrap();
        let proof = tree.generate_proof(b"key").unwrap();
        tree.remove(b"key").unwrap();

        assert!(!MerklePatriciaTrie::<Keccak256>::verify_proof(
            tree.root().unwrap(),
            &proof
        ));
    }

    #[test]
    fn non_membership_proof_for_absent_key_verifies() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        let proof = tree.generate_proof(b"missing").unwrap();

        assert!(proof.is_non_membership());
        assert_eq!(proof.value, None);
        assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(
            tree.root().unwrap(),
            &proof
        ));
    }

    #[test]
    fn tampered_mpt_proof_fails_verification() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        let mut proof = tree.generate_proof(b"key").unwrap();
        proof.value = Some(b"tampered".to_vec());

        assert!(!MerklePatriciaTrie::<Keccak256>::verify_proof(
            tree.root().unwrap(),
            &proof
        ));
    }

    #[test]
    fn mpt_proof_round_trips_through_serializable() {
        let mut tree = MerklePatriciaTrie::<Keccak256>::new();

        tree.insert(b"key", b"value").unwrap();
        tree.insert(b"keyboard", b"instrument").unwrap();
        let proof = tree.generate_proof(b"key").unwrap();
        let encoded = proof.to_bytes().unwrap();
        let decoded = MptProof::<[u8; 32]>::from_bytes(&encoded).unwrap();

        assert_eq!(decoded, proof);
        assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(
            tree.root().unwrap(),
            &decoded
        ));
    }

    struct XorShift(u64);

    impl XorShift {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            self.0
        }

        /// Short keys over a tiny alphabet, so keys often share prefixes or
        /// are prefixes of each other (including the empty key).
        fn key(&mut self) -> Vec<u8> {
            let len = (self.next() % 5) as usize;
            (0..len).map(|_| (self.next() % 3) as u8 * 0x11).collect()
        }
    }

    fn rebuilt_from_entries(trie: &MerklePatriciaTrie<Keccak256>) -> MerklePatriciaTrie<Keccak256> {
        let mut entries = trie.entries();
        entries.reverse();
        let mut rebuilt = MerklePatriciaTrie::<Keccak256>::new();
        for (key, value) in entries {
            rebuilt.insert(&key, &value).unwrap();
        }
        rebuilt
    }

    #[test]
    fn incremental_updates_stay_canonical() {
        let mut rng = XorShift(0xBEEF);
        let mut trie = MerklePatriciaTrie::<Keccak256>::new();
        let mut keys = Vec::new();

        for step in 0..500 {
            if rng.next().is_multiple_of(3) && !keys.is_empty() {
                let key: Vec<u8> = keys.swap_remove(rng.next() as usize % keys.len());
                trie.remove(&key).unwrap();
            } else {
                let key = rng.key();
                let value = rng.next().to_be_bytes();
                trie.insert(&key, &value[..1 + (rng.next() % 8) as usize])
                    .unwrap();
                if !keys.contains(&key) {
                    keys.push(key);
                }
            }

            // Insertion order must not matter: a trie built from the same
            // entries in a different order has the same shape and root.
            let rebuilt = rebuilt_from_entries(&trie);
            assert_eq!(trie.root(), rebuilt.root(), "root differs at step {step}");
            assert_eq!(trie.node_count(), rebuilt.node_count(), "step {step}");
            assert_eq!(trie.height(), rebuilt.height(), "step {step}");
            assert_eq!(trie.leaf_count(), keys.len(), "step {step}");
        }
    }

    #[test]
    fn entries_are_returned_in_key_order() {
        let mut trie = MerklePatriciaTrie::<Keccak256>::new();
        for key in [b"dog".as_slice(), b"do", b"", b"doge", b"horse"] {
            trie.insert(key, key.len().to_be_bytes().as_slice())
                .unwrap();
        }

        let keys = trie
            .entries()
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();

        assert_eq!(
            keys,
            vec![
                b"".to_vec(),
                b"do".to_vec(),
                b"dog".to_vec(),
                b"doge".to_vec(),
                b"horse".to_vec()
            ]
        );
    }

    #[test]
    fn trait_insert_after_remove_does_not_overwrite() {
        let mut trie = MerklePatriciaTrie::<Keccak256>::new();
        for data in [b"a", b"b", b"c"] {
            MerkleTree::insert(&mut trie, data).unwrap();
        }
        MerkleTree::remove(&mut trie, LeafIndex(0)).unwrap();

        let index = MerkleTree::insert(&mut trie, b"d").unwrap();

        assert_eq!(index, LeafIndex(3));
        assert_eq!(MerkleTree::leaf_count(&trie), 3);
        let c_key = MerklePatriciaTrie::<Keccak256>::key_for_leaf_index(LeafIndex(2));
        assert_eq!(trie.get(&c_key), Some(&b"c"[..]));
        assert_eq!(
            MerkleTree::remove(&mut trie, LeafIndex(0)),
            Err(MerkleError::IndexOutOfBounds { index: 0, len: 3 })
        );
    }

    #[test]
    fn serialization_round_trips() {
        let mut trie = MerklePatriciaTrie::<Keccak256>::new();
        for (key, value) in [
            (b"do".as_slice(), b"verb".as_slice()),
            (b"dog", b"puppy"),
            (b"", b"root"),
        ] {
            trie.insert(key, value).unwrap();
        }

        let bytes = trie.to_bytes().unwrap();
        let decoded = MerklePatriciaTrie::<Keccak256>::from_bytes(&bytes).unwrap();

        assert_eq!(decoded.root(), trie.root());
        assert_eq!(decoded.entries(), trie.entries());
        assert!(MerklePatriciaTrie::<Blake3>::from_bytes(&bytes).is_err());
    }

    #[test]
    fn deserialization_rejects_duplicate_keys() {
        let repr = (
            "Keccak-256".to_string(),
            vec![
                (b"k".to_vec(), b"1".to_vec()),
                (b"k".to_vec(), b"2".to_vec()),
            ],
        );
        let bytes = repr.to_bytes().unwrap();

        assert!(MerklePatriciaTrie::<Keccak256>::from_bytes(&bytes).is_err());
    }

    #[test]
    fn clone_is_independent_and_debug_summarizes() {
        let mut original = MerklePatriciaTrie::<Keccak256>::new();
        original.insert(b"do", b"verb").unwrap();
        let mut copy = original.clone();
        copy.insert(b"dog", b"puppy").unwrap();

        assert_eq!(original.leaf_count(), 1);
        assert_ne!(original.root(), copy.root());
        assert!(format!("{original:?}").contains("leaf_count: 1"));
    }
}
