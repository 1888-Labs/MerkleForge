//! Comparative benchmarks against external Merkle tree crates.
//!
//! Run with:
//! ```bash
//! cargo bench --bench comparison
//! ```

use criterion::{BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use merkle_core::{traits::Serializable, types::LeafIndex};
use merkle_light::{hash::Algorithm as LightAlgorithm, merkle::MerkleTree as LightMerkleTree};
use merkle_variants::BinaryMerkleTree as ForgeMerkleTree;
use merkleforge_hash::Sha256 as ForgeSha256;
use rs_merkle::{Hasher as RsHasher, MerkleTree as RsMerkleTree, algorithms::Sha256 as RsSha256};
use sha2::{Digest as Sha2Digest, Sha256 as Sha2_256};
use std::{hash::Hasher, hint::black_box, path::Path, time::Duration};

/// Leaves per tree; `COMPARISON_LEAVES` overrides the default 10,000.
fn leaf_count() -> usize {
    std::env::var("COMPARISON_LEAVES")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(10_000)
}

#[derive(Clone, Default)]
struct LightSha256(Sha2_256);

impl Hasher for LightSha256 {
    fn finish(&self) -> u64 {
        0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0.update(bytes);
    }
}

impl LightAlgorithm<[u8; 32]> for LightSha256 {
    fn hash(&mut self) -> [u8; 32] {
        self.0.finalize_reset().into()
    }

    fn reset(&mut self) {
        self.0.reset();
    }
}

fn leaf_data() -> Vec<[u8; 32]> {
    (0..leaf_count())
        .map(|index| {
            let mut leaf = [0_u8; 32];
            leaf[..8].copy_from_slice(&index.to_le_bytes());
            leaf[8..16].copy_from_slice(&(index.rotate_left(7)).to_le_bytes());
            leaf
        })
        .collect()
}

fn forge_tree(leaves: &[[u8; 32]]) -> ForgeMerkleTree<ForgeSha256> {
    let mut tree = ForgeMerkleTree::with_capacity(leaves.len());
    for leaf in leaves {
        tree.insert(leaf)
            .expect("comparison leaves are always non-empty");
    }
    tree
}

fn rs_leaves(leaves: &[[u8; 32]]) -> Vec<<RsSha256 as RsHasher>::Hash> {
    leaves.iter().map(|leaf| RsSha256::hash(leaf)).collect()
}

fn rs_tree(leaves: &[[u8; 32]]) -> RsMerkleTree<RsSha256> {
    RsMerkleTree::<RsSha256>::from_leaves(&rs_leaves(leaves))
}

fn light_tree(leaves: &[[u8; 32]]) -> LightMerkleTree<[u8; 32], LightSha256> {
    leaves.iter().copied().collect()
}

fn comparison_benchmarks(c: &mut Criterion) {
    let leaves = leaf_data();
    let rs_hashed_leaves = rs_leaves(&leaves);

    let forge = forge_tree(&leaves);
    let forge_proof = forge
        .generate_proof(LeafIndex(leaf_count() / 2))
        .expect("proof index exists");
    let forge_root = *forge.root().expect("comparison tree is non-empty");
    let forge_leaf = leaves[leaf_count() / 2];

    let rs = RsMerkleTree::<RsSha256>::from_leaves(&rs_hashed_leaves);
    let rs_proof = rs.proof(&[leaf_count() / 2]);
    let rs_root = rs.root().expect("comparison tree is non-empty");
    let rs_leaf = rs_hashed_leaves[leaf_count() / 2];

    let light = light_tree(&leaves);
    let light_proof = light.gen_proof(leaf_count() / 2);

    let forge_proof_size = forge_proof.to_bytes().expect("serializes").len();
    let rs_proof_size = rs_proof.to_bytes().len();
    let light_proof_size = light_proof.lemma().len() * 32;

    if std::env::var_os("MERKLEFORGE_PRINT_COMPARISON_SIZES").is_some() {
        eprintln!(
            "comparison proof sizes: MerkleForge={forge_proof_size} bytes, rs-merkle={rs_proof_size} bytes, merkle_light={light_proof_size} bytes"
        );
    }

    let mut construction = c.benchmark_group("comparison/construction");
    construction.throughput(Throughput::Elements(leaf_count() as u64));
    construction.bench_with_input(
        BenchmarkId::new("library", "MerkleForge"),
        &leaves,
        |b, input| {
            b.iter_batched(
                || input.clone(),
                |data| black_box(forge_tree(black_box(&data))),
                BatchSize::LargeInput,
            );
        },
    );
    construction.bench_with_input(
        BenchmarkId::new("library", "MerkleForge from_leaves"),
        &leaves,
        |b, input| {
            b.iter_batched(
                || input.clone(),
                |data| {
                    black_box(ForgeMerkleTree::<ForgeSha256>::from_leaves(black_box(
                        &data,
                    )))
                },
                BatchSize::LargeInput,
            );
        },
    );
    construction.bench_with_input(
        BenchmarkId::new("library", "rs-merkle"),
        &leaves,
        |b, input| {
            b.iter_batched(
                || input.clone(),
                |data| black_box(rs_tree(black_box(&data))),
                BatchSize::LargeInput,
            );
        },
    );
    construction.bench_with_input(
        BenchmarkId::new("library", "merkle_light"),
        &leaves,
        |b, input| {
            b.iter_batched(
                || input.clone(),
                |data| black_box(light_tree(black_box(&data))),
                BatchSize::LargeInput,
            );
        },
    );
    construction.finish();

    let mut proof_size = c.benchmark_group("comparison/proof_size_bytes");
    proof_size.bench_function("MerkleForge", |b| {
        b.iter(|| black_box(forge_proof.to_bytes().expect("serializes").len()));
    });
    proof_size.bench_function("rs-merkle", |b| {
        b.iter(|| black_box(rs_proof.to_bytes().len()));
    });
    proof_size.bench_function("merkle_light", |b| {
        b.iter(|| black_box(light_proof.lemma().len() * 32));
    });
    proof_size.finish();

    let mut generation = c.benchmark_group("comparison/proof_generation");
    generation.bench_function("MerkleForge", |b| {
        b.iter(|| black_box(forge.generate_proof(LeafIndex(black_box(leaf_count() / 2)))));
    });
    generation.bench_function("rs-merkle", |b| {
        b.iter(|| black_box(rs.proof(&[black_box(leaf_count() / 2)])));
    });
    generation.bench_function("merkle_light", |b| {
        b.iter(|| black_box(light.gen_proof(black_box(leaf_count() / 2))));
    });
    generation.finish();

    let mut verification = c.benchmark_group("comparison/proof_verification");
    verification.bench_function("MerkleForge", |b| {
        b.iter(|| {
            black_box(ForgeMerkleTree::<ForgeSha256>::verify(
                black_box(&forge_root),
                black_box(&forge_leaf),
                black_box(&forge_proof),
            ));
        });
    });
    verification.bench_function("rs-merkle", |b| {
        b.iter(|| {
            black_box(rs_proof.verify(
                black_box(rs_root),
                black_box(&[leaf_count() / 2]),
                black_box(&[rs_leaf]),
                black_box(leaf_count()),
            ));
        });
    });
    verification.bench_function("merkle_light", |b| {
        b.iter(|| {
            black_box(light_proof.validate::<LightSha256>());
        });
    });
    verification.finish();
}

/// Multiproofs for 100 leaves spread evenly across the tree: generation and
/// verification. merkle_light has no multiproofs.
fn multiproof_benchmarks(c: &mut Criterion) {
    let leaves = leaf_data();
    let n = leaves.len();
    let indices: Vec<usize> = (0..100).map(|k| k * n / 100 + 3).collect();
    let forge_indices: Vec<LeafIndex> = indices.iter().copied().map(LeafIndex).collect();
    let forge_leaves: Vec<[u8; 32]> = indices.iter().map(|&i| leaves[i]).collect();
    let rs_hashed = rs_leaves(&leaves);
    let rs_leaf_hashes: Vec<[u8; 32]> = indices.iter().map(|&i| rs_hashed[i]).collect();

    let forge = ForgeMerkleTree::<ForgeSha256>::from_leaves(&leaves).expect("non-empty");
    let forge_root = *forge.root().expect("non-empty");
    let forge_proof = forge
        .generate_multi_proof(&forge_indices)
        .expect("valid indices");
    let rs = RsMerkleTree::<RsSha256>::from_leaves(&rs_hashed);
    let rs_root = rs.root().expect("non-empty");
    let rs_proof = rs.proof(&indices);
    assert!(ForgeMerkleTree::<ForgeSha256>::verify_multi_proof(
        &forge_root,
        &forge_leaves,
        &forge_proof
    ));
    assert!(rs_proof.verify(rs_root, &indices, &rs_leaf_hashes, n));

    let mut generation = c.benchmark_group("comparison/multiproof_generation_100");
    generation.bench_function("MerkleForge", |b| {
        b.iter(|| black_box(forge.generate_multi_proof(black_box(&forge_indices))));
    });
    generation.bench_function("rs-merkle", |b| {
        b.iter(|| black_box(rs.proof(black_box(&indices))));
    });
    generation.finish();

    // Verifying 10,000 single proofs: MerkleForge's `verify_batch` (all cores
    // with the `parallel` feature, one thread without) against a loop over each
    // library's own verifier.
    let batch: Vec<usize> = (0..10_000).map(|k| (k * 7919) % n).collect();
    let forge_items: Vec<([u8; 32], merkle_core::types::MerkleProof<[u8; 32]>)> = batch
        .iter()
        .map(|&i| {
            (
                leaves[i],
                forge.generate_proof(LeafIndex(i)).expect("valid index"),
            )
        })
        .collect();
    let rs_items: Vec<(usize, rs_merkle::MerkleProof<RsSha256>)> =
        batch.iter().map(|&i| (i, rs.proof(&[i]))).collect();
    let light = light_tree(&leaves);
    let light_proofs: Vec<_> = batch.iter().map(|&i| light.gen_proof(i)).collect();
    assert!(ForgeMerkleTree::<ForgeSha256>::verify_batch(
        &forge_root,
        &forge_items
    ));

    let mut many = c.benchmark_group("comparison/verify_10k_proofs");
    many.sample_size(10);
    many.bench_function("MerkleForge verify_batch", |b| {
        b.iter(|| {
            black_box(ForgeMerkleTree::<ForgeSha256>::verify_batch(
                black_box(&forge_root),
                black_box(&forge_items),
            ))
        });
    });
    many.bench_function("rs-merkle", |b| {
        b.iter(|| {
            black_box(
                rs_items
                    .iter()
                    .all(|(i, p)| p.verify(rs_root, &[*i], &[rs_hashed[*i]], n)),
            )
        });
    });
    many.bench_function("merkle_light", |b| {
        b.iter(|| black_box(light_proofs.iter().all(|p| p.validate::<LightSha256>())));
    });
    many.finish();

    let mut verification = c.benchmark_group("comparison/multiproof_verification_100");
    verification.bench_function("MerkleForge", |b| {
        b.iter(|| {
            black_box(ForgeMerkleTree::<ForgeSha256>::verify_multi_proof(
                black_box(&forge_root),
                black_box(&forge_leaves),
                black_box(&forge_proof),
            ))
        });
    });
    verification.bench_function("rs-merkle", |b| {
        b.iter(|| {
            black_box(rs_proof.verify(
                black_box(rs_root),
                black_box(&indices),
                black_box(&rs_leaf_hashes),
                n,
            ))
        });
    });
    verification.finish();
}

fn benchmark_config() -> Criterion {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("merkle-bench must be inside the workspace");
    let output_directory = workspace_root.join("target/criterion");

    Criterion::default()
        .output_directory(&output_directory)
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3))
}

criterion_group! {
    name = benches;
    config = benchmark_config();
    targets = comparison_benchmarks, multiproof_benchmarks
}
criterion_main!(benches);
