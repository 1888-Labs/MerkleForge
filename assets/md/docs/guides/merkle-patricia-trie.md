# Merkle Patricia trie

<Rust inline>`MerklePatriciaTrie<H>`</Rust><TypeScript inline>`MerklePatriciaTrie`</TypeScript> implements Ethereum's Merkle Patricia Trie (MPT): nibble-addressed byte
keys, four node types (empty, leaf, extension, branch), hex-prefix path encoding, RLP-encoded
nodes, and Ethereum's inline-or-hash node references.

With `Keccak256`, roots **match Ethereum**: the implementation is tested against the official
`trietest.json` and `trieanyorder.json` vectors from the Ethereum test suite.

<LangBar />

## Insert, get, remove

<Rust>
  ```rust
  use merkle_variants::MerklePatriciaTrie;
  use merkleforge_hash::Keccak256;

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut trie = MerklePatriciaTrie::<Keccak256>::new();

      trie.insert(b"account:alice", b"100")?;
      trie.insert(b"account:bob", b"250")?;
      trie.insert(b"account:alice", b"90")?; // overwrite

      assert_eq!(trie.get(b"account:alice"), Some(b"90".as_slice()));
      assert_eq!(trie.get(b"account:carol"), None);
      assert_eq!(trie.leaf_count(), 2);

      // Removal collapses branches and extensions, so the root equals that of a
      // trie built from the remaining entries.
      trie.remove(b"account:bob")?;

      let mut fresh = MerklePatriciaTrie::<Keccak256>::new();
      fresh.insert(b"account:alice", b"90")?;
      assert_eq!(trie.root(), fresh.root());
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { MerklePatriciaTrie } from '@merkleforge/variants'

  const trie = new MerklePatriciaTrie(Keccak256)

  trie.insert(utf8('account:alice'), utf8('100'))
  trie.insert(utf8('account:bob'), utf8('250'))
  trie.insert(utf8('account:alice'), utf8('90')) // overwrite

  console.assert(bytesEqual(trie.get(utf8('account:alice'))!, utf8('90')))
  console.assert(trie.get(utf8('account:carol')) === undefined)
  console.assert(trie.leafCount === 2)

  // Removal collapses branches and extensions, so the root equals that of a
  // trie built from the remaining entries.
  trie.remove(utf8('account:bob'))

  const fresh = new MerklePatriciaTrie(Keccak256)
  fresh.insert(utf8('account:alice'), utf8('90'))
  console.assert(bytesEqual(trie.root()!, fresh.root()!))
  ```
</TypeScript>

Unlike the sparse tree, `get` returns the stored **value** bytes. Values must be non-empty
(`EmptyLeafData`), and removing a missing key <Rust inline>returns</Rust><TypeScript inline>throws</TypeScript>
`UnsupportedOperation`. `root()` is <Rust inline>`None`</Rust><TypeScript inline>`undefined`</TypeScript>
for an empty trie.

## Ethereum compatibility

The trie produces the same root as Ethereum clients for the same key-value set. This is the
`dogs` case from Ethereum's `trieanyorder.json`:

<Rust>
  ```rust
  use merkle_variants::MerklePatriciaTrie;
  use merkleforge_hash::Keccak256;

  fn hex32(s: &str) -> [u8; 32] {
      let s = s.trim_start_matches("0x");
      let mut out = [0_u8; 32];
      for (i, byte) in out.iter_mut().enumerate() {
          *byte = u8::from_str_radix(&s[2 * i..2 * i + 2], 16).expect("valid hex");
      }
      out
  }

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut trie = MerklePatriciaTrie::<Keccak256>::new();
      trie.insert(b"doe", b"reindeer")?;
      trie.insert(b"dog", b"puppy")?;
      trie.insert(b"dogglesworth", b"cat")?;

      let expected = hex32("0x8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3");
      assert_eq!(trie.root(), Some(&expected));
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { toHex, utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { MerklePatriciaTrie } from '@merkleforge/variants'

  const trie = new MerklePatriciaTrie(Keccak256)
  trie.insert(utf8('doe'), utf8('reindeer'))
  trie.insert(utf8('dog'), utf8('puppy'))
  trie.insert(utf8('dogglesworth'), utf8('cat'))

  const expected = '0x8aad789dff2f538bca5d8ea56e8abe10f4c7ba3a5dea95fea4cd6e7c3a1168d3'
  console.assert(toHex(trie.root()!) === expected)
  ```
</TypeScript>

Insertion order does not matter: the trie is canonical.

How compatibility works: the `Keccak256` adapter overrides
<Rust inline>`HashFunction::hash_encoded_node`</Rust><TypeScript inline>`HashFunction.hashEncodedNode`</TypeScript> to
return plain `Keccak-256(rlp)`, with no domain prefix, exactly as Ethereum hashes nodes. RLP already
distinguishes node types, so the `0x00`/`0x01` prefixes used by binary and sparse trees are not
needed here.

:::note
With `Sha256` or `Blake3` the trie works the same way but its roots are not Ethereum roots. A
custom adapter needs to override <Rust inline>`hash_encoded_node`</Rust><TypeScript inline>`hashEncodedNode` (a `defineHashFunction` option)</TypeScript>
to produce Ethereum-compatible roots; matching on the algorithm name no longer works (changed in
v0.5.0). The WebAssembly `createKeccak256Wasm()` from `@merkleforge/hash-wasm` already does this.
:::

## Witnesses (proofs)

<Rust inline>`generate_proof(key)` returns an `MptProof<D>`: the key, the value (or `None`)</Rust><TypeScript inline>`generateProof(key)` returns an `MptProof`: the key, the value (or `null`)</TypeScript>,
and the RLP-encoded nodes on the key's path, root first. It has the shape of an
[EIP-1186](https://eips.ethereum.org/EIPS/eip-1186) `accountProof` / `storageProof` entry.

<Rust>
  ```rust
  use merkle_variants::{MerklePatriciaTrie, MptProof};
  use merkleforge_hash::Keccak256;

  fn main() -> Result<(), merkle_core::error::MerkleError> {
      let mut trie = MerklePatriciaTrie::<Keccak256>::new();
      trie.insert(b"account:alice", b"100")?;
      trie.insert(b"account:bob", b"250")?;
      let root = *trie.root().expect("trie is not empty");

      // Membership
      let proof = trie.generate_proof(b"account:alice")?;
      assert!(proof.is_membership());
      assert_eq!(proof.value.as_deref(), Some(b"100".as_slice()));
      assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(&root, &proof));

      // Non-membership: the path ends at an empty slot or a divergent node.
      let absent = trie.generate_proof(b"account:carol")?;
      assert!(absent.is_non_membership());
      assert!(MerklePatriciaTrie::<Keccak256>::verify_proof(&root, &absent));

      // A forged value fails verification.
      let forged: MptProof<[u8; 32]> = MptProof::new(
          proof.key.clone(),
          Some(b"1000000".to_vec()),
          proof.proof_nodes.clone(),
      );
      assert!(!MerklePatriciaTrie::<Keccak256>::verify_proof(&root, &forged));
      Ok(())
  }
  ```
</Rust>

<TypeScript>
  ```ts
  import { bytesEqual, utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { MerklePatriciaTrie, type MptProof } from '@merkleforge/variants'

  const trie = new MerklePatriciaTrie(Keccak256)
  trie.insert(utf8('account:alice'), utf8('100'))
  trie.insert(utf8('account:bob'), utf8('250'))
  const root = trie.root()!

  // Membership
  const proof = trie.generateProof(utf8('account:alice'))
  console.assert(proof.value !== null && bytesEqual(proof.value, utf8('100')))
  console.assert(MerklePatriciaTrie.verifyProof(Keccak256, root, proof))

  // Non-membership: the path ends at an empty slot or a divergent node.
  const absent = trie.generateProof(utf8('account:carol'))
  console.assert(absent.value === null)
  console.assert(MerklePatriciaTrie.verifyProof(Keccak256, root, absent))

  // A forged value fails verification.
  const forged: MptProof = { ...proof, value: utf8('1000000') }
  console.assert(!MerklePatriciaTrie.verifyProof(Keccak256, root, forged))
  ```
</TypeScript>

<Rust inline>`verify_proof`</Rust><TypeScript inline>`verifyProof`</TypeScript> checks that the first
node hashes to the trusted root, then walks the key's nibble path through the witness. The
verifier needs only the root and the proof, so it works for light clients and cross-chain bridges.

<Rust>
  `MptProof` derives `Serialize` / `Deserialize`, so you can send it over the network directly.

  `generate_proof` on an empty trie returns `MerkleError::EmptyTree`.
</Rust>

<TypeScript>
  `MptProof` is a plain object with `key`, `value`, and `proofNodes`. To send it over the network,
  use `encodeMptProof` / `decodeMptProof` or `mptProofToSerdeJSON` / `mptProofFromSerdeJSON`, which
  produce the same bytes and JSON as the Rust crate (see [Serialization](/docs/guides/serialization)).
  TypeScript has no `isMembership()` helper: a proof with `value !== null` is a membership proof.

  `generateProof` on an empty trie throws `MerkleError` with code `EmptyTree`.

  `MerklePatriciaTrie.proveValue(hasher, root, key, proofNodes)` is the lower-level verifier: it
  returns the value the witness proves, `null` for a proven absence, or throws `InvalidProof`. It
  also accepts `eth_getProof`-style witnesses, where short child nodes are embedded in their parent
  rather than listed.
</TypeScript>

## Verifying `eth_getProof`

<Rust>
  `eth_getProof` verification is TypeScript only. Switch to TypeScript to see
  `@merkleforge/ethereum`, or verify the `accountProof` and `storageProof` node lists yourself with
  `verify_proof`: the account key is `keccak256(address)` and storage keys are `keccak256(slot)`.
</Rust>

<TypeScript>
  `@merkleforge/ethereum` verifies [EIP-1186](https://eips.ethereum.org/EIPS/eip-1186) responses
  against a state root. `verifyEthGetProof` checks the account proof, every storage proof, and that
  the response's own fields agree with them, and returns the decoded account (`null` if it does not
  exist) and slot values as `bigint`. It throws `InvalidProof` on any mismatch.
  `verifyAccountProof`, `verifyStorageProof`, and `mappingSlot` are also exported.

  ```bash
  npm i @merkleforge/ethereum
  ```

  ```ts
  import { toHex } from '@merkleforge/core'
  import { mappingSlot, verifyEthGetProof } from '@merkleforge/ethereum'

  const RPC_URL = 'https://ethereum-rpc.publicnode.com'
  const rpc = async (method: string, params: unknown[]) => {
    const res = await fetch(RPC_URL, {
      method: 'POST',
      headers: { 'content-type': 'application/json' },
      body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
    })
    return (await res.json()).result
  }

  // USDC on mainnet: balances live in the mapping at slot 9.
  const usdc = '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48'
  const holder = '0x37305B1cD40574E4C5Ce33f8e8306Be057fD7341'
  const slot = toHex(mappingSlot(holder, 9n))

  // Take the state root from a block header you trust (a light client, a
  // checkpoint). Here it comes from the same RPC, for brevity.
  const block = await rpc('eth_getBlockByNumber', ['latest', false])
  const response = await rpc('eth_getProof', [usdc, [slot], block.number])

  // Throws InvalidProof if any part of the response disagrees with the root.
  const { account, storage } = verifyEthGetProof(block.stateRoot, response)
  console.log('contract nonce:', account?.nonce)
  console.log('USDC balance (6 decimals):', storage[0]?.value)
  ```
</TypeScript>

## Low-level building blocks

<Rust>
  The `merkle_variants::patricia` module exposes the encoding primitives for tooling and tests:

  | Item | Purpose |
  | --- | --- |
  | `NibblePath` | Byte key to 4-bit nibble path (`from_key`, `len`, `get`, `slice`, `common_prefix_len`) |
  | `hp_encode` / `hp_decode` | Hex-prefix encoding for leaf and extension paths |
  | `rlp_encode` / `rlp_decode` | RLP for `MptNode` values |
  | `MptNode<D>` | `Empty`, `Leaf`, `Extension`, `Branch` |
  | `NodeRef<D>` | `Hash(D)` for nodes ≥ 32 bytes of RLP, `Inline(Vec<u8>)` otherwise |
  | `hash_node::<H>` | Ethereum-style reference for a node |
  | `compute_root_hash::<H>` | Root digest for a root node (always hashed) |

  ```rust
  use merkle_variants::patricia::{compute_root_hash, hp_encode, MptNode, NibblePath};
  use merkleforge_hash::Keccak256;

  fn main() {
      // The empty trie root, keccak256(rlp("")), well known from Ethereum.
      let empty = compute_root_hash::<Keccak256>(&MptNode::Empty);
      assert_eq!(empty[..4], [0x56, 0xe8, 0x1f, 0x17]);

      let path = NibblePath::from_key(b"key");
      assert_eq!(path.len(), 6); // two nibbles per byte

      // Hex-prefix flag for a leaf with an even-length path is 0x20.
      assert_eq!(hp_encode(&path, true)[0], 0x20);
  }
  ```
</Rust>

<TypeScript>
  `@merkleforge/variants` exports the same encoding primitives:

  | Item | Purpose |
  | --- | --- |
  | `bytesToNibbles` / `Nibbles` | Byte key to 4-bit nibble path (a plain `number[]`) |
  | `hpEncode` / `hpDecode` | Hex-prefix encoding for leaf and extension paths |
  | `rlpEncode` / `rlpDecode` | RLP for `MptNode` values |
  | `encodeRlp` / `decodeRlp` / `RlpValue` | General RLP for byte strings and nested lists |
  | `MptNode` | `{ kind: 'empty' \| 'leaf' \| 'extension' \| 'branch', … }` |
  | `NodeRef` | `{ kind: 'hash', hash }` for nodes ≥ 32 bytes of RLP, `{ kind: 'inline', rlp }` otherwise |
  | `hashNode(hasher, node)` | Ethereum-style reference for a node |
  | `computeRootHash(hasher, node)` | Root digest for a root node (always hashed) |

  ```ts
  import { toHex, utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { bytesToNibbles, computeRootHash, hpEncode } from '@merkleforge/variants'

  // The empty trie root, keccak256(rlp("")), well known from Ethereum.
  const empty = computeRootHash(Keccak256, { kind: 'empty' })
  console.assert(toHex(empty).startsWith('0x56e81f17'))

  const path = bytesToNibbles(utf8('key'))
  console.assert(path.length === 6) // two nibbles per byte

  // Hex-prefix flag for a leaf with an even-length path is 0x20.
  console.assert(hpEncode(path, true)[0] === 0x20)
  ```
</TypeScript>

## Through the generic trait

<Rust>
  `MerklePatriciaTrie` implements `MerkleTree<H>`: leaf index `i` is stored under
  `b"merkleforge:mpt:trait-leaf:" ‖ i` (8 bytes, big-endian). `MerkleTree::generate_proof` returns
  `UnsupportedOperation` because Patricia proofs are key-addressed; call the inherent
  `generate_proof(key)` instead.
</Rust>

<TypeScript>
  `trie.indexed()` returns a `MerkleTree` view that shares the trie's state: leaf index `i` is stored
  under `utf8('merkleforge:mpt:trait-leaf:') ‖ i` (8 bytes, big-endian). Its `generateProof` throws
  `UnsupportedOperation` because Patricia proofs are key-addressed; call
  `trie.generateProof(key)` instead.
</TypeScript>
