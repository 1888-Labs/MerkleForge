<div className="mf-page">
  <div className="mf-hero">
    <div>
      <div className="mf-hero-owner">
        <OwnerCredit />
      </div>

      <HomePage.Logo className="mf-hero-logo" />

      <div className="mf-hero-tagline">Authenticated data structures for Rust and TypeScript</div>

      <div className="mf-hero-copy">
        Binary, Sparse, and Ethereum-compatible Patricia Merkle trees behind one API,
        with pluggable hash functions and stateless proof verification. Rust crates and
        TypeScript packages produce byte-identical roots and proofs: prove on a Rust server,
        verify in the browser.
      </div>

      <div className="mf-hero-buttons">
        <HomePage.Button href="/docs/getting-started" variant="accent">Get started</HomePage.Button>
        <HomePage.Button href="/docs">Why MerkleForge?</HomePage.Button>
        <HomePage.Button href="https://github.com/1888-Labs/MerkleForge">Rust on GitHub</HomePage.Button>
        <HomePage.Button href="https://github.com/1888-Labs/MerkleForge-ts">TypeScript on GitHub</HomePage.Button>
      </div>
    </div>

    <div>
      <LangBar always />

      <Rust>
        <div className="mf-install">
          :::code-group
          ```bash [cargo]
          cargo add merkle-core merkleforge-hash merkle-variants
          ```

          ```toml [Cargo.toml]
          [dependencies]
          merkle-core = "0.5.1"
          merkleforge-hash = "0.5.1"
          merkle-variants = "0.5.1"
          ```
          :::
        </div>

        <div className="mf-badges">
          <div className="mf-badge"><span>version</span><span>v0.5.1</span></div>
          <div className="mf-badge"><span>unsafe</span><span>forbidden</span></div>
          <div className="mf-badge"><span>license</span><span>MIT/Apache</span></div>
        </div>
      </Rust>

      <TypeScript>
        <div className="mf-install">
          :::code-group
          ```bash [npm]
          npm i @merkleforge/core @merkleforge/hash @merkleforge/variants
          ```

          ```bash [pnpm]
          pnpm add @merkleforge/core @merkleforge/hash @merkleforge/variants
          ```

          ```bash [yarn]
          yarn add @merkleforge/core @merkleforge/hash @merkleforge/variants
          ```

          ```bash [bun]
          bun add @merkleforge/core @merkleforge/hash @merkleforge/variants
          ```
          :::
        </div>

        <div className="mf-badges">
          <div className="mf-badge"><span>npm</span><span>@merkleforge</span></div>
          <div className="mf-badge"><span>module</span><span>ESM</span></div>
          <div className="mf-badge"><span>provenance</span><span>signed</span></div>
        </div>
      </TypeScript>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-grid">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">One API, three trees</div>
        <div className="mf-feature-copy">Binary, Sparse, and Patricia trees share one <code>MerkleTree</code> interface, in both languages. <Link to="/docs/guides/choosing-a-tree">Choose a tree</Link></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Pluggable hashes</div>
        <div className="mf-feature-copy">Swap SHA-256, Keccak-256, and BLAKE3 without touching the rest of your code. In Rust it is one type parameter that compiles away.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Stateless proofs</div>
        <div className="mf-feature-copy">Verify a leaf with only the trusted root, the proof, and the leaf bytes. No tree required.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Ethereum-compatible</div>
        <div className="mf-feature-copy">The Patricia trie with Keccak-256 matches Ethereum state roots and passes the official trie test vectors.</div>
      </div>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">Rust and TypeScript</div>
    <div className="mf-section-title">One proof format, two languages</div>

    <div className="mf-grid" data-cols="2">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Rust crates</div>
        <div className="mf-feature-copy"><code>merkle-core</code>, <code>merkleforge-hash</code>, and <code>merkle-variants</code> on crates.io. The reference implementation: zero <code>unsafe</code>, <code>no_std</code> support in <code>merkle-core</code>, and Criterion benchmarks. <a href="https://crates.io/crates/merkle-variants">crates.io</a> · <a href="https://docs.rs/merkle-core">docs.rs</a></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">TypeScript packages</div>
        <div className="mf-feature-copy"><code>@merkleforge/core</code>, <code>hash</code>, and <code>variants</code> on npm, with an idiomatic API for Node.js, browsers, Deno, and Bun. Published with provenance through npm trusted publishing. <Link to="/docs/reference/typescript">Packages and API mapping</Link></div>
      </div>
    </div>

    <div className="mf-feature-copy mf-section-note">
      Roots, proofs, and the bincode and JSON proof encodings are byte-identical between the two,
      checked in CI against vectors generated by the Rust crates. Every guide shows both: use the
      Rust / TypeScript switch.
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">TypeScript extras</div>
    <div className="mf-section-title">For the browser and the EVM</div>

    <div className="mf-grid">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">WebAssembly hashes</div>
        <div className="mf-feature-copy"><code>@merkleforge/hash-wasm</code>: Keccak-256 and SHA-256 with identical output, about 6x faster Keccak-256 than pure JavaScript.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">OpenZeppelin drop-in</div>
        <div className="mf-feature-copy"><code>@merkleforge/openzeppelin</code>: <code>StandardMerkleTree</code> and <code>SimpleMerkleTree</code> with the same roots, proofs, and dumps as <code>@openzeppelin/merkle-tree</code>.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">eth_getProof</div>
        <div className="mf-feature-copy"><code>@merkleforge/ethereum</code>: verify Ethereum account and storage proofs against a trusted state root.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Solidity verifiers</div>
        <div className="mf-feature-copy">On-chain verifiers for binary, sparse, Patricia, and Ethereum proofs, with helpers that format proofs for them. <a href="https://github.com/1888-Labs/MerkleForge-ts/tree/main/contracts">contracts/</a></div>
      </div>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">Downloads</div>
    <div className="mf-section-title">Live from crates.io and npm</div>
    <div className="mf-feature-copy">Downloads in the last 7 days show ongoing use; all-time totals also count the registry mirrors that fetch every new release.</div>
    <div className="mf-subhead">Rust crates</div>

    <CrateDownloads />

    <div className="mf-subhead">TypeScript packages</div>

    <NpmDownloads />
  </div>

  <div className="mf-section mf-split">
    <div>
      <div className="mf-eyebrow">Example</div>
      <div className="mf-section-title">Build a root, prove a leaf, verify anywhere</div>

      <div className="mf-feature-copy">
        Insert raw bytes, get a 32-byte root, and hand a light client an <code>O(log n)</code> proof.
        The verifier never sees the rest of the data, and it can run in either language.
      </div>

      <div className="mf-hero-buttons">
        <HomePage.Button href="/examples">More examples</HomePage.Button>
        <HomePage.Button href="/demo">Try the live demo</HomePage.Button>
      </div>
    </div>

    <div>
      <Rust>
        ```rust [src/main.rs]
        use merkle_core::prelude::*;
        use merkle_variants::BinaryMerkleTree;
        use merkleforge_hash::Sha256;

        fn main() -> Result<(), MerkleError> {
            let tree = BinaryMerkleTree::<Sha256>::from_leaves([
                "alice:100", "bob:250", "carol:75",
            ])?;

            let root = tree.root().expect("tree is non-empty");
            let proof = tree.generate_proof(LeafIndex(1))?;

            // Stateless: only the root, the leaf bytes, and the proof.
            assert!(BinaryMerkleTree::<Sha256>::verify(root, b"bob:250", &proof));
            Ok(())
        }
        ```
      </Rust>

      <TypeScript>
        ```ts [index.ts]
        import { utf8 } from '@merkleforge/core'
        import { Sha256 } from '@merkleforge/hash'
        import { BinaryMerkleTree } from '@merkleforge/variants'

        const tree = BinaryMerkleTree.fromLeaves(Sha256, ['alice:100', 'bob:250', 'carol:75'].map(utf8))

        const root = tree.root()!
        const proof = tree.generateProof(1)

        // Stateless: only the root, the leaf bytes, and the proof.
        console.assert(BinaryMerkleTree.verify(Sha256, root, utf8('bob:250'), proof))
        ```
      </TypeScript>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">Built for verifiable systems</div>
    <div className="mf-section-title">Pick the structure that fits your data</div>

    <div className="mf-grid" data-cols="3">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">BinaryMerkleTree</div>
        <div className="mf-feature-copy">Ordered lists, transaction batches, file chunks, and logs. Compact inclusion proofs by leaf index. <Link to="/docs/guides/binary-merkle-tree">Guide</Link></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">SparseMerkleTree</div>
        <div className="mf-feature-copy">A 2<sup>256</sup> key space for account state and rollups, with membership and non-membership proofs. <Link to="/docs/guides/sparse-merkle-tree">Guide</Link></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">MerklePatriciaTrie</div>
        <div className="mf-feature-copy">Ethereum-style nibble paths, RLP-encoded nodes, and EIP-1186-style witnesses. <Link to="/docs/guides/merkle-patricia-trie">Guide</Link></div>
      </div>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">Used by</div>

    <div className="mf-grid" data-cols="2">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">ProofStream</div>
        <div className="mf-feature-copy">Turns live indexed DEX activity into portable proof receipts. Rows are committed with a MerkleForge Keccak-256 binary tree and verified in the browser from a single row, its proof, and the dataset root.</div>
        <div className="mf-feature-copy"><a href="https://github.com/1888-Labs/ProofStream">Repository</a> · <a href="https://1888-labs.github.io/ProofStream/">Live app</a> · <a href="https://ethglobal.com/showcase/proofstream-jvup1">ETHGlobal showcase</a></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Your project?</div>
        <div className="mf-feature-copy">Building with MerkleForge? Open a pull request to add it to the <Link to="/docs/community/projects">projects page</Link>.</div>
      </div>
    </div>
  </div>

  <div className="mf-footer">
    <OwnerCredit compact />

    <span>MIT OR Apache-2.0 · Not yet independently audited, see <Link to="/docs/reference/security">Security</Link></span>
    <span><a href="https://crates.io/crates/merkle-variants">crates.io</a> · <a href="https://docs.rs/merkle-core">docs.rs</a> · <a href="https://www.npmjs.com/org/merkleforge">npm</a> · <a href="https://github.com/1888-Labs/MerkleForge">GitHub (Rust)</a> · <a href="https://github.com/1888-Labs/MerkleForge-ts">GitHub (TypeScript)</a></span>
  </div>
</div>
