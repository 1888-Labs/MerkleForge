# Choosing a tree

MerkleForge ships three tree variants. They share the <Rust inline>`MerkleTree<H>` trait</Rust><TypeScript inline>`MerkleTree` interface</TypeScript> but are built for
different shapes of data.

<LangBar />

| | `BinaryMerkleTree<H>` | `SparseMerkleTree<H>` | `MerklePatriciaTrie<H>` |
| --- | --- | --- | --- |
| **Data shape** | Ordered list | Map from 32-byte keys to values | Map from byte keys to values |
| **Addressed by** | <Rust inline>`LeafIndex`</Rust><TypeScript inline>`number` index</TypeScript> | 32-byte key | Byte-string key |
| **Proofs** | Inclusion | Membership and non-membership | Membership and non-membership (RLP witness) |
| **Proof size** | `⌈log₂ n⌉` siblings | 256 siblings, empty ones compressed to `None` | One RLP node per trie level on the path |
| **Best hash** | `Sha256` or `Blake3` | `Sha256` or `Blake3` | `Keccak256` for Ethereum roots |
| **Typical use** | Transaction batches, logs, file chunks, SPV | Account state, rollups, allowlists | Ethereum state, receipts, storage proofs |

## Quick decision guide

* **You have a list and prove items by position:** use
  [`BinaryMerkleTree`](/docs/guides/binary-merkle-tree). It has the smallest proofs and the fastest
  construction.
* **You have a key-value map and need to prove a key is absent:** use
  [`SparseMerkleTree`](/docs/guides/sparse-merkle-tree). Keys are 32 bytes, so hash longer keys
  first.
* **You need roots or witnesses that Ethereum tooling understands:** use
  [`MerklePatriciaTrie<Keccak256>`](/docs/guides/merkle-patricia-trie).

## Writing code that works with any variant

The shared <Rust inline>trait</Rust><TypeScript inline>interface</TypeScript> lets you write generic code. Each variant also has a native API (keys
for the sparse tree and trie); the <Rust inline>trait</Rust><TypeScript inline>interface</TypeScript> maps leaf indices onto it.

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::{BinaryMerkleTree, MerklePatriciaTrie, SparseMerkleTree};
  use merkleforge_hash::Sha256;

  /// Insert rows through the shared trait and return a metadata snapshot.
  fn commit<H, T>(tree: &mut T, rows: &[&[u8]]) -> Result<TreeMetadata, MerkleError>
  where
      H: HashFunction,
      T: MerkleTree<H>,
  {
      for row in rows {
          tree.insert(row)?;
      }
      Ok(tree.metadata())
  }

  fn main() -> Result<(), MerkleError> {
      let rows: &[&[u8]] = &[b"alice:100", b"bob:250", b"carol:75"];

      let binary = commit(&mut BinaryMerkleTree::<Sha256>::new(), rows)?;
      let sparse = commit(&mut SparseMerkleTree::<Sha256>::new(), rows)?;
      let trie = commit(&mut MerklePatriciaTrie::<Sha256>::new(), rows)?;

      for meta in [binary, sparse, trie] {
          println!("{:<18} leaves={} height={}", meta.variant, meta.leaf_count, meta.height);
      }
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  The sparse tree and trie implement `MerkleTree` through an `indexed()` view that shares the tree's
  state:

  ```ts
  import { type Bytes, type MerkleTree, type TreeMetadata, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree, MerklePatriciaTrie, SparseMerkleTree } from '@merkleforge/variants'

  /** Insert rows through the shared interface and return a metadata snapshot. */
  function commit(tree: MerkleTree, rows: Bytes[]): TreeMetadata {
    for (const row of rows) tree.insert(row)
    return tree.metadata()
  }

  const rows = ['alice:100', 'bob:250', 'carol:75'].map(utf8)

  const binary = commit(new BinaryMerkleTree(Sha256), rows)
  const sparse = commit(new SparseMerkleTree(Sha256).indexed(), rows)
  const trie = commit(new MerklePatriciaTrie(Sha256).indexed(), rows)

  for (const meta of [binary, sparse, trie]) {
    console.log(`${meta.variant.padEnd(18)} leaves=${meta.leafCount} height=${meta.height}`)
  }
  ```
</TypeScript>

### How the shared interface maps onto each variant

* **Binary:** `insert` appends and returns the next index. `remove` replaces the leaf with
  <Rust inline>`H::empty()`</Rust><TypeScript inline>`hasher.empty()`</TypeScript>; later indices are unchanged.
* **Sparse:** leaf index `i` is stored at a key whose last 8 bytes are `i` in big-endian order.
  `insert` returns the first unoccupied index at or above the leaf count, so it never overwrites a
  leaf after a removal. <Rust inline>`generate_proof`</Rust><TypeScript inline>`generateProof`</TypeScript> returns a `MerkleProof` with 256 path nodes.
* **Patricia:** leaf index `i` is stored under the key `merkleforge:mpt:trait-leaf: ‖ i`
  (8 bytes, big-endian). <Rust inline>`generate_proof` through the trait returns
  `MerkleError::UnsupportedOperation`</Rust><TypeScript inline>`generateProof` through the view throws
  `MerkleError` with code `'UnsupportedOperation'`</TypeScript>, because Patricia proofs are key-addressed. Use
  <Rust inline>`MerklePatriciaTrie::generate_proof(key)`</Rust><TypeScript inline>`trie.generateProof(key)`</TypeScript> instead.

:::tip
Use the <Rust inline>trait</Rust><TypeScript inline>interface</TypeScript> for generic plumbing (ingestion, metadata, benchmarks). Use each variant's native API
when you need key-addressed proofs.
:::
