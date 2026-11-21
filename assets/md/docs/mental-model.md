# Mental model

Think of a Merkle root as a **fingerprint for many values**. Change any value and the fingerprint
changes. A proof is the short path that lets another machine check one value against that
fingerprint.

<LangBar />

## The four pieces

| Piece | What it is | In MerkleForge |
| --- | --- | --- |
| **Leaf** | The hash of one value you inserted. | <Rust inline>`H::hash(data)`</Rust><TypeScript inline>`hasher.hash(data)`</TypeScript>, computed for you by `insert`. |
| **Root** | One digest that commits to every leaf. | `tree.root()` returns <Rust inline>`Option<&H::Digest>`</Rust><TypeScript inline>`Uint8Array` or `undefined`</TypeScript>. |
| **Proof** | The sibling hashes on the path from one leaf to the root. | <Rust inline>`MerkleProof<D>` (binary), `SparseMerkleProof<D>`, `MptProof<D>`</Rust><TypeScript inline>`MerkleProof` (binary), `SparseMerkleProof`, `MptProof`</TypeScript>. |
| **Verifier** | Code that rebuilds the root from a leaf and a proof. | <Rust inline>`ProofVerifier::verify` and the per-tree `verify*` functions</Rust><TypeScript inline>The static `verify*` functions on each tree class</TypeScript>. |

```text
                 root = H(0x01 ‖ n01 ‖ n23)
                /                          \
     n01 = H(0x01 ‖ l0 ‖ l1)      n23 = H(0x01 ‖ l2 ‖ l3)
        /            \               /            \
 l0 = H(0x00‖a)  l1 = H(0x00‖b)  l2 = H(0x00‖c)  l3 = H(0x00‖d)
```

To prove `a` is leaf 0, the prover sends `l1` and `n23`. The verifier computes `l0` from `a`, then
`n01`, then the root, and compares it with the root it already trusts. For `n` leaves that is
`⌈log₂ n⌉` hashes, no matter how big the dataset is.

## Who holds what

* The **prover** (a server, indexer, or full node) holds the full tree. It publishes the root and
  answers proof requests.
* The **verifier** (a light client, browser, contract, or embedded device) holds only a **trusted
  root**, for example from a block header or a signed checkpoint.

A proof is only as good as the root you check it against. Get the root from a source you trust.

## Domain separation: `0x00` and `0x01`

MerkleForge hashes leaves and internal nodes in different **domains**:

| Hash | Pre-image |
| --- | --- |
| Leaf | `H(0x00 ‖ data)` |
| Internal node | `H(0x01 ‖ left ‖ right)` |

Without the prefixes, an internal node's pre-image (`left ‖ right`, 64 bytes) is also a valid leaf
value. An attacker could then present an internal node as if it were a leaf and forge a "proof"
for data that was never inserted. This is a **second-preimage attack**. With distinct prefixes, a
leaf hash can never equal an internal node hash for a different structure.

The `Sha256` and `Keccak256` adapters use the byte prefixes above. `Blake3` gets the same
separation from BLAKE3's `derive_key` mode with two context strings. See
[Hash functions](/docs/guides/hash-functions) for the exact constructions.

## Verifying by hand

Nothing in a proof is magic. This program rebuilds the root manually and checks it against the
library's own verifier:

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::BinaryMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), MerkleError> {
      let tree = BinaryMerkleTree::<Sha256>::from_leaves(["a", "b", "c", "d"])?;
      let root = *tree.root().expect("non-empty");
      let proof = tree.generate_proof(LeafIndex(0))?;

      // 1. current = H(0x00 ‖ leaf)
      let mut current = Sha256::hash(b"a");
      // 2. Walk the siblings from the leaf level up to the root.
      for node in &proof.path {
          current = match node.side {
              ProofSide::Left => Sha256::hash_nodes(&node.hash, &current),
              ProofSide::Right => Sha256::hash_nodes(&current, &node.hash),
          };
      }
      // 3. Compare with the trusted root.
      assert_eq!(current, root);
      assert!(BinaryMerkleTree::<Sha256>::verify(&root, b"a", &proof));
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const tree = BinaryMerkleTree.fromLeaves(Sha256, ['a', 'b', 'c', 'd'].map(utf8))
  const root = tree.root()!
  const proof = tree.generateProof(0)

  // 1. current = H(0x00 ‖ leaf)
  let current = Sha256.hash(utf8('a'))
  // 2. Walk the siblings from the leaf level up to the root.
  for (const node of proof.path) {
    current =
      node.side === 'left' ? Sha256.hashNodes(node.hash, current) : Sha256.hashNodes(current, node.hash)
  }
  // 3. Compare with the trusted root.
  console.assert(bytesEqual(current, root))
  console.assert(BinaryMerkleTree.verify(Sha256, root, utf8('a'), proof))
  ```
</TypeScript>

<Rust inline>`ProofSide::Left`</Rust><TypeScript inline>`side: 'left'`</TypeScript> means the sibling sits on the left, so it goes first in the concatenation.

## Padding and empty slots

* `BinaryMerkleTree` pads the leaf layer to the next power of two with <Rust inline>`H::empty()`</Rust><TypeScript inline>`hasher.empty()`</TypeScript>,
  the hash of an empty leaf. Five leaves become eight slots.
* `SparseMerkleTree` has 2²⁵⁶ slots and precomputes the root of an empty subtree at every level,
  so empty regions cost nothing to store.
* `MerklePatriciaTrie` follows Ethereum's node layout and has no padding.

Because empty slots hash to <Rust inline>`H::empty()`</Rust><TypeScript inline>`hasher.empty()`</TypeScript>, **empty leaf data is rejected** with
<Rust inline>`MerkleError::EmptyLeafData`</Rust><TypeScript inline>a `MerkleError` with code `'EmptyLeafData'`</TypeScript>; otherwise an inserted empty value would be indistinguishable from
padding.

## One caveat: the leaf count is not committed

<Rust inline>`MerkleProof::leaf_count`</Rust><TypeScript inline>`MerkleProof.leafCount`</TypeScript> tells the verifier how deep the path should be. The root does **not**
commit to it, so any count with the same padded size verifies. If the exact number of leaves matters
to you, get it from a trusted source and compare.
