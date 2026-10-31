# Sparse Merkle tree

<Rust inline>`SparseMerkleTree<H>`</Rust><TypeScript inline>`SparseMerkleTree`</TypeScript> models a fixed-depth **256-bit key space**: 2²⁵⁶ leaf slots, almost all
empty. It stores only non-empty nodes, precomputes the empty-subtree hash at every level, and
compresses single-leaf subtrees into shortcut nodes (Buterin, 2018) and small subtrees into batch
entries (Ouvrard, 2018/2019).

Use it for account state, key-value commitments, allowlists, and rollup state, especially when you
must prove that a key is **absent**.

<LangBar />

## Keys and values

* A key is <Rust inline>`[u8; 32]`</Rust><TypeScript inline>a 32-byte `Uint8Array`</TypeScript>. If your natural key is longer or variable-length (an address, a user ID),
  hash it to 32 bytes first.
* A value is any non-empty byte string. The tree stores <Rust inline>`H::hash(value)`</Rust><TypeScript inline>`hasher.hash(value)`</TypeScript>.

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Sha256;

  /// Derive a 32-byte tree key from an arbitrary identifier.
  fn key_for(id: &str) -> [u8; 32] {
      Sha256::hash(id.as_bytes())
  }

  fn main() -> Result<(), MerkleError> {
      let mut state = SparseMerkleTree::<Sha256>::new();
      assert_eq!(state.root(), None); // root() is None while empty
      assert_eq!(state.depth(), 256);

      state.insert(key_for("alice"), b"balance:100")?;
      state.insert(key_for("bob"), b"balance:250")?;

      // Inserting an existing key overwrites its value.
      state.insert(key_for("alice"), b"balance:90")?;
      assert_eq!(state.leaf_count(), 2);

      // get() returns the stored leaf *hash*, not the value.
      assert_eq!(state.get(key_for("alice")), Some(&Sha256::hash(b"balance:90")));
      assert_eq!(state.get(key_for("carol")), None);

      state.remove(key_for("bob"))?;
      assert_eq!(state.leaf_count(), 1);
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  /** Derive a 32-byte tree key from an arbitrary identifier. */
  const keyFor = (id: string) => Sha256.hash(utf8(id))

  const state = new SparseMerkleTree(Sha256)
  console.assert(state.root() === undefined) // root() is undefined while empty
  console.assert(state.depth === 256)

  state.insert(keyFor('alice'), utf8('balance:100'))
  state.insert(keyFor('bob'), utf8('balance:250'))

  // Inserting an existing key overwrites its value.
  state.insert(keyFor('alice'), utf8('balance:90'))
  console.assert(state.leafCount === 2)

  // get() returns the stored leaf *hash*, not the value.
  console.assert(bytesEqual(state.get(keyFor('alice'))!, Sha256.hash(utf8('balance:90'))))
  console.assert(state.get(keyFor('carol')) === undefined)

  state.remove(keyFor('bob'))
  console.assert(state.leafCount === 1)
  ```
</TypeScript>

`root()` returns <Rust inline>`None`</Rust><TypeScript inline>`undefined`</TypeScript> for an empty tree. If you need the canonical root of an empty tree (for
example as a genesis state), use <Rust inline>`empty_root()`</Rust><TypeScript inline>`emptyRoot()`</TypeScript>.

## Membership proofs

<Rust>
  ```rust
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut state = SparseMerkleTree::<Sha256>::new();
      let alice = [7_u8; 32];
      let bob = [9_u8; 32];

      state.insert(alice, b"balance:100")?;
      state.insert(bob, b"balance:250")?;

      let proof = state.generate_membership_proof(alice)?;
      let root = state.root().expect("state is not empty");

      assert!(proof.is_membership());
      assert_eq!(proof.siblings.len(), 256);

      // Checks the key, the value, and the path.
      assert!(SparseMerkleTree::<Sha256>::verify_membership(
          root,
          alice,
          b"balance:100",
          &proof,
      ));
      // The same proof does not verify for another key or value.
      assert!(!SparseMerkleTree::<Sha256>::verify_membership(root, bob, b"balance:100", &proof));
      assert!(!SparseMerkleTree::<Sha256>::verify_membership(root, alice, b"balance:999", &proof));
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { isSparseMembershipProof, SparseMerkleTree } from '@merkleforge/variants'

  const state = new SparseMerkleTree(Sha256)
  const alice = new Uint8Array(32).fill(7)
  const bob = new Uint8Array(32).fill(9)

  state.insert(alice, utf8('balance:100'))
  state.insert(bob, utf8('balance:250'))

  const proof = state.generateMembershipProof(alice)
  const root = state.root()!

  console.assert(isSparseMembershipProof(proof))
  console.assert(proof.siblings.length === 256)

  // Checks the key, the value, and the path.
  console.assert(SparseMerkleTree.verifyMembership(Sha256, root, alice, utf8('balance:100'), proof))
  // The same proof does not verify for another key or value.
  console.assert(!SparseMerkleTree.verifyMembership(Sha256, root, bob, utf8('balance:100'), proof))
  console.assert(!SparseMerkleTree.verifyMembership(Sha256, root, alice, utf8('balance:999'), proof))
  ```
</TypeScript>

<Rust>
  A `SparseMerkleProof<D>` has three public fields: `key`, `siblings` (256 entries ordered leaf to
  root, with `None` for empty subtrees), and `leaf` (the stored leaf hash, or `None`).

  :::warning
  Use `verify_membership`, which checks that the proof is for the key you expect. The older
  `SparseMerkleTree::verify` does **not** check the key and is deprecated.
  :::
</Rust>

<TypeScript>
  A `SparseMerkleProof` is a plain object with three fields: `key`, `siblings` (256 entries ordered
  leaf to root, with `null` for empty subtrees), and `leaf` (the stored leaf hash, or `null`).
  `verifyMembership` and `verifyNonMembership` always check that the proof is for the key you
  expect; there is no key-blind `verify`.
</TypeScript>

## Non-membership proofs

The same function proves absence. For a missing key,
<Rust inline>`generate_membership_proof` returns a proof with `leaf: None`</Rust><TypeScript inline>`generateMembershipProof` returns a proof with `leaf: null`</TypeScript>,
and the verifier rebuilds the root with <Rust inline>`H::empty()`</Rust><TypeScript inline>`hasher.empty()`</TypeScript>
in the leaf slot.

<Rust>
  ```rust
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut state = SparseMerkleTree::<Sha256>::new();
      state.insert([1_u8; 32], b"registered")?;

      let missing = [2_u8; 32];
      let proof = state.generate_membership_proof(missing)?;
      let root = state.root().expect("non-empty");

      assert!(proof.is_non_membership());
      assert!(SparseMerkleTree::<Sha256>::verify_non_membership(root, missing, &proof));

      // A non-membership proof can't be replayed for a key that *is* present.
      let present = state.generate_membership_proof([1_u8; 32])?;
      assert!(!SparseMerkleTree::<Sha256>::verify_non_membership(root, [1_u8; 32], &present));
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { isSparseMembershipProof, SparseMerkleTree } from '@merkleforge/variants'

  const state = new SparseMerkleTree(Sha256)
  const registered = new Uint8Array(32).fill(1)
  state.insert(registered, utf8('registered'))

  const missing = new Uint8Array(32).fill(2)
  const proof = state.generateMembershipProof(missing)
  const root = state.root()!

  console.assert(!isSparseMembershipProof(proof)) // proof.leaf === null
  console.assert(SparseMerkleTree.verifyNonMembership(Sha256, root, missing, proof))

  // A non-membership proof can't be replayed for a key that *is* present.
  const present = state.generateMembershipProof(registered)
  console.assert(!SparseMerkleTree.verifyNonMembership(Sha256, root, registered, present))
  ```
</TypeScript>

## Batch operations

<Rust inline>`batch_insert` and `batch_remove`</Rust><TypeScript inline>`batchInsert` and `batchRemove`</TypeScript> apply every change first, then recompute each affected node
once (one-phase batch update, Ma et al.). For many updates this is much faster than a loop of
single inserts.

<Rust>
  ```rust
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut batched = SparseMerkleTree::<Sha256>::new();
      let mut sequential = SparseMerkleTree::<Sha256>::new();

      let updates: Vec<([u8; 32], Vec<u8>)> = (0_u8..100)
          .map(|i| ([i; 32], format!("account:{i}").into_bytes()))
          .collect();

      batched.batch_insert(&updates)?;
      for (key, value) in &updates {
          sequential.insert(*key, value)?;
      }
      assert_eq!(batched.root(), sequential.root());
      assert_eq!(batched.leaf_count(), 100);

      // Remove several keys in one pass. Every key must exist.
      batched.batch_remove(&[[0_u8; 32], [1_u8; 32], [2_u8; 32]])?;
      assert_eq!(batched.leaf_count(), 97);
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  const batched = new SparseMerkleTree(Sha256)
  const sequential = new SparseMerkleTree(Sha256)

  const key = (i: number) => new Uint8Array(32).fill(i)
  const updates = Array.from({ length: 100 }, (_, i) => [key(i), utf8(`account:${i}`)] as const)

  batched.batchInsert(updates)
  for (const [k, value] of updates) sequential.insert(k, value)
  console.assert(bytesEqual(batched.root()!, sequential.root()!))
  console.assert(batched.leafCount === 100)

  // Remove several keys in one pass. Every key must exist.
  batched.batchRemove([key(0), key(1), key(2)])
  console.assert(batched.leafCount === 97)
  ```
</TypeScript>

Rules:

* If a key appears more than once in <Rust inline>`batch_insert`</Rust><TypeScript inline>`batchInsert`</TypeScript>, the **last** value wins.
* <Rust inline>`batch_insert` fails</Rust><TypeScript inline>`batchInsert` throws</TypeScript> with `EmptyLeafData` if any value is empty, before changing the tree.
* <Rust inline>`batch_remove` fails</Rust><TypeScript inline>`batchRemove` throws</TypeScript> with `UnsupportedOperation` if any key is missing, before changing the tree.
* Single `remove` of a missing key also <Rust inline>returns</Rust><TypeScript inline>throws</TypeScript> `UnsupportedOperation`.

In the published Rust benchmark run (2026-08-17), a 1,000-key batch insert took 20.33 ms versus 8.69 s
for 1,000 sequential inserts. That run predates the v0.5.0 fix that makes single inserts
recompute only the changed path, so today's sequential numbers are far lower; batching still saves
work on shared ancestors. See [Benchmarks](/docs/performance/benchmarks).

## Empty hashes and storage introspection

<Rust>
  ```rust
  use merkle_core::prelude::*;
  use merkle_variants::SparseMerkleTree;
  use merkleforge_hash::Sha256;

  fn main() -> Result<(), MerkleError> {
      let mut tree = SparseMerkleTree::<Sha256>::new();

      // Level 0 is the empty leaf; level 256 is the empty-tree root.
      assert_eq!(tree.empty_hash_at(0), &Sha256::empty());
      assert_eq!(tree.empty_root(), tree.empty_hash_at(256));
      assert_eq!(tree.empty_hashes().len(), 257);

      tree.insert([0xAB; 32], b"value")?;
      println!("materialized nodes: {}", tree.stored_node_count());
      println!("nodes written by last mutation: {}", tree.last_node_access_count());
      println!("estimate without batching: {}", tree.unbatched_node_count_estimate());
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  const tree = new SparseMerkleTree(Sha256)

  // Level 0 is the empty leaf; level 256 is the empty-tree root.
  console.assert(bytesEqual(tree.emptyHashAt(0), Sha256.empty()))
  console.assert(bytesEqual(tree.emptyRoot(), tree.emptyHashAt(256)))
  console.assert(tree.emptyHashes().length === 257)

  tree.insert(new Uint8Array(32).fill(0xab), utf8('value'))
  console.log('materialized nodes:', tree.storedNodeCount)
  console.log('nodes written by last mutation:', tree.lastNodeAccessCount)
  console.log('estimate without batching:', tree.unbatchedNodeCountEstimate())
  ```
</TypeScript>

<Rust>
  The `merkle_variants::sparse` module also exports the constants `SPARSE_TREE_DEPTH` (256),
  `EMPTY_HASH_LEVELS` (257), and `BATCH_DEPTH` (4), and the helper `bit_at(key, depth)` that
  returns the path bit for a key at a depth (bit 0 is the high bit of `key[0]`).
</Rust>

<TypeScript>
  `@merkleforge/variants` also exports the constants `SPARSE_TREE_DEPTH` (256),
  `EMPTY_HASH_LEVELS` (257), `BATCH_DEPTH` (4), and `SPARSE_KEY_BYTES` (32), and the helper
  `bitAt(key, depth)` that returns the path bit for a key at a depth as a boolean (bit 0 is the high
  bit of `key[0]`).
</TypeScript>

## Verifying on-chain

<Rust>
  Proof formatting for Solidity is TypeScript only. Switch to TypeScript to see it, or encode the
  proof yourself for the `MerkleForgeSparse` contract in
  [MerkleForge-ts/contracts](https://github.com/1888-Labs/MerkleForge-ts/tree/main/contracts).
</Rust>

<TypeScript>
  A Keccak-256 sparse proof carries 256 siblings, most of them empty. `toSoliditySparseProof`
  drops the empty ones and records which levels are present in a bitmap, producing the arguments
  for `MerkleForgeSparse.verifyMembership` / `verifyNonMembership` (see
  [contracts/](https://github.com/1888-Labs/MerkleForge-ts/tree/main/contracts)). Values are
  `0x` hex strings and a `bigint`, ready for viem or ethers.

  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { SparseMerkleTree, toSoliditySparseProof } from '@merkleforge/variants'

  const state = new SparseMerkleTree(Keccak256)
  const alice = Keccak256.hash(utf8('alice'))
  state.insert(alice, utf8('balance:100'))
  state.insert(Keccak256.hash(utf8('bob')), utf8('balance:250'))

  const proof = toSoliditySparseProof(state.generateMembershipProof(alice))
  // Only the one non-empty sibling is kept; bit `level` of `bitmap` marks where it sits.
  console.assert(proof.siblings.length === 1)
  console.log(proof)
  ```
</TypeScript>

## Through the generic trait

<Rust>
  `SparseMerkleTree` implements `MerkleTree<H>` by mapping leaf index `i` to a key whose last 8 bytes
  are `i` (big-endian). `MerkleTree::generate_proof` returns an ordinary `MerkleProof` with 256 path
  nodes. Prefer the key-addressed API above for real state.
</Rust>

<TypeScript>
  `smt.indexed()` returns a `MerkleTree` view that shares the tree's state and maps leaf index `i`
  to a key whose last 8 bytes are `i` (big-endian). Its `insert` takes the first free index at or
  above the leaf count, so it never overwrites a leaf. `generateProof` returns an ordinary
  `MerkleProof` with 256 path nodes. Prefer the key-addressed API above for real state.

  ```ts
  import { utf8 } from '@merkleforge/core'
  import { Sha256 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  const state = new SparseMerkleTree(Sha256)
  const view = state.indexed()

  const index = view.insert(utf8('first'))
  const proof = view.generateProof(index)
  console.assert(proof.path.length === 256)
  ```
</TypeScript>
