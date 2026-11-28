# Use cases

Every use case below has the same shape: someone commits to a lot of data with one 32-byte
**root**, publishes or signs that root, and later hands out small **proofs** that a specific piece
is (or is not) part of it. Whoever checks the proof needs only the root, the piece, and the proof,
not the rest of the data.

Each section names the tree, hash, and packages to use. Items marked **planned** are on the
[roadmap](/roadmap) and not in the libraries yet.

## Blockchain and Ethereum

### Airdrops, allowlists, and claims

Commit a list of `(address, amount)` pairs on-chain as one root, and let each user claim with a
proof. Nobody stores the full list on-chain, and gas stays constant as the list grows.

* **Tree:** `StandardMerkleTree` from `@merkleforge/openzeppelin` (same roots and proofs as
  `@openzeppelin/merkle-tree`), or `BinaryMerkleTree` with `Keccak256` and the MerkleForge
  Solidity verifier.
* **Verify on-chain:** OpenZeppelin's `MerkleProof.verify`, or the MerkleForge verifiers in
  [`contracts/`](https://github.com/1888-Labs/MerkleForge-ts/tree/main/contracts).
* **Try it:** [airdrop claim demo](/demo/airdrop).

### Light clients and Ethereum state proofs

Check an account balance, nonce, or contract storage slot against a trusted block's state root
instead of trusting an RPC provider's answer.

* **Tree:** `MerklePatriciaTrie` with `Keccak256` (matches Ethereum roots).
* **Packages:** `@merkleforge/ethereum` (`verifyEthGetProof`) in TypeScript;
  `MerkleForgeEthereum` on-chain.
* **Try it:** [Ethereum state proof demo](/demo/ethereum).

### Rollup and app-chain state

Keep account or application state as a key-value map, prove reads, and prove a key is absent.

* **Tree:** `SparseMerkleTree` (membership and non-membership proofs over 32-byte keys).
* **Hash:** `Sha256` or `Blake3` off-chain, `Keccak256` when proofs are checked by Solidity.
  SNARK-friendly hashes such as Poseidon2 are **planned** for proofs checked inside circuits.

### Transaction batches, receipts, and bridges

Commit a batch of transactions or messages, post the root, and prove inclusion of any one of them
on the other side, one at a time or with a [multiproof](/docs/guides/proofs).

* **Tree:** `BinaryMerkleTree`, `generateMultiProof` for many items at once.

## Beyond blockchain

### File integrity and chunked downloads

Split a file into fixed-size chunks and commit to them. A downloader that trusts only the root can
verify every chunk as it arrives, from any mirror or peer, and pinpoint a corrupted chunk instead of
re-downloading the whole file. This is how BitTorrent v2 and content-addressed storage work.

* **Tree:** `BinaryMerkleTree`, one leaf per chunk, `Sha256` or `Blake3`.
* **Try it:** [file integrity demo](/demo/file-integrity), which runs on a file you pick without uploading it.

### Revocation lists and allowlists with "not in the list" proofs

Publish a root over revoked certificate or credential IDs. A verifier can check a membership proof
(revoked) or a non-membership proof (still valid) without downloading the whole list.

* **Tree:** `SparseMerkleTree`, keyed by a hash of the ID.
* **Try it:** [revocation list demo](/demo/revocation).

### Transparency and audit logs

Append events to a log and publish its root, so anyone can check an entry is in the log. Certificate
Transparency uses this to catch mis-issued TLS certificates.

* **Today:** `BinaryMerkleTree` gives inclusion proofs for a log you rebuild or append to.
* **Planned:** RFC 9162 append-only trees with consistency proofs (proof that a newer log extends
  an older one without rewriting it).

### Databases and replicas

Compare two replicas of a dataset by comparing roots, then walk down only the subtrees that differ.
Serve reads with a proof so clients can check them against a root they trust.

* **Tree:** `SparseMerkleTree` for key-value data, `BinaryMerkleTree` for ordered data.
* **Planned:** replica diffing helpers and on-disk authenticated storage.

## AI and data

### Dataset commitments

Publish a root over a training or evaluation set, one leaf per example or shard. Later, prove a
given example was in the set (for licensing and attribution), or with a sparse tree prove it was
not (for test-set contamination checks), without releasing the dataset.

* **Tree:** `BinaryMerkleTree` for an ordered list of shards, `SparseMerkleTree` keyed by a hash of
  each example when you need non-membership proofs.
* **Planned:** dataset tooling and Python bindings.

### Model weights and checkpoints

Commit to a model file in fixed-size chunks. Mirrors and caches can serve the chunks, and a loader
verifies each one against the published root before using it, so a corrupted or swapped shard is
caught early.

* **Tree:** `BinaryMerkleTree` with `Sha256` or `Blake3`, exactly as in the
  [file integrity demo](/demo/file-integrity).
* **Planned:** GPU-accelerated builds for very large checkpoints.

### Agent and inference logs

Record prompts, tool calls, and outputs of an AI system in an append-only log and publish its root,
so the operator cannot quietly rewrite history.

* **Today:** `BinaryMerkleTree` inclusion proofs over log entries.
* **Planned:** append-only trees with consistency proofs.

## Zero-knowledge

### Anonymous membership and private claims

Prove "I am in this group" or "I own a note in this pool" inside a zero-knowledge proof, without
revealing which member. These systems use Merkle trees with SNARK-friendly hashes.

* **Planned:** Poseidon2 adapters, incremental append-only trees, and Noir and Circom verifiers
  generated from MerkleForge proof formats. See the [roadmap](/roadmap).

## Picking a tree

If you are unsure which tree fits, start with [Choosing a tree](/docs/guides/choosing-a-tree). In
short: an ordered list proved by position is a `BinaryMerkleTree`; a key-value map that needs "not
present" proofs is a `SparseMerkleTree`; anything that must match Ethereum roots is a
`MerklePatriciaTrie` with `Keccak256`.
