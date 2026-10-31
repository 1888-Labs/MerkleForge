# Why MerkleForge

MerkleForge is a framework for **authenticated data structures**, available as Rust crates and
TypeScript packages. You put data in, get a 32-byte root out, and hand out small proofs that
anyone holding the root can check, without downloading the rest of the data.

<LangBar />

## The problem

Data verification is one of the major bottlenecks in modern blockchain networks. As systems
process millions of transactions, checking the integrity of state data gets slow and
resource-intensive. Every state lookup, transaction batch, and light-client proof depends on the
speed and correctness of the underlying data structure.

<Rust>
  The Rust ecosystem has strong cryptographic building blocks, but Merkle tree support is spread
  across variant-specific crates. Libraries like `rs-merkle` focus on binary trees, while
  Ethereum-specific tries are maintained separately. Teams end up stitching together tools with
  different APIs, documentation depth, test coverage, and benchmark methodology.
</Rust>

<TypeScript>
  JavaScript and TypeScript have the same gap. Libraries like `merkletreejs` and
  `@openzeppelin/merkle-tree` focus on binary trees, while Ethereum-compatible tries are maintained
  separately. Teams end up stitching together tools with different APIs, proof formats, and test
  coverage, and proofs made in one language rarely verify in another.
</TypeScript>

## The solution

MerkleForge unifies the three Merkle tree variants used across production systems behind one
API:

<Rust>
  | Variant | Typical use |
  | --- | --- |
  | [`BinaryMerkleTree<H>`](/docs/guides/binary-merkle-tree) | Transaction batching, Bitcoin-style SPV, logs, file chunks |
  | [`SparseMerkleTree<H>`](/docs/guides/sparse-merkle-tree) | Account state, key-value maps, Layer-2 rollups |
  | [`MerklePatriciaTrie<H>`](/docs/guides/merkle-patricia-trie) | Ethereum state roots, receipts, storage witnesses |

  All three are generic over a [`HashFunction`](/docs/guides/hash-functions), implement the shared
  `MerkleTree<H>` trait, and produce proofs that verify **statelessly**.
</Rust>

<TypeScript>
  | Variant | Typical use |
  | --- | --- |
  | [`BinaryMerkleTree`](/docs/guides/binary-merkle-tree) | Transaction batching, airdrops and allowlists, logs, file chunks |
  | [`SparseMerkleTree`](/docs/guides/sparse-merkle-tree) | Account state, key-value maps, Layer-2 rollups |
  | [`MerklePatriciaTrie`](/docs/guides/merkle-patricia-trie) | Ethereum state roots, receipts, storage witnesses |

  All three take a [`HashFunction`](/docs/guides/hash-functions) object at construction, implement
  the shared `MerkleTree` interface, and produce proofs that verify **statelessly**.
</TypeScript>

## Features

<Rust>
  * **Unified API.** `BinaryMerkleTree<H>`, `SparseMerkleTree<H>`, and `MerklePatriciaTrie<H>` all
    implement `MerkleTree<H>`, so generic code works with any of them.
  * **Pluggable hash functions.** Swap SHA-256, Keccak-256, and BLAKE3 by changing one type
    parameter. Generics are monomorphised at compile time, so there is no runtime dispatch.
  * **Stateless verification.** A light client verifies a leaf with the trusted root, the proof, and
    the leaf bytes. See [Proofs & stateless verification](/docs/guides/proofs).
  * **Domain separation built in.** Leaves hash as `H(0x00 ‖ data)` and internal nodes as
    `H(0x01 ‖ left ‖ right)`, which blocks second-preimage attacks. See the
    [mental model](/docs/mental-model).
  * **Reproducible benchmarks.** A dedicated Criterion suite compares hashes, tree variants, and
    other Rust Merkle libraries. See [Benchmarks](/docs/performance/benchmarks).
  * **Tested.** Unit, integration, property-based (Proptest), and Ethereum test-vector tests.
  * **`#![forbid(unsafe_code)]`** in every published crate.
</Rust>

<TypeScript>
  * **Unified API.** `BinaryMerkleTree`, `SparseMerkleTree`, and `MerklePatriciaTrie` all implement
    the `MerkleTree` interface, so generic code works with any of them.
  * **Pluggable hash functions.** Swap SHA-256, Keccak-256, and BLAKE3 by passing a different
    `HashFunction` object, for example `BinaryMerkleTree.fromLeaves(Keccak256, leaves)`. The hash is
    a value, not a type parameter. `@merkleforge/hash-wasm` adds WebAssembly Keccak-256 and SHA-256
    with identical output.
  * **Byte-identical with Rust.** Roots, proofs, and proof encodings (bincode and JSON) match the Rust
    crates byte for byte, so a Rust server can prove and a browser can verify, or the other way round.
  * **Stateless verification.** A light client verifies a leaf with the trusted root, the proof, and
    the leaf bytes. See [Proofs & stateless verification](/docs/guides/proofs).
  * **Domain separation built in.** Leaves hash as `H(0x00 ‖ data)` and internal nodes as
    `H(0x01 ‖ left ‖ right)`, which blocks second-preimage attacks. See the
    [mental model](/docs/mental-model).
  * **Runs anywhere.** ESM only, on Node.js 22+, browsers, Deno, and Bun. The only runtime dependency
    is [`@noble/hashes`](https://github.com/paulmillr/noble-hashes).
  * **Ethereum and on-chain.** `eth_getProof` verification, an OpenZeppelin-compatible
    `StandardMerkleTree`, and Solidity verifiers for Keccak-256 proofs.
  * **Tested.** Unit, property-based (fast-check), cross-implementation vectors generated from the
    Rust crates, and Ethereum test-vector tests.
</TypeScript>

## Crates and packages

<Rust>
  | Crate | Role |
  | --- | --- |
  | [`merkle-core`](https://crates.io/crates/merkle-core) | Traits, proof types, serialization helpers, and errors. `no_std` + `alloc` capable. |
  | [`merkleforge-hash`](https://crates.io/crates/merkleforge-hash) | `Sha256`, `Keccak256`, and `Blake3` adapters. |
  | [`merkle-variants`](https://crates.io/crates/merkle-variants) | `BinaryMerkleTree`, `SparseMerkleTree`, and `MerklePatriciaTrie`. |
  | `merkle-bench` | Criterion benchmarks. Not published. |

  :::note
  The hash crate is published as `merkleforge-hash` (the original name was taken) and imported in
  Rust as `merkleforge_hash`. In the repository it lives in the `merkle-hash/` folder.
  :::
</Rust>

<TypeScript>
  | Package | Role |
  | --- | --- |
  | [`@merkleforge/core`](https://www.npmjs.com/package/@merkleforge/core) | `HashFunction` and `MerkleTree` interfaces, proof types, `MerkleError`, byte helpers. No runtime dependencies. |
  | [`@merkleforge/hash`](https://www.npmjs.com/package/@merkleforge/hash) | `Sha256`, `Keccak256`, and `Blake3` adapters, built on `@noble/hashes`. |
  | [`@merkleforge/variants`](https://www.npmjs.com/package/@merkleforge/variants) | `BinaryMerkleTree` (with multiproofs), `SparseMerkleTree`, `MerklePatriciaTrie`, and proof codecs. |
  | [`@merkleforge/hash-wasm`](https://www.npmjs.com/package/@merkleforge/hash-wasm) | WebAssembly `Keccak256` and `Sha256` with identical output. Optional. |
  | [`@merkleforge/ethereum`](https://www.npmjs.com/package/@merkleforge/ethereum) | Verify `eth_getProof` account and storage proofs against a state root. |
  | [`@merkleforge/openzeppelin`](https://www.npmjs.com/package/@merkleforge/openzeppelin) | Drop-in `StandardMerkleTree` and `SimpleMerkleTree` for `@openzeppelin/merkle-tree`, with the same roots, proofs, and dumps. |

  The last three have no Rust counterpart. The source lives in
  [1888-Labs/MerkleForge-ts](https://github.com/1888-Labs/MerkleForge-ts).
</TypeScript>

## Rust and TypeScript together

The same trees, hashes, and proof formats are available on npm as
[`@merkleforge/*`](https://www.npmjs.com/org/merkleforge) packages. Roots and proofs match the Rust
crates byte for byte, so a Rust server can prove and a browser can verify. Every guide shows both
languages: pick one with the **Rust / TypeScript** switch in the sidebar. See
[TypeScript packages](/docs/reference/typescript) for the package list and how the APIs map.

:::warning
MerkleForge has **not** completed an independent security audit. Review the code against your
threat model before using it in security-critical systems. See [Security](/docs/reference/security).
:::

## Next steps

* [Getting started](/docs/getting-started): install the <Rust inline>crates</Rust><TypeScript inline>packages</TypeScript> and verify your first proof.
* [Mental model](/docs/mental-model): leaves, roots, proofs, and verifiers in five minutes.
* [Choosing a tree](/docs/guides/choosing-a-tree): which variant fits your data.
