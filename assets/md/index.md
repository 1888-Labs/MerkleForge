<div className="mf-page">
  <div className="mf-hero">
    <div>
      <div className="mf-hero-owner">
        <OwnerCredit />
      </div>

      <HomePage.Logo className="mf-hero-logo" />

      <div className="mf-hero-tagline">Authenticated data structures for Rust and TypeScript</div>

      <div className="mf-hero-copy">
        Merkle trees with one API and small proofs anyone can check. Rust and TypeScript
        produce byte-identical roots and proofs: prove on a Rust server, verify in the browser.
      </div>

      <div className="mf-hero-buttons">
        <HomePage.Button href="/docs/getting-started" variant="accent">Get started</HomePage.Button>
        <HomePage.Button href="/demo">Live demos</HomePage.Button>
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
          merkle-core = "0.5.2"
          merkleforge-hash = "0.5.2"
          merkle-variants = "0.5.2"
          ```
          :::
        </div>

        <div className="mf-badges">
          <div className="mf-badge"><span>version</span><span>v0.5.2</span></div>
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
    <div className="mf-grid mf-highlights">
      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Three trees</div>
        <div className="mf-feature-copy">Binary, Sparse, and Ethereum-compatible Patricia trees. <Link to="/docs/guides/choosing-a-tree">Choose one</Link></div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Any hash</div>
        <div className="mf-feature-copy">SHA-256, Keccak-256, or BLAKE3, swapped in one place.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Stateless proofs</div>
        <div className="mf-feature-copy">Verify with only the root, the leaf, and the proof.</div>
      </div>

      <div className="mf-card mf-feature">
        <div className="mf-feature-title">Rust and TypeScript</div>
        <div className="mf-feature-copy">Same roots and proofs in both, plus Solidity verifiers.</div>
      </div>
    </div>
  </div>

  <div className="mf-section mf-split">
    <div>
      <div className="mf-eyebrow">Example</div>
      <div className="mf-section-title">Build a root, prove a leaf, verify anywhere</div>

      <div className="mf-feature-copy">
        Insert raw bytes, get a 32-byte root, and hand a light client an <code>O(log n)</code> proof.
        The verifier never sees the rest of the data, and it can run in either language.
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
    <div className="mf-eyebrow">Explore</div>

    <div className="mf-links">
      <Link to="/docs/use-cases">Use cases</Link>
      <Link to="/demo">Live demos</Link>
      <Link to="/examples">Examples</Link>
      <Link to="/benchmarks">Benchmarks</Link>
      <Link to="/roadmap">Roadmap</Link>
      <a href="https://github.com/1888-Labs/ProofStream">Used by ProofStream</a>
    </div>
  </div>

  <div className="mf-section">
    <div className="mf-eyebrow">Downloads</div>

    <CrateDownloads compact />

    <NpmDownloads compact />
  </div>

  <div className="mf-footer">
    <OwnerCredit compact />

    <span>MIT OR Apache-2.0 · Not yet independently audited, see <Link to="/docs/reference/security">Security</Link></span>
    <span><a href="https://crates.io/crates/merkle-variants">crates.io</a> · <a href="https://docs.rs/merkle-core">docs.rs</a> · <a href="https://www.npmjs.com/org/merkleforge">npm</a> · <a href="https://github.com/1888-Labs/MerkleForge">GitHub (Rust)</a> · <a href="https://github.com/1888-Labs/MerkleForge-ts">GitHub (TypeScript)</a></span>
  </div>
</div>
