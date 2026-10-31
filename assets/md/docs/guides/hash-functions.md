# Hash functions

Every MerkleForge tree is parameterised by a `HashFunction`. The hash decides the digest, how
leaves and internal nodes are hashed, and what an empty slot looks like.

<LangBar />

<Rust>
  Trees are generic over `H: HashFunction`. Swapping the hash is a type-level change: Rust
  monomorphises the generics, so there is no runtime dispatch.
</Rust>

<TypeScript>
  A `HashFunction` is a plain object passed to the tree's constructor (and to the static verifiers).
  Swapping the hash means passing a different object; no other code changes.
</TypeScript>

## The adapters

<Rust inline>`merkleforge-hash`</Rust><TypeScript inline>`@merkleforge/hash`</TypeScript> ships
three adapters. All produce 32-byte
<Rust inline>`[u8; 32]`</Rust><TypeScript inline>`Uint8Array`</TypeScript> digests.

| Type | Algorithm | <Rust inline>`algorithm_name()`</Rust><TypeScript inline>`algorithmName`</TypeScript> | Best for |
| --- | --- | --- | --- |
| `Sha256` | SHA-256 | `"SHA-256"` | Interoperable binary and sparse trees; hardware SHA extensions |
| `Keccak256` | Keccak-256 | `"Keccak-256"` | Ethereum-compatible Patricia roots |
| `Blake3` | BLAKE3 | `"BLAKE3"` | Highest software throughput when Ethereum compatibility isn't needed |

<Rust>
  ```rust
  use merkle_core::traits::HashFunction;
  use merkleforge_hash::{Blake3, Keccak256, Sha256};

  fn main() {
      let sha = Sha256::hash(b"alice:100");
      let keccak = Keccak256::hash(b"alice:100");
      let blake = Blake3::hash(b"alice:100");

      assert_eq!(sha.len(), 32);
      assert_ne!(sha, keccak);
      assert_ne!(sha, blake);

      assert_eq!(Blake3::algorithm_name(), "BLAKE3");
      assert_eq!(Blake3::digest_size(), 32);

      // Combine two child digests into a parent.
      let left = Blake3::hash(b"left leaf");
      let right = Blake3::hash(b"right leaf");
      let _parent = Blake3::hash_nodes(&left, &right);
  }
  ```

  `merkleforge_hash` re-exports `HashFunction`, so `use merkleforge_hash::HashFunction;` also works.
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Blake3, Keccak256, Sha256 } from '@merkleforge/hash'

  const sha = Sha256.hash(utf8('alice:100'))
  const keccak = Keccak256.hash(utf8('alice:100'))
  const blake = Blake3.hash(utf8('alice:100'))

  console.assert(sha.length === 32)
  console.assert(!bytesEqual(sha, keccak))
  console.assert(!bytesEqual(sha, blake))

  console.assert(Blake3.algorithmName === 'BLAKE3')
  console.assert(Blake3.digestSize === 32)

  // Combine two child digests into a parent.
  const left = Blake3.hash(utf8('left leaf'))
  const right = Blake3.hash(utf8('right leaf'))
  const parent = Blake3.hashNodes(left, right)
  ```

  The `HashFunction` type itself is exported from `@merkleforge/core`. The adapters are built on
  [`@noble/hashes`](https://github.com/paulmillr/noble-hashes), so they run unchanged in Node.js,
  browsers, Deno, and Bun.
</TypeScript>

## Domain separation

| Adapter | Leaf: `hash(data)` | Internal node: <Rust inline>`hash_nodes(l, r)`</Rust><TypeScript inline>`hashNodes(l, r)`</TypeScript> | Encoded node: <Rust inline>`hash_encoded_node(rlp)`</Rust><TypeScript inline>`hashEncodedNode(rlp)`</TypeScript> |
| --- | --- | --- | --- |
| `Sha256` | `SHA-256(0x00 ‖ data)` | `SHA-256(0x01 ‖ l ‖ r)` | `SHA-256(0x00 ‖ rlp)` (default) |
| `Keccak256` | `Keccak-256(0x00 ‖ data)` | `Keccak-256(0x01 ‖ l ‖ r)` | `Keccak-256(rlp)`, plain, Ethereum-compatible |
| `Blake3` | `derive_key("MerkleForge 2026 leaf v1", data)` | `derive_key("MerkleForge 2026 internal-node v1", l ‖ r)` | Same as leaf (default) |

`empty()` is the leaf hash of an empty slice,
<Rust inline>`hash(b"")`</Rust><TypeScript inline>`hash(new Uint8Array())`</TypeScript>, for all
three adapters. Binary trees pad with it and sparse trees use it for vacant slots.

Why it matters: see [Mental model → Domain separation](/docs/mental-model#domain-separation-0x00-and-0x01).

<Rust>
  ```rust
  use merkle_core::traits::HashFunction;
  use merkleforge_hash::Sha256;
  use sha2::{Digest, Sha256 as RawSha256};

  fn main() {
      // The adapter's leaf hash is SHA-256 over 0x00 ‖ data.
      let mut raw = RawSha256::new();
      raw.update([0x00]);
      raw.update(b"alice:100");
      let expected: [u8; 32] = raw.finalize().into();

      assert_eq!(Sha256::hash(b"alice:100"), expected);
      assert_eq!(Sha256::empty(), Sha256::hash(b""));
  }
  ```

  (This snippet uses the [`sha2`](https://crates.io/crates/sha2) crate directly to show the
  pre-image. You don't need it to use MerkleForge.)
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { sha256 } from '@noble/hashes/sha2.js'

  // The adapter's leaf hash is SHA-256 over 0x00 ‖ data.
  const expected = sha256.create().update(Uint8Array.of(0x00)).update(utf8('alice:100')).digest()

  console.assert(bytesEqual(Sha256.hash(utf8('alice:100')), expected))
  console.assert(bytesEqual(Sha256.empty(), Sha256.hash(new Uint8Array())))
  ```

  (This snippet uses [`@noble/hashes`](https://github.com/paulmillr/noble-hashes) directly to show
  the pre-image. It is already a dependency of `@merkleforge/hash`, but import it explicitly only if
  you need raw hashes.)
</TypeScript>

## Choosing one

* **Ethereum interop:** `Keccak256`. Required for Patricia roots that match Ethereum.
* **Broad interoperability:** `Sha256`. Ubiquitous, and fast on CPUs with SHA extensions.
* **Raw speed:** `Blake3`. In the published run it hashed 1 MB at 5.99 GiB/s versus 1.28 GiB/s for
  SHA-256 and 291 MiB/s for Keccak-256. See [Benchmarks](/docs/performance/benchmarks).

<TypeScript>
  Those throughput figures are for the Rust crates. In JavaScript, the WebAssembly adapters below are
  the fastest option for Keccak-256 and SHA-256.
</TypeScript>

A tree built with one hash can't be read with another: serialized trees store the algorithm
name and deserialization rejects a mismatch.

## WebAssembly hashes

<Rust>
  This is a TypeScript-only package: the Rust adapters are already native code. Switch to
  **TypeScript** to see it.
</Rust>

<TypeScript>
  [`@merkleforge/hash-wasm`](https://www.npmjs.com/package/@merkleforge/hash-wasm) provides
  WebAssembly versions of `Keccak256` and `Sha256`, about 6x faster for Keccak-256 in Node.js. Their
  output and `algorithmName` are identical to `@merkleforge/hash`, so trees, proofs, and JSON built
  with either are interchangeable. The factories are async because they load the WebAssembly module;
  the returned adapter is synchronous.

  ```bash
  npm i @merkleforge/hash-wasm
  ```

  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { createKeccak256Wasm } from '@merkleforge/hash-wasm'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  // Loads the WebAssembly module once; the adapter is synchronous after that.
  const KeccakWasm = await createKeccak256Wasm()

  const rows = ['alice:100', 'bob:250', 'carol:75'].map(utf8)
  const fast = BinaryMerkleTree.fromLeaves(KeccakWasm, rows)
  const plain = BinaryMerkleTree.fromLeaves(Keccak256, rows)

  // Same output and algorithmName: trees and proofs are interchangeable.
  console.assert(bytesEqual(fast.root()!, plain.root()!))
  console.assert(KeccakWasm.algorithmName === Keccak256.algorithmName)
  ```

  `createSha256Wasm()` works the same way. There is no WebAssembly BLAKE3.
</TypeScript>

## The `HashFunction` trait

<Rust>
  ```rust
  use core::fmt::Debug;

  pub trait HashFunction: Send + Sync + 'static {
      type Digest: AsRef<[u8]> + Clone + Debug + PartialEq + Eq + Send + Sync + 'static;

      /// Leaf hash. Must apply the leaf domain, e.g. H(0x00 ‖ data).
      fn hash(data: &[u8]) -> Self::Digest;

      /// Parent of two children. Default: hash(0x01 ‖ left ‖ right).
      fn hash_nodes(left: &Self::Digest, right: &Self::Digest) -> Self::Digest {
          let mut buf = Vec::with_capacity(1 + left.as_ref().len() + right.as_ref().len());
          buf.push(0x01);
          buf.extend_from_slice(left.as_ref());
          buf.extend_from_slice(right.as_ref());
          Self::hash(&buf)
      }

      /// Hash of a self-describing encoding such as an RLP Patricia node. Default: hash(encoded).
      fn hash_encoded_node(encoded: &[u8]) -> Self::Digest {
          Self::hash(encoded)
      }

      /// Digest of an empty slot. Default: hash(&[]).
      fn empty() -> Self::Digest {
          Self::hash(&[])
      }

      fn algorithm_name() -> &'static str;
      fn digest_size() -> usize;
  }
  ```

  This mirrors the definition in `merkle_core::traits::hash_function` (doc comments shortened).
</Rust>

<TypeScript>
  In TypeScript it is an interface; digests are always `Uint8Array`, so there is no associated
  `Digest` type:

  ```ts
  import type { Bytes } from '@merkleforge/core'

  export interface HashFunction {
    /** Human-readable name, e.g. "SHA-256". Stored in tree metadata. */
    readonly algorithmName: string
    /** Digest length in bytes. */
    readonly digestSize: number
    /** Leaf hash. Must apply the leaf domain, e.g. H(0x00 ‖ data). */
    hash(data: Bytes): Bytes
    /** Parent of two children, e.g. H(0x01 ‖ left ‖ right). */
    hashNodes(left: Bytes, right: Bytes): Bytes
    /** Hash of a self-describing encoding such as an RLP Patricia node. */
    hashEncodedNode(encoded: Bytes): Bytes
    /** Digest of an empty slot. */
    empty(): Bytes
  }
  ```

  This mirrors the definition in `@merkleforge/core` (doc comments shortened). Interfaces can't
  carry default methods, so the defaults come from `defineHashFunction` instead: `hashNodes` hashes
  `0x01 ‖ left ‖ right` with `hash`, `hashEncodedNode` delegates to `hash`, and `empty` is `hash` of
  no bytes, the same defaults as the Rust trait.
</TypeScript>

## Writing a custom adapter

<Rust>
  Implement `hash`, `algorithm_name`, and `digest_size`. Apply the `0x00` leaf prefix in `hash`; the
  default `hash_nodes` applies `0x01`. This example wraps the `sha2` crate's SHA-512/256:

  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::BinaryMerkleTree;
  use sha2::{Digest, Sha512_256};

  pub struct Sha512Trunc;

  impl HashFunction for Sha512Trunc {
      type Digest = [u8; 32];

      fn hash(data: &[u8]) -> [u8; 32] {
          let mut hasher = Sha512_256::new();
          hasher.update([0x00]); // leaf domain
          hasher.update(data);
          hasher.finalize().into()
      }

      // hash_nodes, hash_encoded_node, and empty use the defaults.

      fn algorithm_name() -> &'static str {
          "SHA-512/256"
      }

      fn digest_size() -> usize {
          32
      }
  }

  fn main() -> Result<(), MerkleError> {
      let tree = BinaryMerkleTree::<Sha512Trunc>::from_leaves(["a", "b", "c"])?;
      let root = tree.root().expect("non-empty");
      let proof = tree.generate_proof(LeafIndex(2))?;
      assert!(BinaryMerkleTree::<Sha512Trunc>::verify(root, b"c", &proof));
      assert_eq!(tree.metadata().hash_algorithm, "SHA-512/256");
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  Pass `algorithmName`, `digestSize`, and `hash` to `defineHashFunction`, which fills in the rest
  and returns a frozen `HashFunction`. Apply the `0x00` leaf prefix (`LEAF_PREFIX`) in `hash`; the
  default `hashNodes` applies `0x01` (`NODE_PREFIX`). This example wraps `@noble/hashes`'
  SHA-512/256:

  ```ts
  import { concatBytes, defineHashFunction, LEAF_PREFIX, utf8 } from '@merkleforge/core'
  import { BinaryMerkleTree } from '@merkleforge/variants'
  import { sha512_256 } from '@noble/hashes/sha2.js'

  export const Sha512Trunc = defineHashFunction({
    algorithmName: 'SHA-512/256',
    digestSize: 32,
    hash: (data) => sha512_256(concatBytes(Uint8Array.of(LEAF_PREFIX), data)), // leaf domain
    // hashNodes, hashEncodedNode, and empty use the defaults.
  })

  const tree = BinaryMerkleTree.fromLeaves(Sha512Trunc, ['a', 'b', 'c'].map(utf8))
  const root = tree.root()!
  const proof = tree.generateProof(2)
  console.assert(BinaryMerkleTree.verify(Sha512Trunc, root, utf8('c'), proof))
  console.assert(tree.metadata().hashAlgorithm === 'SHA-512/256')
  ```

  An adapter built this way works with every tree and verifier, but only matches a Rust tree if both
  sides compute the same bytes.
</TypeScript>

Checklist for a custom adapter:

1. **Separate the domains.** `hash` must not collide with
   <Rust inline>`hash_nodes`</Rust><TypeScript inline>`hashNodes`</TypeScript> for different
   structures. Prefix leaves with `0x00`, or override both methods with your own separation (as
   `Blake3` does with context strings).
2. **Override for speed if you like.**
   <Rust inline>`hash_nodes` can hash `0x01 ‖ l ‖ r` without the intermediate `Vec`, as the shipped
   adapters do.</Rust><TypeScript inline>`hashNodes` can hash `0x01 ‖ l ‖ r` with an incremental
   hasher instead of concatenating, as `@merkleforge/hash-wasm` does.</TypeScript> Keep the same
   output.
3. **Ethereum roots** need
   <Rust inline>`hash_encoded_node`</Rust><TypeScript inline>`hashEncodedNode`</TypeScript> to
   return the plain hash of the RLP bytes.
4. **Use a unique <Rust inline>`algorithm_name`</Rust><TypeScript inline>`algorithmName`</TypeScript>.**
   Serialized trees store it to reject mismatched hashes.

<Rust>
  :::warning
  Since v0.5.0, the default `hash_nodes` hashes `0x01 ‖ left ‖ right` instead of
  `left ‖ right`. Custom adapters that relied on the old default now produce different roots.
  The shipped adapters override `hash_nodes` and are unaffected.
  :::
</Rust>
