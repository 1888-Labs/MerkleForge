<div className="mf-page" style={{ paddingTop: 0 }}>
  # Airdrop claim

  Token airdrops, allowlists, and vesting contracts store one 32-byte Merkle root on-chain instead
  of every recipient. Each recipient later sends their allocation and a proof; the contract hashes
  the claim up to the root and pays out only if it matches.

  This demo uses `StandardMerkleTree` from
  [`@merkleforge/openzeppelin`](https://www.npmjs.com/package/@merkleforge/openzeppelin), a drop-in
  for `@openzeppelin/merkle-tree` with identical roots, proofs, and `dump()` JSON. Edit the list,
  pick a recipient, then try to claim more than the allocation.

  <AirdropDemo />

  ## Build the tree

  TypeScript only: `@merkleforge/openzeppelin` has no Rust crate yet.

  ```ts [airdrop.ts]
  import { StandardMerkleTree } from '@merkleforge/openzeppelin'

  const values = [
    ['0x1111111111111111111111111111111111111111', '5000000000000000000'],
    ['0x2222222222222222222222222222222222222222', '2500000000000000000'],
  ]
  const tree = StandardMerkleTree.of(values, ['address', 'uint256'])

  console.log(tree.root) // store this in the contract
  const proof = tree.getProof(0) // send this with the claim
  console.log(StandardMerkleTree.verify(tree.root, ['address', 'uint256'], values[0], proof))
  ```

  ```text
  0xd4dee0beab2d53f2cc83e567171bd2820e49898130a22622b10ead383e90bd77
  true
  ```

  ## Verify the claim on-chain

  ```solidity [Airdrop.sol]
  import {MerkleProof} from "@openzeppelin/contracts/utils/cryptography/MerkleProof.sol";

  function claim(bytes32[] calldata proof, uint256 amount) external {
      bytes32 leaf = keccak256(bytes.concat(keccak256(abi.encode(msg.sender, amount))));
      require(MerkleProof.verify(proof, root, leaf), "invalid proof");
      // mark claimed, then transfer `amount`
  }
  ```

  The double hash in the leaf stops a 64-byte internal node from being passed off as a leaf.
</div>
