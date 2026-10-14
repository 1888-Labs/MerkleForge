<p align="center">
  <a href="https://github.com/1888-Labs/MerkleForge">
    <picture>
      <source media="(prefers-color-scheme: dark)" srcset="https://raw.githubusercontent.com/1888-Labs/MerkleForge/main/assets/brand/svg/mark/merkleforge-mark-white.svg">
      <img alt="MerkleForge" src="https://raw.githubusercontent.com/1888-Labs/MerkleForge/main/assets/brand/svg/mark/merkleforge-mark-black.svg" width="72">
    </picture>
  </a>
</p>

<h1 align="center">merkleforge-hash</h1>

<p align="center">SHA-256, Keccak-256, and BLAKE3 hash adapters for MerkleForge,<br>with domain-separated leaf and internal-node hashing.</p>

<p align="center">
  <a href="https://crates.io/crates/merkleforge-hash"><img alt="crates.io" src="https://img.shields.io/crates/v/merkleforge-hash.svg?style=flat-square&label=crates.io&color=000000"></a>
  <a href="https://docs.rs/merkleforge-hash"><img alt="docs.rs" src="https://img.shields.io/docsrs/merkleforge-hash?style=flat-square&label=docs.rs&color=000000"></a>
  <a href="#license"><img alt="license" src="https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-000000?style=flat-square"></a>
</p>

`merkleforge-hash` provides ready-made implementations of
`merkle_core::traits::HashFunction`. These adapters can drive every tree in
`merkle-variants`, so applications can switch hash algorithms by changing a
single generic type.

## Installation

```toml
[dependencies]
merkle-core = "0.4.1"
merkleforge-hash = "0.4.1"
```

Add `merkle-variants` when you want complete tree implementations:

```toml
[dependencies]
merkle-core = "0.4.1"
merkleforge-hash = "0.4.1"
merkle-variants = "0.4.1"
```

## Quick Start

```rust
use merkle_core::traits::HashFunction;
use merkleforge_hash::{Blake3, Keccak256, Sha256};

let sha = Sha256::hash(b"alice:100");
let keccak = Keccak256::hash(b"alice:100");
let blake = Blake3::hash(b"alice:100");

assert_eq!(sha.len(), 32);
assert_eq!(keccak.len(), 32);
assert_eq!(blake.len(), 32);
assert_ne!(sha, keccak);
assert_ne!(sha, blake);
```

Use an adapter with a tree:

```rust
use merkle_variants::BinaryMerkleTree;
use merkleforge_hash::Sha256;

let mut tree = BinaryMerkleTree::<Sha256>::new();
```

## Available Adapters

| Adapter | Algorithm | Digest | Best for |
| --- | --- | --- | --- |
| `Sha256` | SHA-256 | 32 bytes | Interoperable binary and sparse trees |
| `Keccak256` | Keccak-256 | 32 bytes | Ethereum-compatible Patricia trie roots |
| `Blake3` | BLAKE3 | 32 bytes | High-throughput non-Ethereum workloads |

## Domain Separation

Merkle trees must avoid confusing leaf data with internal node data. The
official adapters separate those domains:

| Adapter | Leaf domain | Internal-node domain |
| --- | --- | --- |
| `Sha256` | `SHA-256(0x00 || data)` | `SHA-256(0x01 || left || right)` |
| `Keccak256` | `Keccak-256(0x00 || data)` | `Keccak-256(0x01 || left || right)` |
| `Blake3` | `derive_key("MerkleForge 2026 leaf v1", data)` | `derive_key("MerkleForge 2026 internal-node v1", left || right)` |

Custom `HashFunction` implementations should preserve the same invariant.

## Choosing an Adapter

- Use `Keccak256` when roots must match Ethereum tooling.
- Use `Sha256` when broad interoperability or hardware SHA support matters.
- Use `Blake3` when raw software throughput matters and Ethereum compatibility
  is not required.

Benchmark data is available in [`BENCHMARKS.md`](https://github.com/1888-Labs/MerkleForge/blob/main/BENCHMARKS.md) and on the
public benchmark dashboard.

## Links

- API docs: <https://docs.rs/merkleforge-hash>
- Repository: <https://github.com/1888-Labs/MerkleForge>
- Website: <https://1888-labs.github.io/MerkleForge/>

## Part of MerkleForge

| Crate | Role |
| --- | --- |
| [`merkle-core`](https://crates.io/crates/merkle-core) | Shared traits, proof types, serialization, and errors |
| [`merkleforge-hash`](https://crates.io/crates/merkleforge-hash) | SHA-256, Keccak-256, and BLAKE3 adapters |
| [`merkle-variants`](https://crates.io/crates/merkle-variants) | Binary, Sparse, and Patricia tree implementations |

## Safety

`merkleforge-hash` uses `#![forbid(unsafe_code)]`. The crate contains no unsafe
blocks.

Security note: MerkleForge Framework has not yet completed an independent
security audit. Review the code and threat model before using it in
security-critical environments.

## License

Licensed under either of:

- [MIT License](https://github.com/1888-Labs/MerkleForge/blob/main/LICENSE)
- [Apache License, Version 2.0](https://github.com/1888-Labs/MerkleForge/blob/main/LICENSE-APACHE)

at your option.
