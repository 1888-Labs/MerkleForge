# Metadata

Every tree returns a `TreeMetadata` snapshot through the
<Rust inline>`MerkleTree` trait</Rust><TypeScript inline>`MerkleTree` interface</TypeScript>. It is
cheap to build and useful for logging, dashboards, health checks, and benchmarks.

<LangBar />

| Field | Type | Meaning |
| --- | --- | --- |
| <Rust inline>`leaf_count`</Rust><TypeScript inline>`leafCount`</TypeScript> | <Rust inline>`usize`</Rust><TypeScript inline>`number`</TypeScript> | Logical leaves stored |
| `height` | <Rust inline>`usize`</Rust><TypeScript inline>`number`</TypeScript> | `0` for an empty tree, `1` for a single leaf |
| <Rust inline>`node_count`</Rust><TypeScript inline>`nodeCount`</TypeScript> | <Rust inline>`usize`</Rust><TypeScript inline>`number`</TypeScript> | Allocated or materialized nodes (see below) |
| <Rust inline>`hash_algorithm`</Rust><TypeScript inline>`hashAlgorithm`</TypeScript> | <Rust inline>`&'static str`</Rust><TypeScript inline>`string`</TypeScript> | <Rust inline>`H::algorithm_name()`</Rust><TypeScript inline>`hasher.algorithmName`</TypeScript>, e.g. `"SHA-256"` |
| `variant` | <Rust inline>`&'static str`</Rust><TypeScript inline>`string`</TypeScript> | `"BinaryMerkleTree"`, `"SparseMerkleTree"`, or `"MerklePatriciaTrie"` |

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::BinaryMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), MerkleError> {
      let tree = BinaryMerkleTree::<Sha256>::from_leaves(["a", "b", "c", "d", "e"])?;
      let metadata = tree.metadata();

      println!("variant: {}", metadata.variant);
      println!("algorithm: {}", metadata.hash_algorithm);
      println!("leaves: {}", metadata.leaf_count);
      println!("height: {}", metadata.height);
      println!("allocated nodes: {}", metadata.node_count);

      assert_eq!(metadata.variant, "BinaryMerkleTree");
      assert_eq!(metadata.hash_algorithm, "SHA-256");
      assert_eq!(metadata.leaf_count, 5);
      assert_eq!(metadata.height, 4); // 5 leaves pad to 8: 8 -> 4 -> 2 -> 1
      assert_eq!(metadata.node_count, 15); // 8 + 4 + 2 + 1
      Ok(())
  }
  ```

  `metadata()` is a trait method, so bring `MerkleTree` into scope (the prelude does).
</Rust>

<TypeScript>
  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const tree = BinaryMerkleTree.fromLeaves(Sha256, ['a', 'b', 'c', 'd', 'e'].map(utf8))
  const metadata = tree.metadata()

  console.log('variant:', metadata.variant)
  console.log('algorithm:', metadata.hashAlgorithm)
  console.log('leaves:', metadata.leafCount)
  console.log('height:', metadata.height)
  console.log('allocated nodes:', metadata.nodeCount)

  console.assert(metadata.variant === 'BinaryMerkleTree')
  console.assert(metadata.hashAlgorithm === 'SHA-256')
  console.assert(metadata.leafCount === 5)
  console.assert(metadata.height === 4) // 5 leaves pad to 8: 8 -> 4 -> 2 -> 1
  console.assert(metadata.nodeCount === 15) // 8 + 4 + 2 + 1
  ```

  `metadata()` is a method on every tree class (and on `indexed()` views), so no import is needed
  beyond the tree itself.
</TypeScript>

## What `node_count` means per variant

* **Binary:** the length of the flat node array, padding included (`2 × capacity − 1`).
* **Sparse:** materialized storage entries (internal, shortcut, and batch nodes). Thanks to
  shortcut and batch compression this is far smaller than 256 per leaf. Compare with
  <Rust inline>`stored_node_count()`</Rust><TypeScript inline>`storedNodeCount`</TypeScript> and <Rust inline>`unbatched_node_count_estimate()`</Rust><TypeScript inline>`unbatchedNodeCountEstimate()`</TypeScript>.
* **Patricia:** materialized trie nodes, also available as <Rust inline>`node_count()`</Rust><TypeScript inline>`nodeCount`</TypeScript>.

## Serializing metadata

<Rust>
  `TreeMetadata` implements `Serialize`, so you can log it as JSON:

  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Blake3;

  fn main() -> Result<(), Box<dyn std::error::Error>> {
      let mut tree = SparseMerkleTree::<Blake3>::new();
      tree.insert([1_u8; 32], b"value")?;

      let json = serde_json::to_string(&tree.metadata())?;
      assert!(json.contains("\"variant\":\"SparseMerkleTree\""));
      println!("{json}");
      Ok(())
  }
  ```

  Because its string fields are `&'static str`, `TreeMetadata` can't be deserialized from an owned
  buffer. Treat it as output only.
</Rust>

<TypeScript>
  `TreeMetadata` is a plain object, so `JSON.stringify` logs it directly:

  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Blake3 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  const tree = new SparseMerkleTree(Blake3)
  tree.insert(new Uint8Array(32).fill(1), utf8('value'))

  const json = JSON.stringify(tree.metadata())
  console.assert(json.includes('"variant":"SparseMerkleTree"'))
  console.log(json) // keys are camelCase: leafCount, nodeCount, hashAlgorithm
  ```

  The JSON keys are camelCase (`leafCount`, `nodeCount`, `hashAlgorithm`), while Rust's
  `serde_json` writes snake_case (`leaf_count`, …). Rename the keys if a dashboard reads both. Unlike
  Rust, `JSON.parse` reads it back without restriction.
</TypeScript>
