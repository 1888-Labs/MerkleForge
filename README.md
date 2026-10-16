<p align="center">
  <a href="https://1888-labs.github.io/MerkleForge/">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="assets/brand/svg/lockup/merkleforge-lockup-white.svg">
      <img alt="MerkleForge" src="assets/brand/svg/lockup/merkleforge-lockup-black.svg" width="420">
    </picture>
  </a>
</p>

<p align="center">
  Authenticated data structures for Rust: Binary, Sparse, and Ethereum-compatible<br>
  Patricia Merkle trees behind one API, with pluggable hashes and stateless proofs.
</p>

<p align="center">
  <a href="https://crates.io/crates/merkle-core"><img alt="crates.io" src="https://img.shields.io/crates/v/merkle-core.svg?style=flat-square&label=crates.io&color=000000"></a>
  <a href="https://docs.rs/merkle-core"><img alt="docs.rs" src="https://img.shields.io/docsrs/merkle-core?style=flat-square&label=docs.rs&color=000000"></a>
  <a href="https://github.com/1888-Labs/MerkleForge/actions/workflows/ci.yml"><img alt="CI" src="https://img.shields.io/github/actions/workflow/status/1888-Labs/MerkleForge/ci.yml?style=flat-square&label=CI"></a>
  <a href="BENCHMARKS.md"><img alt="benchmarks" src="https://img.shields.io/badge/benchmarks-reproducible-000000?style=flat-square"></a>
  <a href="#license"><img alt="license" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-000000?style=flat-square"></a>
</p>

<p align="center">
  <a href="https://crates.io/crates/merkle-core"><img alt="merkle-core downloads" src="https://img.shields.io/crates/d/merkle-core?style=flat-square&label=merkle-core&color=000000"></a>
  <a href="https://crates.io/crates/merkleforge-hash"><img alt="merkleforge-hash downloads" src="https://img.shields.io/crates/d/merkleforge-hash?style=flat-square&label=merkleforge-hash&color=000000"></a>
  <a href="https://crates.io/crates/merkle-variants"><img alt="merkle-variants downloads" src="https://img.shields.io/crates/d/merkle-variants?style=flat-square&label=merkle-variants&color=000000"></a>
</p>

<p align="center">
  <a href="https://1888-labs.github.io/MerkleForge/">Website</a> ·
  <a href="https://1888-labs.github.io/MerkleForge/docs/getting-started">Docs</a> ·
  <a href="https://1888-labs.github.io/MerkleForge/demo/">Live demo</a> ·
  <a href="BENCHMARKS.md">Benchmarks</a> ·
  <a href="CONTRIBUTING.md">Contributing</a>
</p>

---

> ⚠️ **Not yet audited.** MerkleForge is tested with unit, integration,
> property-based, benchmark, and Ethereum vector tests, but has not had an
> independent security audit. Review it against your threat model before using
> it in security-critical systems, and [report issues responsibly](.github/SECURITY.md).

## Why MerkleForge

Every state lookup, transaction batch, and light-client proof depends on the
Merkle tree underneath it. In Rust, that support is fragmented: `rs-merkle`
covers binary trees, Ethereum tries live elsewhere, and each crate has its own
API, test depth, and benchmark method.

MerkleForge puts the three variants used in production blockchains behind one
API:

| Variant | Best for |
|---------|----------|
| `BinaryMerkleTree<H>` | Transaction batching, Bitcoin-style SPV |
| `SparseMerkleTree<H>` | Account state, Layer-2 rollups |
| `MerklePatriciaTrie<H>` | Ethereum state roots, EVM compatibility |

- **One trait.** Every tree implements `MerkleTree<H>`, so you can switch variants without rewriting integration code.
- **Pluggable hashes.** `Sha256`, `Keccak256`, or `Blake3`, chosen by a type parameter at compile time with no runtime cost.
- **Stateless proofs.** Light clients verify inclusion with only a trusted root, the leaf, and a compact `O(log n)` proof.
- **Ethereum-compatible.** Patricia trie state roots are validated against the official Ethereum test vectors.
- **Measured.** Criterion benchmarks against `rs-merkle` and `merkle_light`, published in [`BENCHMARKS.md`](BENCHMARKS.md).
- **Safe.** `#![forbid(unsafe_code)]` across the workspace, with Proptest invariants for every variant.

## Quick start

```bash
cargo add merkle-core merkleforge-hash merkle-variants
```

```rust
use merkle_core::prelude::*;
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

fn main() -> Result<(), MerkleError> {
    let mut tree = BinaryMerkleTree::<Sha256>::new();
    tree.insert(b"alice:100")?;
    tree.insert(b"bob:250")?;
    tree.insert(b"carol:75")?;

    let root = tree.root().expect("tree is non-empty");
    let proof = tree.generate_proof(LeafIndex(0))?;

    // Stateless verification: no tree needed.
    assert!(BinaryMerkleTree::<Sha256>::verify(root, b"alice:100", &proof));
    Ok(())
}
```

Switching to BLAKE3 means changing one type parameter: `BinaryMerkleTree::<Blake3>`.
To see the full server/client split, run
`cargo run -p merkle-variants --example light_client`.

**Next:** [Getting started](https://1888-labs.github.io/MerkleForge/docs/getting-started) ·
[Choosing a tree](https://1888-labs.github.io/MerkleForge/docs/guides/choosing-a-tree) ·
[Live demo](https://1888-labs.github.io/MerkleForge/demo/)

## Project map

| Crate | Published as | What it holds |
|-------|--------------|---------------|
| [`merkle-core/`](merkle-core/) | `merkle-core` | Traits, proof types, errors |
| [`merkle-hash/`](merkle-hash/) | `merkleforge-hash` | SHA-256, Keccak-256, BLAKE3 adapters |
| [`merkle-variants/`](merkle-variants/) | `merkle-variants` | Binary, Sparse, Patricia trees |
| [`merkle-bench/`](merkle-bench/) | not published | Criterion benchmark suite |
| [`docs/`](docs/) | n/a | The website and docs ([Vocs](https://vocs.dev)) |

| Read this | For |
|-----------|-----|
| [ARCHITECTURE.md](ARCHITECTURE.md) | How the crates fit together, hashing, proofs, and where to make a change |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Setup, required checks, and pull request workflow |
| [BENCHMARKS.md](BENCHMARKS.md) | Results and reproduction commands |
| [profiling/](profiling/) | Flamegraphs, `perf`, and energy measurement |
| [REFERENCES.md](REFERENCES.md) | Research the design draws on |
| [assets/brand/](assets/brand/BRAND.md) | Official logos and usage rules |

## Contributing

Contributions are welcome. Good first areas include docs fixes, extra test
vectors, benchmark reproductions on new hardware, and examples. Run the checks
before you open a PR:

```bash
make fmt && make lint && make test
```

For anything that touches the public API or cryptography, open an issue first.
The [contributing guide](CONTRIBUTING.md) has the full workflow, and everyone
is expected to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Projects using MerkleForge

- **[ProofStream](https://github.com/1888-Labs/ProofStream)**: commits live DEX
  activity with `BinaryMerkleTree<Keccak256>` so a browser can verify a single
  record against the dataset root
  ([ETHGlobal showcase](https://ethglobal.com/showcase/proofstream-jvup1)).

Built something with MerkleForge? Open a PR to add it here.

## License

Licensed under either the [MIT License](LICENSE) or the
[Apache License, Version 2.0](LICENSE-APACHE), at your option.

---

*Maintained by [1888 Labs](https://github.com/1888-Labs), led by [Blessing Samuel](https://github.com/dicethedev), and open-source contributors.*
