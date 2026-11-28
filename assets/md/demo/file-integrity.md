<div className="mf-page" style={{ paddingTop: 0 }}>
  # File integrity

  Peer-to-peer and content-addressed systems (BitTorrent v2, IPFS, software update and backup
  tools) split a file into chunks and publish the Merkle root of the chunk hashes. A downloader
  that knows only the root can fetch chunks from untrusted peers in any order, check each one with
  a short proof the moment it arrives, and refetch just the chunk that fails, without waiting for
  the whole file.

  This demo splits a file into 64 KiB chunks and builds a `BinaryMerkleTree` with SHA-256 from
  [`@merkleforge/hash-wasm`](https://www.npmjs.com/package/@merkleforge/hash-wasm) (it falls back
  to the pure-JS adapter if WebAssembly is unavailable). Your file is read locally and never
  uploaded.

  <FileIntegrityDemo />

  ## In code

  ```ts [chunks.ts]
  import { createSha256Wasm } from '@merkleforge/hash-wasm'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const CHUNK = 64 * 1024
  const file = new Uint8Array(1_000_000).map((_, i) => i % 251)
  const chunks = []
  for (let at = 0; at < file.length; at += CHUNK) chunks.push(file.subarray(at, at + CHUNK))

  const Sha256 = await createSha256Wasm()
  const tree = BinaryMerkleTree.fromLeaves(Sha256, chunks)
  const root = tree.root()! // publish this

  // Downloader: check chunk 3 from an untrusted peer.
  const ok = BinaryMerkleTree.verify(Sha256, root, chunks[3]!, tree.generateProof(3))
  console.log(chunks.length, ok)
  ```

  ```text
  16 true
  ```

  Proofs for a file of `n` chunks hold `log2(n)` hashes: 14 hashes (448 bytes) for a 1 GiB file.
</div>
