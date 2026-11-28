# Serialization

Proofs have two wire formats shared by both languages: a compact binary encoding
([`bincode`](https://crates.io/crates/bincode) 2, standard config) and a JSON shape (what
`serde_json` produces). A proof encoded in Rust decodes in TypeScript, and the other way round.

<LangBar />

<Rust>
  Proofs and trees implement `serde`'s `Serialize` and `Deserialize`. On top of that, `merkle-core`
  provides the `Serializable` trait with compact binary helpers backed by bincode.
</Rust>

<TypeScript>
  Proofs are plain objects and trees are classes, so there is no `serde` or trait to derive.
  Instead, each proof type has a pair of explicit codecs for each format, all of which produce the
  same bytes or JSON as the Rust crates:

  | Proof | Binary (bincode) | Rust-compatible JSON |
  | --- | --- | --- |
  | `MerkleProof` (`@merkleforge/core`) | `encodeMerkleProof` / `decodeMerkleProof` | `merkleProofToSerdeJSON` / `merkleProofFromSerdeJSON` |
  | `SparseMerkleProof` | `encodeSparseProof` / `decodeSparseProof` | `sparseProofToSerdeJSON` / `sparseProofFromSerdeJSON` |
  | `MptProof` | `encodeMptProof` / `decodeMptProof` | `mptProofToSerdeJSON` / `mptProofFromSerdeJSON` |
  | `BinaryMultiProof` | `encodeBinaryMultiProof` / `decodeBinaryMultiProof` | `multiProofToSerdeJSON` / `multiProofFromSerdeJSON` |

  The sparse, Patricia, and multiproof codecs are exported from `@merkleforge/variants`. Decoders
  take an optional `digestSize` (default 32) where the proof holds fixed-size digests.
</TypeScript>

## The `Serializable` trait

<Rust>
  ```rust
  use merkle_core::error::MerkleError;

  pub trait Serializable: Sized {
      fn to_bytes(&self) -> Result<Vec<u8>, MerkleError>;
      fn from_bytes(bytes: &[u8]) -> Result<Self, MerkleError>;
      fn serialized_size(&self) -> Result<usize, MerkleError> {
          Ok(self.to_bytes()?.len())
      }
  }
  ```

  A blanket implementation covers **every** type that is `Serialize + DeserializeOwned`, so you
  never implement it by hand. Errors map to `MerkleError::SerializationError` and
  `MerkleError::DeserializationError`.
</Rust>

<TypeScript>
  The trait is Rust only. Its TypeScript counterpart is the `encode*` / `decode*` codecs above:
  `encodeMerkleProof(proof)` returns the same bytes as Rust's `proof.to_bytes()`, and
  `decodeMerkleProof(bytes)` reads them. For the size, take `.length` of the encoded bytes.
  Malformed input throws `MerkleError` with code `DeserializationError`. Unlike Rust's `from_bytes`,
  the decoders also reject trailing bytes after the proof.
</TypeScript>

## Proofs

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::BinaryMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), MerkleError> {
      let tree = BinaryMerkleTree::<Sha256>::from_leaves(["alice", "bob", "carol", "dave"])?;
      let proof = tree.generate_proof(LeafIndex(2))?;

      let bytes = proof.to_bytes()?;
      let recovered = MerkleProof::<[u8; 32]>::from_bytes(&bytes)?;
      assert_eq!(proof, recovered);
      println!("{} bytes", proof.serialized_size()?);

      // Garbage input is an error, not a panic.
      assert!(MerkleProof::<[u8; 32]>::from_bytes(b"not a proof").is_err());
      Ok(())
  }
  ```

  `SparseMerkleProof<D>` and `MptProof<D>` work the same way.
</Rust>

<TypeScript>
  ```ts
  import { decodeMerkleProof, encodeMerkleProof, isMerkleError, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const tree = BinaryMerkleTree.fromLeaves(Sha256, ['alice', 'bob', 'carol', 'dave'].map(utf8))
  const proof = tree.generateProof(2)

  // Same bytes as the Rust proof.to_bytes().
  const bytes = encodeMerkleProof(proof)
  const recovered = decodeMerkleProof(bytes)
  console.assert(BinaryMerkleTree.verify(Sha256, tree.root()!, utf8('carol'), recovered))
  console.log(`${bytes.length} bytes`)

  // Garbage input throws a MerkleError, not some other exception.
  try {
    decodeMerkleProof(utf8('not a proof'))
    console.assert(false)
  } catch (error) {
    console.assert(isMerkleError(error, 'DeserializationError'))
  }
  ```

  `encodeSparseProof` / `decodeSparseProof` and `encodeMptProof` / `decodeMptProof` work the same way.
</TypeScript>

## Any serde format

<Rust>
  Because the types are plain `serde` types, you can use JSON, CBOR, or anything else. With
  [`serde_json`](https://crates.io/crates/serde_json):

  ```rust
  use merkle_variants::{SparseMerkleTree, sparse::SparseMerkleProof};
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      let mut state = SparseMerkleTree::<Sha256>::new();
      state.insert([7_u8; 32], b"balance:100")?;
      let proof = state.generate_membership_proof([7_u8; 32])?;

      let json = serde_json::to_string(&proof)?;
      let decoded: SparseMerkleProof<[u8; 32]> = serde_json::from_str(&json)?;
      assert_eq!(proof, decoded);
      Ok(())
  }
  ```

  :::tip
  JSON encodes `[u8; 32]` as an array of numbers. A TypeScript client reads that shape directly with
  `@merkleforge/variants` (`sparseProofFromSerdeJSON` and friends). For other languages, define a
  small wire format with hex strings and convert at the boundary.
  :::
</Rust>

<TypeScript>
  `JSON.stringify` on a proof object does not give you a portable format (`Uint8Array` fields
  become `{"0": …}` objects). Convert with the `*ToSerdeJSON` helpers first: they produce exactly
  the objects `serde_json` writes for the Rust types (snake_case fields, bytes as arrays of numbers,
  `"Left"` / `"Right"` sides), so a Rust service can read them with `serde_json::from_str`.

  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import {
    SparseMerkleTree,
    sparseProofFromSerdeJSON,
    sparseProofToSerdeJSON,
  } from '@merkleforge/variants'

  const state = new SparseMerkleTree(Sha256)
  const key = new Uint8Array(32).fill(7)
  state.insert(key, utf8('balance:100'))
  const proof = state.generateMembershipProof(key)

  // The same JSON as serde_json::to_string(&proof) in Rust.
  const json = JSON.stringify(sparseProofToSerdeJSON(proof))
  const decoded = sparseProofFromSerdeJSON(JSON.parse(json))
  console.assert(
    SparseMerkleTree.verifyMembership(Sha256, state.root()!, key, utf8('balance:100'), decoded),
  )
  ```

  For CBOR, MessagePack, or another format, encode the same `*ToSerdeJSON` object with the library of
  your choice, or send the bincode bytes.
</TypeScript>

## Trees

`BinaryMerkleTree`, `SparseMerkleTree`, and `MerklePatriciaTrie` serialize compactly. The encoding
stores only:

| Tree | Stored |
| --- | --- |
| `BinaryMerkleTree` | Hash algorithm name + leaf hashes |
| `SparseMerkleTree` | Hash algorithm name + `(key, leaf hash)` pairs in key order |
| `MerklePatriciaTrie` | Hash algorithm name + `(key, value)` entries in key order |

<TypeScript>
  In TypeScript, each tree has `toJSON()` (so `JSON.stringify(tree)` works) and a static
  `fromJSON(hasher, json)`. The JSON is `{ hashAlgorithm, leaves }` (binary and sparse) or
  `{ hashAlgorithm, entries }` (Patricia), with every byte string as `0x` hex. This tree format is
  TypeScript's own: it is not the Rust serde or bincode tree encoding, so move proofs (not whole
  trees) between the two languages, or rebuild the tree from its leaves.
</TypeScript>

Internal nodes are **recomputed** on load, so a decoded tree is always internally consistent.
Deserialization rejects data written with a different hash algorithm.

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::BinaryMerkleTree;
  use merkleforge_hash::{Blake3, Sha256};

  fn main() -> Result<(), MerkleError> {
      let mut tree = BinaryMerkleTree::<Sha256>::from_leaves(["alice", "bob", "carol"])?;
      tree.remove(LeafIndex(1))?;

      let bytes = tree.to_bytes()?;
      let restored = BinaryMerkleTree::<Sha256>::from_bytes(&bytes)?;
      assert_eq!(restored.root(), tree.root());
      assert_eq!(restored.leaf_count(), tree.leaf_count());

      // Bytes from a SHA-256 tree can't be loaded as a BLAKE3 tree.
      assert!(BinaryMerkleTree::<Blake3>::from_bytes(&bytes).is_err());
      Ok(())
  }
  ```

  :::note
  Tree `Serialize` / `Deserialize` (and `Clone` / `Debug`) were added in v0.5.0. Proof
  serialization is available in earlier releases too.
  :::
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, isMerkleError, utf8 } from '@merkleforge/core'
  import { Blake3, Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const tree = BinaryMerkleTree.fromLeaves(Sha256, ['alice', 'bob', 'carol'].map(utf8))
  tree.remove(1)

  // toJSON() makes JSON.stringify work directly; leaf hashes are hex strings.
  const json = JSON.stringify(tree)
  const restored = BinaryMerkleTree.fromJSON(Sha256, JSON.parse(json))
  console.assert(bytesEqual(restored.root()!, tree.root()!))
  console.assert(restored.leafCount === tree.leafCount)

  // JSON from a SHA-256 tree can't be loaded as a BLAKE3 tree.
  try {
    BinaryMerkleTree.fromJSON(Blake3, JSON.parse(json))
    console.assert(false)
  } catch (error) {
    console.assert(isMerkleError(error, 'DeserializationError'))
  }
  ```
</TypeScript>

## Metadata

<Rust>
  `TreeMetadata` implements `Serialize` and `Deserialize`, but its string fields are
  `&'static str`, so it can't be deserialized from an owned buffer and is **not** `Serializable`.
  Serialize it for logs and dashboards, for example with `serde_json::to_string(&tree.metadata())`.
</Rust>

<TypeScript>
  `TreeMetadata` is a plain object, so `JSON.stringify(tree.metadata())` works and `JSON.parse`
  reads it back. Its keys are camelCase (`leafCount`, `nodeCount`, …), not the snake_case keys Rust
  writes.
</TypeScript>

See [Metadata](/docs/guides/metadata).
