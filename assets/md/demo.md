<div className="mf-page" style={{ paddingTop: 0 }}>
  # Demos

  Each demo models a real system and runs the published
  [`@merkleforge/*`](https://www.npmjs.com/org/merkleforge) TypeScript packages in your browser.
  Nothing you type or pick is uploaded; the only network request is the optional live fetch on the
  Ethereum page, and only when you click it. Roots and proofs match the Rust crates byte for byte.

  ## Performance

  <div className="mf-grid" data-cols="2">
    <Link className="mf-card mf-demo-card" to="/demo/benchmarks">
      <div className="mf-eyebrow">BinaryMerkleTree · 5 hash adapters</div>
      <div className="mf-feature-title">In-browser benchmarks</div>

      <div className="mf-feature-copy">
        Build trees of up to 131,072 leaves on your own device with Keccak-256, SHA-256 (pure JS and
        WebAssembly), and BLAKE3, and compare build, proof, and verify times.
      </div>
    </Link>
  </div>

  ## Blockchain

  <div className="mf-grid" data-cols="2">
    <Link className="mf-card mf-demo-card" to="/demo/airdrop">
      <div className="mf-eyebrow">@merkleforge/openzeppelin · Solidity</div>
      <div className="mf-feature-title">Airdrop claim</div>

      <div className="mf-feature-copy">
        Publish a token allocation as one root, generate a claim proof that OpenZeppelin's
        <code>MerkleProof</code> accepts on-chain, and watch an inflated claim fail.
      </div>
    </Link>

    <Link className="mf-card mf-demo-card" to="/demo/ethereum">
      <div className="mf-eyebrow">@merkleforge/ethereum · Patricia trie</div>
      <div className="mf-feature-title">Ethereum state proof</div>

      <div className="mf-feature-copy">
        Verify a real mainnet <code>eth_getProof</code> response for WETH against a block's state
        root, catch a lying RPC, or fetch a live proof.
      </div>
    </Link>
  </div>

  ## Beyond blockchain

  <div className="mf-grid" data-cols="2">
    <Link className="mf-card mf-demo-card" to="/demo/file-integrity">
      <div className="mf-eyebrow">BinaryMerkleTree · SHA-256 WASM</div>
      <div className="mf-feature-title">File integrity</div>

      <div className="mf-feature-copy">
        Commit to a file in 64 KiB chunks, as BitTorrent v2 and IPFS do, then verify each chunk on
        its own and pinpoint a corrupted one.
      </div>
    </Link>

    <Link className="mf-card mf-demo-card" to="/demo/revocation">
      <div className="mf-eyebrow">SparseMerkleTree · Keccak-256</div>
      <div className="mf-feature-title">Revocation list</div>

      <div className="mf-feature-copy">
        Keep revoked certificates, DIDs, or API keys in a sparse Merkle tree and prove that an ID
        is revoked, or that it is not.
      </div>
    </Link>

    <Link className="mf-card mf-demo-card" to="/demo/light-client">
      <div className="mf-eyebrow">BinaryMerkleTree · SHA-256</div>
      <div className="mf-feature-title">Light client</div>

      <div className="mf-feature-copy">
        A server commits rows to a root and sends one row with its proof; a client that holds only
        the root accepts it or detects tampering.
      </div>
    </Link>
  </div>
</div>
