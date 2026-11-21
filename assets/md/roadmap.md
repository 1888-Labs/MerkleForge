<div className="mf-page" style={{ paddingTop: 0 }}>
  # Roadmap

  MerkleForge started as fast, correct Merkle trees for blockchains. The same idea, a 32-byte root
  that commits to a large dataset plus small proofs anyone can check, applies anywhere data has to be
  trusted without being re-downloaded: files, logs, databases, machine-learning datasets and models,
  and zero-knowledge circuits. This page is where we are taking MerkleForge across **CPU**, **GPU**,
  **ZK**, and **AI** systems.

  Plans are grouped as **Now** (in review or being built), **Next** (designed, starting soon), and
  **Later** (direction, open to discussion). Nothing below is a promise of a date. If something
  matters to you, say so in [Rust issues](https://github.com/1888-Labs/MerkleForge/issues) or
  [TypeScript issues](https://github.com/1888-Labs/MerkleForge-ts/issues), which is how items move up.

  Two rules hold for everything here: roots and proofs stay byte-identical across Rust, TypeScript,
  and Solidity, and a faster backend never changes an output. Every new backend is tested against
  the reference implementation.

  ## Shipped

  <div className="mf-grid">
    <div className="mf-card mf-feature">
      <div className="mf-feature-title">Three trees, one API</div>
      <div className="mf-feature-copy">Binary, Sparse, and Ethereum-compatible Patricia trees in Rust (<code>merkle-variants</code>) and TypeScript (<code>@merkleforge/variants</code>), with multiproofs and byte-identical proof codecs.</div>
    </div>

    <div className="mf-card mf-feature">
      <div className="mf-feature-title">Ethereum and the EVM</div>
      <div className="mf-feature-copy"><code>eth_getProof</code> verification, an OpenZeppelin drop-in, and Solidity verifiers for binary, sparse, Patricia, and Ethereum proofs.</div>
    </div>

    <div className="mf-card mf-feature">
      <div className="mf-feature-title">Pluggable hashes</div>
      <div className="mf-feature-copy">SHA-256, Keccak-256, and BLAKE3 in both languages, plus WebAssembly Keccak-256 and SHA-256 for JavaScript.</div>
    </div>

    <div className="mf-card mf-feature">
      <div className="mf-feature-title">Trusted publishing</div>
      <div className="mf-feature-copy">npm packages ship with provenance through npm trusted publishing; Rust vectors pin the TypeScript port in CI.</div>
    </div>
  </div>

  ## CPU and WebAssembly

  The fastest hash call is the one you batch. Tree builds are a sequence of independent node hashes
  per layer, which is ideal for SIMD and for multiple cores.

  | Status | Item |
  | --- | --- |
  | **Now** | **Batch hashing API** in TypeScript ([in review](https://github.com/1888-Labs/MerkleForge-ts/pull/29)): hash adapters can hash a whole tree layer in one call (`hashNodePairs`, `hashLeaves`). `BinaryMerkleTree` stores nodes in one flat buffer and skips hashing padding subtrees. |
  | **Now** | **New WebAssembly backend** for `@merkleforge/hash-wasm`: a 9 KB `no_std` Rust module with SIMD Keccak-256 (two permutations at once). Keccak-256 tree builds are about 5x faster than in 0.2.2 (485 ms to 98 ms at 131,072 leaves), and pure-JS tree builds about 3x faster, with byte-for-byte identical output ([benchmarks and parity](/docs/performance/benchmarks)). |
  | **Now** | **CI on every runtime**: the full test suite in Chromium, Firefox, and WebKit, and the built packages on Node.js, Deno, and Bun. |
  | Next | One-call proof verification in WebAssembly (fold the whole path in one call), batched multiproofs, and batch hashing for `SparseMerkleTree` updates. |
  | Next | BLAKE3 in WebAssembly, and SHA-256 SIMD (multi-buffer). |
  | Next | Rust: multi-buffer SIMD hashing (AVX2, AVX-512, NEON) behind the same `HashFunction` trait, so a layer of nodes is hashed 4 to 8 at a time. |
  | Later | Parallel tree builds: `rayon` in Rust and Web Workers in JavaScript for trees with millions of leaves. |
  | Later | Embedded and `no_std` targets for `merkle-variants` (RISC-V, Cortex-M) for device attestation and firmware updates. |

  ## GPU

  Large trees (10⁸ leaves and more: rollup state, data availability, dataset commitments) spend
  nearly all their time hashing. The batch API above is shaped for GPUs on purpose: one layer is one
  dispatch of independent pair hashes.

  | Status | Item |
  | --- | --- |
  | Next | **WebGPU backend** for the browser and Deno: a `HashFunction` whose `hashNodePairs` runs Keccak-256, SHA-256, and Poseidon2 compute shaders. Same digests, tested against the CPU adapters. |
  | Later | **Native GPU backend** in Rust through `wgpu` (Vulkan, Metal, DirectX 12), with CUDA where it pays off, for building and updating very large trees. |
  | Later | Published crossover measurements: at what tree size the GPU beats the CPU on common hardware, so the library can choose for you. The CPU stays the default. |

  ## Zero-knowledge proofs

  Zero-knowledge systems prove statements about Merkle trees inside a circuit, where Keccak-256 and
  SHA-256 are expensive. They need SNARK-friendly hashes and proof formats that circuits can consume.

  | Status | Item |
  | --- | --- |
  | Next | **Poseidon2 and Poseidon adapters** over BN254 (and other fields), with parameters that match circomlib, Noir, and gnark, so roots agree with existing circuits. |
  | Next | **Incremental (append-only) Merkle trees** for commitments and nullifiers, the shape used by Semaphore-style anonymous signalling and privacy pools. |
  | Later | **Circuit verifiers generated from the same proof format**: Noir and Circom templates that verify binary and sparse MerkleForge proofs, alongside the existing Solidity verifiers. |
  | Later | Interop with proving systems (Halo2, Plonky3, SP1 and RISC Zero guests) so the Rust crates can run inside zkVMs and their proofs verify on-chain. |

  ## AI and data systems

  Machine-learning pipelines move huge datasets and model files between parties that need to trust
  them. A Merkle root is a compact, verifiable fingerprint of any of these, and a proof shows that
  one piece belongs without sharing the rest.

  | Status | Item |
  | --- | --- |
  | Next | **Dataset commitments**: commit to a training or evaluation set (one leaf per example or shard), then prove a specific example was or was not included, for audits, licensing, and contamination checks. |
  | Next | **Model and checkpoint integrity**: a root over fixed-size chunks of model weights, so downloads verify chunk by chunk and a corrupted or swapped shard is caught before loading. Try it with the [file integrity demo](/demo/file-integrity). |
  | Later | **Verifiable AI agent and inference logs**: an append-only log of prompts, tool calls, and outputs with consistency proofs, so anyone can check the log was never rewritten. |
  | Later | **Retrieval provenance (RAG)**: prove that a retrieved passage comes from a committed corpus version. |
  | Later | Python bindings (PyO3) so data and ML tooling can use the Rust crates directly. |

  ## Other systems beyond blockchain

  | Status | Item |
  | --- | --- |
  | Next | **Transparency logs**: an append-only tree with RFC 9162 (Certificate Transparency) inclusion and consistency proofs. |
  | Next | **File sync and content-addressed storage**: chunked file verification and fast diffing between two replicas by comparing subtree roots. |
  | Later | Authenticated key-value storage on disk for databases that need verifiable reads and audits. |
  | Later | Supply-chain attestations: commit to SBOMs and build outputs and prove a component was part of a release. |
  | Later | More language bindings (Go, Swift, Kotlin) generated from the Rust crates. |

  ## Docs and examples

  | Status | Item |
  | --- | --- |
  | **Now** | A [demo section](/demo) that runs the TypeScript packages in your browser: light client, in-browser benchmarks, airdrop claims, file integrity, revocation lists, and Ethereum state proofs. |
  | **Now** | A [use cases](/docs/use-cases) guide that maps real systems to trees, hashes, and packages. |
  | Next | End-to-end examples per use case, from building a tree to verifying on-chain or in a client. |

  MerkleForge has not completed an independent security audit. See the
  [security notes](/docs/reference/security) before using it in production.
</div>
