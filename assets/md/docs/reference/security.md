# Security

Audit status, how to report vulnerabilities, and what verifiers must trust, for the Rust crates and
the TypeScript packages.

<LangBar />

## Audit status

:::warning
**MerkleForge has not completed an independent security audit.** This applies to both the Rust
crates and the TypeScript packages.

The <Rust inline>Rust codebase is tested with unit, integration, property-based, benchmark, and
Ethereum test-vector tests</Rust><TypeScript inline>TypeScript codebase is tested with unit,
property-based (fast-check), cross-implementation (vectors generated from the Rust crates), and
Ethereum test-vector tests, plus real mainnet `eth_getProof` responses</TypeScript>, but tests are
not an audit. Review the code, your threat model, and the test
coverage before using MerkleForge in production systems, financial applications, or other
security-critical environments.
:::

## Reporting a vulnerability

Please **do not** report vulnerabilities in public issues.

1. Open the **Security** tab of the
   <Rust inline>[MerkleForge repository](https://github.com/1888-Labs/MerkleForge/security)</Rust><TypeScript inline>[MerkleForge-ts repository](https://github.com/1888-Labs/MerkleForge-ts/security)</TypeScript> and click
   **Report a vulnerability** to use GitHub Security Advisories.
2. Or email the details to [dicethedev@gmail.com](mailto\:dicethedev@gmail.com).

Include as much as you can:

* a clear description of the issue;
* steps to reproduce, ideally with proof-of-concept code;
* affected versions or components;
* potential impact and severity, and whether you believe it is being actively exploited.

Don't disclose publicly until the issue is confirmed and fixed, and don't access, modify, or destroy
data that isn't yours. The maintainers will acknowledge and investigate reports, may contact you
for details, and will coordinate disclosure once a fix is available.

There is no formal bug bounty. With your consent, your contribution may be credited in the
changelog or release notes.

The canonical policy is
<Rust inline>[`.github/SECURITY.md`](https://github.com/1888-Labs/MerkleForge/blob/main/.github/SECURITY.md)</Rust><TypeScript inline>[`.github/SECURITY.md`](https://github.com/1888-Labs/MerkleForge-ts/blob/main/.github/SECURITY.md)
in MerkleForge-ts</TypeScript>.

## Design properties

<Rust>
  * `#![forbid(unsafe_code)]` in every published crate.
  * **Domain separation** between leaves (`0x00`) and internal nodes (`0x01`), or BLAKE3 context
    strings, to prevent second-preimage attacks. See [Mental model](/docs/mental-model#domain-separation-0x00-and-0x01).
  * **Empty leaves rejected** so inserted data can't be confused with padding.
  * **Verifiers never panic** on malformed proofs; they return `false`.
  * **Deserialization rebuilds** internal nodes from leaves and rejects data from another hash
    algorithm, so a decoded tree is always internally consistent.
</Rust>

<TypeScript>
  * **Small dependency surface.** `@merkleforge/core` has no runtime dependencies, and the only
    third-party runtime dependency of the core, hash, and tree packages is
    [`@noble/hashes`](https://github.com/paulmillr/noble-hashes). The optional
    `@merkleforge/hash-wasm` package uses [`hash-wasm`](https://github.com/Daninet/hash-wasm).
  * **Domain separation** between leaves (`0x00`) and internal nodes (`0x01`), or BLAKE3 context
    strings, to prevent second-preimage attacks. See [Mental model](/docs/mental-model#domain-separation-0x00-and-0x01).
  * **Empty leaves rejected** (`MerkleError` with code `EmptyLeafData`) so inserted data can't be
    confused with padding.
  * **Verifiers never throw** on malformed proofs; they return `false`.
  * **Deserialization rebuilds** internal nodes from leaves and rejects data from another hash
    algorithm, so a decoded tree is always internally consistent.
  * **No internal buffers handed out.** Methods return copies of internal `Uint8Array` state.

  **Releases**

  * Releases are published from GitHub Actions through **npm trusted publishing** (OIDC) with
    **provenance**. There is no npm token.
  * Each release is **staged** first and goes live only after a maintainer approves every package
    with **2FA**.
  * The packages are set to require 2FA and to disallow publishing tokens.

  Check the registry signatures and provenance attestations of your installed packages with:

  ```bash
  npm audit signatures
  ```
</TypeScript>

## What integrators must get right

* **Trust the root, not the prover.** A proof shows consistency with a root; it says nothing about
  whether that root is the right one.
* **<Rust inline>`MerkleProof::leaf_count`</Rust><TypeScript inline>`MerkleProof.leafCount`</TypeScript> is not committed** by the binary root. Obtain it independently if it
  matters.
* **Check the key** on sparse proofs with <Rust inline>`verify_membership`. The deprecated
  `SparseMerkleTree::verify` accepts a valid proof for *any* key holding the value.</Rust><TypeScript inline>`SparseMerkleTree.verifyMembership`, which takes the
  key and rejects a proof for a different one.</TypeScript>
* **Canonical leaf encoding.** Prover and verifier must hash exactly the same bytes.
* **Custom hash adapters** must keep leaf and node domains separate. <Rust inline>Since v0.5.0, the default
  `hash_nodes` applies the `0x01` prefix for you.</Rust><TypeScript inline>`defineHashFunction` fills in a
  default `hashNodes` that applies the `0x01` prefix for you.</TypeScript> See
  [Hash functions](/docs/guides/hash-functions#writing-a-custom-adapter).
