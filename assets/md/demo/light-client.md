<div className="mf-page" style={{ paddingTop: 0 }}>
  # Light-client demo

  A server commits rows to a Merkle root and sends one row plus its proof to a client. The client
  holds only the trusted root. Edit the rows, pick one to prove, then change the **claimed value** to
  see tampering rejected.

  The demo runs the real TypeScript library in your browser: `BinaryMerkleTree` from
  [`@merkleforge/variants`](https://www.npmjs.com/package/@merkleforge/variants) with the `Sha256`
  adapter from [`@merkleforge/hash`](https://www.npmjs.com/package/@merkleforge/hash). Leaves are
  `SHA-256(0x00 ‖ row)`, internal nodes `SHA-256(0x01 ‖ left ‖ right)`, and the leaf layer is padded
  to a power of two with the empty-leaf hash. Roots and proofs are byte-identical to the Rust
  `BinaryMerkleTree<Sha256>` for the same rows, so a Rust server and this page agree.

  <LightClientDemo />

  ## The same flow in code

  <LangBar always />

  <Rust>
    ```rust [merkle-variants/examples/light_client.rs]
    use merkle_core::{error::MerkleError, types::LeafIndex};
    use merkle_variants::BinaryMerkleTree;
    use merkleforge_hash::Sha256;

    fn main() -> Result<(), MerkleError> {
        let proof;
        let root;

        {
            let mut tree = BinaryMerkleTree::<Sha256>::new();
            tree.insert(b"tx:alice->bob:100")?;
            tree.insert(b"tx:bob->carol:50")?;
            tree.insert(b"tx:carol->dave:25")?;

            root = *tree.root().expect("tree has inserted leaves");
            proof = tree.generate_proof(LeafIndex(0))?;

            println!("server: built tree and exported root + proof");
        }

        let valid = BinaryMerkleTree::<Sha256>::verify(&root, b"tx:alice->bob:100", &proof);

        println!("client: tree dropped before verification");
        println!("Stateless verification: {valid}");

        Ok(())
    }
    ```

    ```bash
    git clone https://github.com/1888-Labs/MerkleForge.git
    cd MerkleForge
    cargo run -p merkle-variants --example light_client
    ```

    ```text
    server: built tree and exported root + proof
    client: tree dropped before verification
    Stateless verification: true
    ```
  </Rust>

  <TypeScript>
    ```ts [index.ts]
    import { type Bytes, type MerkleProof, utf8 } from '@merkleforge/core'
    import { Sha256 } from '@merkleforge/hash'
    import { BinaryMerkleTree } from '@merkleforge/variants'

    function server(): { root: Bytes; proof: MerkleProof } {
      const tree = new BinaryMerkleTree(Sha256)
      tree.insert(utf8('tx:alice->bob:100'))
      tree.insert(utf8('tx:bob->carol:50'))
      tree.insert(utf8('tx:carol->dave:25'))

      console.log('server: built tree and exported root + proof')
      return { root: tree.root()!, proof: tree.generateProof(0) }
    }

    // The tree goes out of scope when `server` returns; only the root and proof leave it.
    const { root, proof } = server()

    const valid = BinaryMerkleTree.verify(Sha256, root, utf8('tx:alice->bob:100'), proof)

    console.log('client: tree dropped before verification')
    console.log(`Stateless verification: ${valid}`)
    ```

    ```bash
    npm i @merkleforge/core @merkleforge/hash @merkleforge/variants
    npx tsx index.ts
    ```

    ```text
    server: built tree and exported root + proof
    client: tree dropped before verification
    Stateless verification: true
    ```
  </TypeScript>

  Learn more in [Proofs & stateless verification](/docs/guides/proofs).
</div>
