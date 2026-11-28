<div className="mf-page" style={{ paddingTop: 0 }}>
  # In-browser benchmarks

  Build a `BinaryMerkleTree` from 32-byte leaves on your own device, with each hash adapter, then
  generate and verify proofs for 500 of the leaves. Most published Merkle tree numbers come from a
  server. This page shows what your users' browsers will see, phones included.

  * **Keccak-256** and **SHA-256** come in two forms with identical output: pure JS from
    [`@merkleforge/hash`](https://www.npmjs.com/package/@merkleforge/hash) and WebAssembly from
    [`@merkleforge/hash-wasm`](https://www.npmjs.com/package/@merkleforge/hash-wasm). Trees,
    proofs, and JSON built with either are interchangeable.
  * **BLAKE3** is pure JS.

  <BrowserBenchmark />

  ## Use the faster adapter in your code

  ```ts [index.ts]
  import { createKeccak256Wasm } from '@merkleforge/hash-wasm'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const Keccak256 = await createKeccak256Wasm() // loads the WebAssembly module once
  const leaves = Array.from({ length: 16_384 }, (_, i) => new TextEncoder().encode(`leaf-${i}`))
  const tree = BinaryMerkleTree.fromLeaves(Keccak256, leaves)
  console.log(tree.leafCount, tree.generateProof(42).path.length)
  ```

  ```text
  16384 14
  ```

  The numbers depend on the `@merkleforge/*` version this site was built with. Server-side
  comparisons with other JavaScript libraries are on the [benchmarks page](/benchmarks).
</div>
