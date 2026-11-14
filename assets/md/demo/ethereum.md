<div className="mf-page" style={{ paddingTop: 0 }}>
  # Ethereum state proof

  Wallets, bridges, and light clients read Ethereum state from RPC providers they do not control.
  `eth_getProof` ([EIP-1186](https://eips.ethereum.org/EIPS/eip-1186)) returns an account and its
  storage together with Merkle Patricia trie proofs, so a client that trusts only a block's state
  root can check every value itself.

  The demo verifies a real response for the WETH contract with `verifyEthGetProof` from
  [`@merkleforge/ethereum`](https://www.npmjs.com/package/@merkleforge/ethereum): the account in the
  state trie, then storage slots 0–2 (`name`, `symbol`, `decimals`) in WETH's storage trie. Pick a
  dishonest response to see it rejected, or fetch a live proof.

  <EthProofDemo />

  ## In code

  TypeScript only: `@merkleforge/ethereum` has no Rust crate yet. The Rust `MerklePatriciaTrie`
  produces the same roots.

  ```ts [proof.ts]
  import { verifyEthGetProof } from '@merkleforge/ethereum'

  const rpc = async (method: string, params: unknown[]) =>
    (
      await (
        await fetch('https://ethereum-rpc.publicnode.com', {
          method: 'POST',
          headers: { 'content-type': 'application/json' },
          body: JSON.stringify({ jsonrpc: '2.0', id: 1, method, params }),
        })
      ).json()
    ).result

  const block = await rpc('eth_getBlockByNumber', ['latest', false])
  const weth = '0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2'
  const response = await rpc('eth_getProof', [weth, ['0x2'], block.number])

  // Throws if any proof or field does not match the state root.
  const { account, storage } = verifyEthGetProof(block.stateRoot, response)
  console.log(account?.nonce, storage[0]?.value)
  ```

  ```text
  1n 18n
  ```

  Here the state root comes from the same RPC; a light client takes it from a block header it has
  verified, for example through the beacon chain's sync committee.
</div>
