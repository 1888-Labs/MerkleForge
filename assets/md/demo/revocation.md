<div className="mf-page" style={{ paddingTop: 0 }}>
  # Revocation list

  Certificate authorities, verifiable-credential issuers, and API gateways need to answer "has this
  been revoked?" for parties that should not download the whole list. A sparse Merkle tree puts
  every possible ID at a fixed position (here `keccak256(id)`), so the issuer can publish one root
  and hand out a short proof for either answer:

  * **membership**: the ID is in the tree, so it is revoked;
  * **non-membership**: the ID's slot is empty, so it is not revoked.

  The demo runs `SparseMerkleTree` from
  [`@merkleforge/variants`](https://www.npmjs.com/package/@merkleforge/variants) with Keccak-256.
  Revoke and reinstate IDs, then check one; the proof is verified against the root alone.

  <RevocationDemo />

  ## In code

  ```ts [revocation.ts]
  import { utf8 } from '@merkleforge/core'
  import { keccak256 } from '@merkleforge/ethereum'
  import { Keccak256 } from '@merkleforge/hash'
  import { SparseMerkleTree } from '@merkleforge/variants'

  const REVOKED = utf8('revoked')
  const key = (id: string) => keccak256(utf8(id))

  const list = new SparseMerkleTree(Keccak256)
  list.insert(key('cert-2041'), REVOKED)
  const root = list.root()! // publish this

  const yes = list.generateMembershipProof(key('cert-2041'))
  const no = list.generateMembershipProof(key('cert-1000')) // leaf: null, a non-membership proof
  console.log(SparseMerkleTree.verifyMembership(Keccak256, root, key('cert-2041'), REVOKED, yes))
  console.log(SparseMerkleTree.verifyNonMembership(Keccak256, root, key('cert-1000'), no))
  ```

  ```text
  true
  true
  ```

  The same tree type backs account state in rollups and nullifier sets in privacy protocols. See the
  [sparse Merkle tree guide](/docs/guides/sparse-merkle-tree).
</div>
