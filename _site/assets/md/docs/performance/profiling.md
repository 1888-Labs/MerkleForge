# Profiling & energy

Criterion answers *how long did this take?* Profiling answers *where did the CPU time go?* Use the
tools below when investigating construction, proof generation, hashing, memory access, or trie
traversal bottlenecks. All commands run from the repository root.

## Install tools

```bash
make install-profiling-tools
```

On Linux, `cargo-flamegraph` uses `perf`. Depending on your distribution:

```bash
sudo apt install linux-tools-common linux-tools-generic
```

If kernel settings block profiling, lower the restriction temporarily:

```bash
sudo sysctl kernel.perf_event_paranoid=1
```

## Flamegraphs

```bash
./scripts/flamegraph.sh baseline_construction
```

This writes `flamegraph.svg`. Open it in a browser and inspect the widest frames first; width is
sampled CPU time. Useful targets: `baseline_construction`, `hash_throughput`, `binary_tree`,
`sparse_tree`, `patricia_trie`, `comparison`.

`profiling/flamegraph.svg` in the repository is a reference artifact showing the expected shape.
Regenerate a machine-specific flamegraph before reporting numbers.

## Cache and instruction counters

```bash
perf stat -e cycles,instructions,cache-misses,cache-references \
  cargo bench --bench hash_throughput
```

Per tree:

```bash
perf stat -e cycles,instructions,cache-misses,cache-references cargo bench --bench binary_tree
perf stat -e cycles,instructions,cache-misses,cache-references cargo bench --bench sparse_tree
perf stat -e cycles,instructions,cache-misses,cache-references cargo bench --bench patricia_trie
```

## Energy

The default energy workload builds a 100,000-leaf binary tree with SHA-256, Keccak-256, and BLAKE3.
Criterion measures wall-clock time; Linux `perf` collects CPU package energy externally.

```bash
cargo bench --bench bench_energy   # wall clock → target/criterion/bench_energy/
```

### RAPL counters with `perf`

On Intel and some AMD machines the kernel exposes RAPL counters through the `power/*` PMU. Check
support, then run the helper script:

```bash
perf list 'power/energy-*'
./scripts/energy-perf.sh bench_energy
```

The script collects `power/energy-pkg/` (package Joules), `power/energy-cores/` and
`power/energy-ram/` when available, `cycles`, `instructions`, `cache-references`, `cache-misses`,
and `duration_time`. Some machines need elevated privileges:

```bash
sudo ./scripts/energy-perf.sh bench_energy
# or
sudo sysctl kernel.perf_event_paranoid=0
```

Use the privileged path only on a machine you control. The explicit command behind the script:

```bash
sudo perf stat -a -r 3 \
  -e cycles,instructions,cache-references,cache-misses,duration_time,power/energy-pkg/ \
  -- cargo bench --bench bench_energy
```

`-a` records across all CPUs, which matters because package energy is a socket-level reading.

### Fallbacks

If `perf` doesn't show `power/energy-pkg/`, try `turbostat`:

```bash
sudo turbostat --Summary --Joules --out profiling/turbostat-energy.txt -- \
  cargo bench --bench bench_energy
```

`powertop` is less precise but can capture a repeatable system-level profile while the benchmark
runs in another terminal:

```bash
sudo powertop --time=60 --html=profiling/powertop-energy.html
```

### Interpreting results

* Lower elapsed time: construction finished sooner.
* Lower `power/energy-pkg/` Joules: the CPU package used less energy.
* Fewer cycles for the same workload: less CPU work.
* Cache misses explain why runs with similar hash speed differ in time or energy.

Energy measurements are hardware-specific. Report the machine, kernel, CPU governor, Rust version,
and command line with every table. BLAKE3's lower cycles per byte usually means less energy per
proof, which is why the suite measures all three algorithms.

The full guide lives in
[`benchmarks/ENERGY.md`](https://github.com/1888-Labs/MerkleForge/blob/main/benchmarks/ENERGY.md)
and [`profiling/README.md`](https://github.com/1888-Labs/MerkleForge/blob/main/profiling/README.md).
