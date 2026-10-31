# API overview

This page maps the public API of each library. Both expose the same trees, hashes, and proof
formats; the switch shows the Rust crates or the TypeScript packages.

<LangBar />

<Rust>
  For full signatures and doc comments, see docs.rs.

  | Crate | Import as | docs.rs |
  | --- | --- | --- |
  | `merkle-core` | `merkle_core` | [docs.rs/merkle-core](https://docs.rs/merkle-core) |
  | `merkleforge-hash` | `merkleforge_hash` | [docs.rs/merkleforge-hash](https://docs.rs/merkleforge-hash) |
  | `merkle-variants` | `merkle_variants` | [docs.rs/merkle-variants](https://docs.rs/merkle-variants) |

  :::note
  docs.rs documents the latest release (v0.5.1). Items marked *v0.5.0* or *v0.5.1* below were
  added in that release.
  :::
</Rust>

<TypeScript>
  Every package ships TypeScript declarations with TSDoc, so your editor shows full signatures and
  the errors each function throws. All six are ESM only and versioned together.

  | Package | Role | npm |
  | --- | --- | --- |
  | `@merkleforge/core` | Interfaces, proof types, errors, byte helpers, proof codecs | [npm](https://www.npmjs.com/package/@merkleforge/core) |
  | `@merkleforge/hash` | `Sha256`, `Keccak256`, `Blake3` | [npm](https://www.npmjs.com/package/@merkleforge/hash) |
  | `@merkleforge/variants` | The three trees, multiproofs, codecs, Solidity formatting | [npm](https://www.npmjs.com/package/@merkleforge/variants) |
  | `@merkleforge/hash-wasm` | WebAssembly `Keccak256` and `Sha256` | [npm](https://www.npmjs.com/package/@merkleforge/hash-wasm) |
  | `@merkleforge/ethereum` | `eth_getProof` verification | [npm](https://www.npmjs.com/package/@merkleforge/ethereum) |
  | `@merkleforge/openzeppelin` | Drop-in `StandardMerkleTree` and `SimpleMerkleTree` | [npm](https://www.npmjs.com/package/@merkleforge/openzeppelin) |

  For how each Rust item maps to TypeScript, see [TypeScript packages](/docs/reference/typescript).
</TypeScript>

## Core

<Rust>
  `merkle-core`: traits, types, and errors shared by every crate. `no_std` + `alloc` capable
  ([details](/docs/guides/no-std)).

  `merkle_core::prelude` re-exports `MerkleError`, `HashFunction`, `MerkleTree`, `ProofVerifier`, `Serializable`,
  `LeafIndex`, `MerkleProof`, `NodeIndex`, `ProofNode`, `ProofSide`, and `TreeMetadata`.

  ```rust
  use merkle_core::prelude::*;

  fn main() {
      let index = LeafIndex(3);
      assert_eq!(index.value(), 3);
      assert_eq!(NodeIndex::from(index), NodeIndex(3));
      assert_eq!(NodeIndex::root(4), NodeIndex(6)); // [L0 L1 L2 L3 P01 P23 Root]
  }
  ```
</Rust>

<TypeScript>
  `@merkleforge/core`: interfaces, proof types, errors, and helpers shared by every package. It has
  no runtime dependencies.

  ```ts
  import { isMerkleError, isTrivialProof, MerkleError, proofDepth, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const tree = BinaryMerkleTree.fromLeaves(Sha256, ['a', 'b', 'c', 'd'].map(utf8))
  const proof = tree.generateProof(3)

  console.assert(proof.leafIndex === 3 && proof.leafCount === 4)
  console.assert(proofDepth(proof) === 2 && !isTrivialProof(proof))
  console.assert(tree.leafCount === 4 && tree.height === 3)

  try {
    tree.generateProof(9)
  } catch (error) {
    console.assert(isMerkleError(error, 'IndexOutOfBounds') && error instanceof MerkleError)
  }
  ```
</TypeScript>

### Traits and interfaces

<Rust>
  In `merkle_core::traits`:

  | Trait | Methods |
  | --- | --- |
  | [`HashFunction`](https://docs.rs/merkle-core/latest/merkle_core/traits/hash_function/trait.HashFunction.html) | `type Digest`; `hash`, `algorithm_name`, `digest_size`; provided `hash_nodes`, `hash_encoded_node` (*v0.5.0*), `empty` |
  | [`MerkleTree<H>`](https://docs.rs/merkle-core/latest/merkle_core/traits/merkle_tree/trait.MerkleTree.html) | `insert`, `remove`, `root`, `leaf_count`, `is_empty`, `height`, `generate_proof`, `metadata` |
  | [`ProofVerifier<H>`](https://docs.rs/merkle-core/latest/merkle_core/traits/merkle_tree/trait.ProofVerifier.html) | `verify(expected_root, leaf_data, proof) -> bool` |
  | [`Serializable`](https://docs.rs/merkle-core/latest/merkle_core/traits/serializable/trait.Serializable.html) | `to_bytes`, `from_bytes`, `serialized_size`; blanket impl for `Serialize + DeserializeOwned` |
</Rust>

<TypeScript>
  | Interface | Members |
  | --- | --- |
  | `HashFunction` | `algorithmName`, `digestSize`; `hash`, `hashNodes`, `hashEncodedNode`, `empty` |
  | `MerkleTree` | `hasher`; `insert`, `remove`, `root`, `isEmpty`, `generateProof`, `metadata`; `leafCount`, `height` |
  | `ProofVerifier` | `(hasher, expectedRoot, leafData, proof) => boolean`, the type of every `verify` |

  `defineHashFunction(spec)` builds a `HashFunction` from `algorithmName`, `digestSize`, and
  `hash`, deriving `hashNodes`, `hashEncodedNode`, and `empty` unless you supply them.
  `LEAF_PREFIX` (`0x00`) and `NODE_PREFIX` (`0x01`) are the domain-separation bytes.
</TypeScript>

### Types

<Rust>
  In `merkle_core::types`:

  | Type | Description |
  | --- | --- |
  | `LeafIndex(pub usize)` | Index into the leaf layer. `value()`, `to_node_index()` |
  | `NodeIndex(pub usize)` | Index into a flat node array. `value()`, `root(leaf_count)` |
  | `ProofSide` | `Left` or `Right`: where the sibling sits |
  | `ProofNode<D>` | `hash: D`, `side: ProofSide` |
  | `MerkleProof<D>` | `leaf_index`, `leaf_count`, `path: Vec<ProofNode<D>>`; `depth()`, `is_trivial()` |
  | `TreeMetadata` | `leaf_count`, `height`, `node_count`, `hash_algorithm`, `variant` |
</Rust>

<TypeScript>
  | Type | Description |
  | --- | --- |
  | `Bytes` | `Uint8Array`: leaf data, digests, and encoded proofs |
  | `ProofSide` | `'left'` or `'right'`: where the sibling sits |
  | `ProofNode` | `hash: Bytes`, `side: ProofSide` |
  | `MerkleProof` | `leafIndex`, `leafCount`, `path: ProofNode[]`; with `proofDepth(proof)` and `isTrivialProof(proof)` |
  | `TreeMetadata` | `leafCount`, `height`, `nodeCount`, `hashAlgorithm`, `variant` |
</TypeScript>

### Errors

<Rust>
  `MerkleError`, a `#[non_exhaustive]` enum. See [Errors](/docs/guides/errors).
</Rust>

<TypeScript>
  `MerkleError` extends `Error`. Its `code` (and `detail`) is one of `EmptyTree`,
  `IndexOutOfBounds`, `InvalidProof`, `InvalidProofStructure`, `EmptyLeafData`,
  `SerializationError`, `DeserializationError`, `HashError`, `UnsupportedOperation`, or `RlpError`.
  `isMerkleError(error, code?)` narrows an unknown error. See [Errors](/docs/guides/errors).
</TypeScript>

### Helpers and codecs

<Rust>
  Proofs serialize through the `Serializable` trait and serde. See [Serialization](/docs/guides/serialization).
</Rust>

<TypeScript>
  | Function | Description |
  | --- | --- |
  | `utf8`, `toHex`, `fromHex`, `bytesEqual`, `concatBytes` | Byte helpers; `toHex` returns a `0x`-prefixed string |
  | `encodeMerkleProof`, `decodeMerkleProof` | Same bytes as Rust's bincode `Serializable` |
  | `merkleProofToSerdeJSON`, `merkleProofFromSerdeJSON` | Same JSON as Rust's `serde_json` |
  | `BincodeWriter`, `BincodeReader`, `byteArrayFromJSON` | Building blocks for the codecs |
</TypeScript>

## Hash functions

<Rust>
  `merkleforge-hash`:

  | Item | Description |
  | --- | --- |
  | [`Sha256`](https://docs.rs/merkleforge-hash/latest/merkleforge_hash/struct.Sha256.html) | SHA-256 with `0x00`/`0x01` domain prefixes |
  | [`Keccak256`](https://docs.rs/merkleforge-hash/latest/merkleforge_hash/struct.Keccak256.html) | Keccak-256 with `0x00`/`0x01` prefixes; plain Keccak for Patricia nodes |
  | [`Blake3`](https://docs.rs/merkleforge-hash/latest/merkleforge_hash/struct.Blake3.html) | BLAKE3 `derive_key` with leaf and internal-node contexts |
  | `HashFunction` | Re-export of `merkle_core::traits::HashFunction` |

  All three are zero-sized, `Copy`, and produce `[u8; 32]`. See [Hash functions](/docs/guides/hash-functions).
</Rust>

<TypeScript>
  `@merkleforge/hash` (built on [`@noble/hashes`](https://github.com/paulmillr/noble-hashes)):

  | Item | Description |
  | --- | --- |
  | `Sha256` | SHA-256 with `0x00`/`0x01` domain prefixes |
  | `Keccak256` | Keccak-256 with `0x00`/`0x01` prefixes; plain Keccak for Patricia nodes |
  | `Blake3` | BLAKE3 `derive_key` with leaf and internal-node contexts |

  Each is a frozen `HashFunction` object producing 32-byte digests, identical to the Rust adapters.
  Pass it to a tree's constructor and to the static verifiers.

  `@merkleforge/hash-wasm` adds `createKeccak256Wasm()` and `createSha256Wasm()`, async factories
  for WebAssembly versions with the same output and `algorithmName` (about 6x faster Keccak-256 in
  Node). See [Hash functions](/docs/guides/hash-functions).
</TypeScript>

## Trees

<Rust>
  `merkle-variants`. Top-level re-exports: `BinaryMerkleTree`, `SparseMerkleTree`,
  `MerklePatriciaTrie`, `MptProof`.
</Rust>

<TypeScript>
  `@merkleforge/variants`. Every tree takes its `HashFunction` in the constructor; verifiers are
  static and take the hash function first. Trees serialize with `toJSON()` and load with
  `fromJSON(hasher, json)`, which rejects JSON written with a different hash.
</TypeScript>

### Binary Merkle tree

<Rust>
  [`BinaryMerkleTree<H>`](https://docs.rs/merkle-variants/latest/merkle_variants/binary/struct.BinaryMerkleTree.html):

  | Method | Notes |
  | --- | --- |
  | `new()`, `with_capacity(n)`, `Default` | Empty tree |
  | `from_leaves(iter)` | One-pass `O(n)` build (*v0.5.0*) |
  | `insert(&[u8]) -> Result<LeafIndex>` | Append a leaf |
  | `remove(LeafIndex)` | Replace with `H::empty()` |
  | `root()`, `leaf_count()`, `is_empty()`, `height()` | Queries |
  | `generate_proof(LeafIndex) -> Result<MerkleProof>` | Inclusion proof |
  | `verify(root, leaf, &proof) -> bool` | Stateless, associated function |
  | `generate_multi_proof(&[LeafIndex]) -> Result<BinaryMultiProof>` | One proof for several leaves (*v0.5.1*) |
  | `verify_multi_proof(root, &leaves, &proof) -> bool` | Stateless; leaves in `proof.leaf_indices` order (*v0.5.1*) |

  Implements `MerkleTree<H>`, `ProofVerifier<H>`; `Clone`, `Debug`, `Serialize`, `Deserialize`
  (*v0.5.0*). `BinaryMultiProof<D>` (`leaf_indices`, `leaf_count`, `hashes`) serializes in the
  same format as the TypeScript package (*v0.5.1*).
</Rust>

<TypeScript>
  `BinaryMerkleTree`:

  | Member | Notes |
  | --- | --- |
  | `new BinaryMerkleTree(hasher)`, `BinaryMerkleTree.withCapacity(hasher, n)` | Empty tree |
  | `BinaryMerkleTree.fromLeaves(hasher, leaves)` | One-pass `O(n)` build |
  | `insert(data): number` | Append a leaf; returns its index |
  | `remove(index)` | Replace with `hasher.empty()` |
  | `root()`, `isEmpty()`, `leafCount`, `height` | Queries (`leafCount` and `height` are getters) |
  | `generateProof(index): MerkleProof` | Inclusion proof |
  | `BinaryMerkleTree.verify(hasher, root, leaf, proof)` | Stateless |
  | `generateMultiProof(indices): BinaryMultiProof` | One proof for several leaves |
  | `BinaryMerkleTree.verifyMultiProof(hasher, root, leaves, proof)` | Stateless; leaves in `proof.leafIndices` order |
  | `metadata()`, `toJSON()`, `BinaryMerkleTree.fromJSON(hasher, json)` | Metadata and JSON |

  Implements `MerkleTree`. `BinaryMultiProof` (`leafIndices`, `leafCount`, `hashes`) uses the same
  bincode and JSON format as Rust; `verifyBinaryMultiProof` is the standalone verifier.
</TypeScript>

Guide: [Binary Merkle tree](/docs/guides/binary-merkle-tree).

### Sparse Merkle tree

<Rust>
  [`SparseMerkleTree<H>`](https://docs.rs/merkle-variants/latest/merkle_variants/sparse/struct.SparseMerkleTree.html):

  | Method | Notes |
  | --- | --- |
  | `new()`, `Default` | Empty 256-depth tree |
  | `insert(key, &[u8])`, `batch_insert(&[(key, Vec<u8>)])` | Insert or overwrite |
  | `remove(key)`, `batch_remove(&[key])` | Remove existing keys |
  | `get(key) -> Option<&Digest>` | Stored leaf hash |
  | `generate_membership_proof(key) -> Result<SparseMerkleProof>` | Membership or non-membership |
  | `verify_membership(root, key, leaf, &proof)` | Checks key, value, path (*v0.5.0*) |
  | `verify_non_membership(root, key, &proof)` | Absence proof |
  | `verify(root, leaf, &proof)` | **Deprecated**: doesn't check the key |
  | `root()`, `empty_root()`, `leaf_count()`, `is_empty()`, `height()`, `depth()` | Queries |
  | `empty_hash_at(level)`, `empty_hashes()` | Empty-subtree cache |
  | `stored_node_count()`, `last_node_access_count()`, `unbatched_node_count_estimate()` | Storage introspection |

  Also: `SparseMerkleProof<D>` (`key`, `siblings`, `leaf`; `is_membership()`,
  `is_non_membership()`), `NodeEntry<D>` (with `commitment()`, *v0.5.0*), `SubtreeBatch<D>`,
  `bit_at`, and the constants `SPARSE_TREE_DEPTH`, `EMPTY_HASH_LEVELS`, `BATCH_DEPTH`.
</Rust>

<TypeScript>
  `SparseMerkleTree` (keys are 32-byte `Uint8Array`s):

  | Member | Notes |
  | --- | --- |
  | `new SparseMerkleTree(hasher)` | Empty 256-depth tree |
  | `insert(key, data)`, `batchInsert([[key, data], …])` | Insert or overwrite |
  | `remove(key)`, `batchRemove(keys)` | Remove existing keys |
  | `get(key)` | Stored leaf hash, or `undefined` |
  | `generateMembershipProof(key): SparseMerkleProof` | Membership or non-membership |
  | `SparseMerkleTree.verifyMembership(hasher, root, key, leaf, proof)` | Checks key, value, and path |
  | `SparseMerkleTree.verifyNonMembership(hasher, root, key, proof)` | Absence proof |
  | `root()`, `emptyRoot()`, `isEmpty()`, `leafCount`, `height`, `depth` | Queries |
  | `emptyHashAt(level)`, `emptyHashes()` | Empty-subtree cache |
  | `storedNodeCount`, `lastNodeAccessCount`, `unbatchedNodeCountEstimate()` | Storage introspection |
  | `indexed()` | A `MerkleTree` view addressed by leaf index |
  | `metadata()`, `toJSON()`, `SparseMerkleTree.fromJSON(hasher, json)` | Metadata and JSON |

  Also: `SparseMerkleProof` (`key`, `siblings` with `null` for empty subtrees, `leaf`),
  `isSparseMembershipProof`, `bitAt`, and the constants `SPARSE_TREE_DEPTH`, `EMPTY_HASH_LEVELS`,
  `BATCH_DEPTH`, `SPARSE_KEY_BYTES`. There is no key-blind `verify`.
</TypeScript>

Guide: [Sparse Merkle tree](/docs/guides/sparse-merkle-tree).

### Merkle Patricia trie

<Rust>
  [`MerklePatriciaTrie<H>`](https://docs.rs/merkle-variants/latest/merkle_variants/patricia/struct.MerklePatriciaTrie.html):

  | Method | Notes |
  | --- | --- |
  | `new()`, `Default` | Empty trie |
  | `insert(key, value)`, `remove(key)`, `get(key) -> Option<&[u8]>` | Key-value operations |
  | `generate_proof(key) -> Result<MptProof>` | RLP witness, membership or non-membership |
  | `verify_proof(root, &proof) -> bool` | Stateless |
  | `root()`, `is_empty()`, `leaf_count()`, `node_count()`, `height()` | Queries |

  Also: `MptProof<D>` (`key`, `value`, `proof_nodes`; `new`, `is_membership`, `is_non_membership`),
  `MptNode<D>`, `NodeRef<D>`, `NibblePath`, `hp_encode`, `hp_decode`, `rlp_encode`, `rlp_decode`,
  `hash_node`, `compute_root_hash`.
</Rust>

<TypeScript>
  `MerklePatriciaTrie`:

  | Member | Notes |
  | --- | --- |
  | `new MerklePatriciaTrie(hasher)` | Empty trie |
  | `insert(key, value)`, `remove(key)`, `get(key)` | Key-value operations; `get` returns `undefined` when absent |
  | `generateProof(key): MptProof` | RLP witness, membership or non-membership |
  | `MerklePatriciaTrie.verifyProof(hasher, root, proof)` | Stateless |
  | `MerklePatriciaTrie.proveValue(hasher, root, key, proofNodes)` | Returns the proven value or `null`; accepts `eth_getProof`-style witnesses |
  | `root()`, `isEmpty()`, `leafCount`, `nodeCount`, `height`, `entries()` | Queries |
  | `indexed()` | A `MerkleTree` view addressed by leaf index |
  | `metadata()`, `toJSON()`, `MerklePatriciaTrie.fromJSON(hasher, json)` | Metadata and JSON |

  Also: `MptProof` (`key`, `value` or `null`, `proofNodes`), `MptNode` and `NodeRef` (`{ kind }`
  unions), `bytesToNibbles`, `hpEncode`, `hpDecode`, `rlpEncode`, `rlpDecode`, `encodeRlp`,
  `decodeRlp`, `hashNode`, and `computeRootHash`.
</TypeScript>

Guide: [Merkle Patricia trie](/docs/guides/merkle-patricia-trie).

## Proof formats

<Rust>
  Every proof type implements `Serialize`, `Deserialize`, and `Serializable` (bincode). See
  [Serialization](/docs/guides/serialization).
</Rust>

<TypeScript>
  Each proof type has a bincode codec and a `serde_json`-compatible JSON codec, with the same bytes
  and JSON as the Rust crates:

  | Proof | Bincode | JSON |
  | --- | --- | --- |
  | `MerkleProof` | `encodeMerkleProof` / `decodeMerkleProof` | `merkleProofToSerdeJSON` / `merkleProofFromSerdeJSON` |
  | `BinaryMultiProof` | `encodeBinaryMultiProof` / `decodeBinaryMultiProof` | `multiProofToSerdeJSON` / `multiProofFromSerdeJSON` |
  | `SparseMerkleProof` | `encodeSparseProof` / `decodeSparseProof` | `sparseProofToSerdeJSON` / `sparseProofFromSerdeJSON` |
  | `MptProof` | `encodeMptProof` / `decodeMptProof` | `mptProofToSerdeJSON` / `mptProofFromSerdeJSON` |

  For the Solidity verifiers, `toSolidityBinaryProof`, `toSolidityBinaryMultiProof`, and
  `toSoliditySparseProof` reshape proofs into hex strings (and a `bigint` bitmap for sparse
  proofs). See [Serialization](/docs/guides/serialization) and
  [Proofs](/docs/guides/proofs#verifying-on-chain).
</TypeScript>

## TypeScript-only packages

<Rust>
  These packages have no Rust equivalent. Switch to TypeScript to see their API, or read
  [TypeScript packages](/docs/reference/typescript).
</Rust>

<TypeScript>
  **`@merkleforge/hash-wasm`**

  | Function | Returns |
  | --- | --- |
  | `createKeccak256Wasm()` | `Promise<HashFunction>`: WebAssembly Keccak-256, same output as `Keccak256` |
  | `createSha256Wasm()` | `Promise<HashFunction>`: WebAssembly SHA-256, same output as `Sha256` |

  **`@merkleforge/ethereum`**

  | Item | Description |
  | --- | --- |
  | `verifyEthGetProof(stateRoot, response, options?)` | Verifies a full `eth_getProof` response; returns the account (or `null`) and storage values. Throws `InvalidProof` on any mismatch |
  | `verifyAccountProof({ stateRoot, address, proof }, options?)` | Account proof only; returns `EthAccount` or `null` |
  | `verifyStorageProof({ storageRoot, slot, proof }, options?)` | One storage slot; returns its value as a `bigint` (`0n` when empty) |
  | `mappingSlot(key, slot)` | Storage slot of `mapping[key]` for a mapping declared at `slot` |
  | `keccak256`, `toBytes32`, `bytesToBigInt` | Helpers |
  | `EMPTY_TRIE_ROOT`, `EMPTY_CODE_HASH` | Constants |

  `options.hasher` swaps the trie hash, for example `await createKeccak256Wasm()`. Types:
  `EthGetProofResponse`, `EthAccount`, `VerifiedEthProof`, `BytesLike`.

  **`@merkleforge/openzeppelin`**

  | Item | Description |
  | --- | --- |
  | `StandardMerkleTree.of(values, leafEncoding)` | ABI-encoded leaves, the same tree as `@openzeppelin/merkle-tree` |
  | `SimpleMerkleTree.of(values)` | `bytes32` leaves |
  | `getProof`, `getMultiProof`, `verify`, `verifyMultiProof`, `root`, `entries()`, `dump()` | The same instance API |
  | `StandardMerkleTree.load(dump)`, `SimpleMerkleTree.load(dump)` | Load an existing `dump()` unchanged |
  | `StandardMerkleTree.verify(…)`, `SimpleMerkleTree.verify(…)` | Static verifiers |
  | `abiEncode(types, values)`, `standardLeafHash`, `standardNodeHash`, `formatLeaf` | Helpers |
</TypeScript>

## Safety

<Rust>
  Every published crate uses `#![forbid(unsafe_code)]` and `#![deny(missing_docs)]`.
</Rust>

<TypeScript>
  The packages compile with `strict`, `noUncheckedIndexedAccess`, and `exactOptionalPropertyTypes`,
  document every public API with TSDoc (including what it throws), and return copies of internal
  `Uint8Array` state rather than the buffers themselves. The tree verifiers (`verify`,
  `verifyMultiProof`, `verifyMembership`, `verifyNonMembership`, `verifyProof`) return `false` for
  malformed proofs instead of throwing; `proveValue` and the `@merkleforge/ethereum` functions throw
  `InvalidProof`, because they return the proven value. Releases are published with provenance
  through npm trusted publishing.
</TypeScript>
