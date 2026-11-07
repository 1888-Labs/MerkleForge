# Profiling & energy

Benchmarks answer *how long did this take?* Profiling answers *where did the CPU time go?* Use the
tools below when investigating construction, proof generation, hashing, memory access, or trie
traversal bottlenecks.

<LangBar />

## Install tools

<Rust>
  All commands on this page run from the root of the
  [MerkleForge](https://github.com/1888-Labs/MerkleForge) repository.

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
</Rust>

<TypeScript>
  Nothing to install: Node.js 22 and newer include the V8 CPU and heap profilers and the inspector
  used by Chrome DevTools. To profile the comparison suite, build the
  [MerkleForge-ts](https://github.com/1888-Labs/MerkleForge-ts) packages and install the
  `bench/` dependencies first:

  ```bash
  git clone https://github.com/1888-Labs/MerkleForge-ts.git
  cd MerkleForge-ts
  pnpm install && pnpm build
  cd bench
  npm install   # comparison libraries, kept out of the pnpm workspace
  ```

  To profile your own workload instead, write a small script that does only the work you care
  about, for example:

  ```js [build-tree.mjs]
  import { utf8 } from '@merkleforge/core'
  import { Keccak256 } from '@merkleforge/hash'
  import { BinaryMerkleTree } from '@merkleforge/variants'

  const leaves = Array.from({ length: 100_000 }, (_, i) => utf8(`leaf-${i}`))
  const tree = BinaryMerkleTree.fromLeaves(Keccak256, leaves)
  console.log(tree.root())
  ```
</TypeScript>

## CPU profiles and flamegraphs

<Rust>
  ```bash
  ./scripts/flamegraph.sh baseline_construction
  ```

  This writes `flamegraph.svg`. Open it in a browser and inspect the widest frames first; width is
  sampled CPU time. Useful targets: `baseline_construction`, `hash_throughput`, `binary_tree`,
  `sparse_tree`, `patricia_trie`, `comparison`.

  `profiling/flamegraph.svg` in the repository is a reference artifact showing the expected shape.
  Regenerate a machine-specific flamegraph before reporting numbers.
</Rust>

<TypeScript>
  `--cpu-prof` starts the V8 CPU profiler and writes a `.cpuprofile` file to the current directory
  when the process exits:

  ```bash
  node --cpu-prof build-tree.mjs

  # The bench suite, from bench/ (npm run bench runs node --expose-gc run.mjs).
  # SIZES limits the tree sizes.
  SIZES=131072 node --expose-gc --cpu-prof run.mjs
  ```

  Open the `.cpuprofile` in the **Performance** panel of Chrome DevTools to see a flame chart, and
  inspect the widest frames first. The bench suite also runs the comparison libraries, so filter
  the profile to MerkleForge frames, or profile a script of your own.

  To profile interactively, start Node.js with the inspector and attach Chrome DevTools from
  `chrome://inspect`:

  ```bash
  node --inspect-brk build-tree.mjs
  ```

  `--inspect-brk` pauses before the first line so you can start recording first; `--inspect` starts
  without pausing.
</TypeScript>

## Hardware counters and memory

<Rust>
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
</Rust>

<TypeScript>
  `--heap-prof` starts the V8 sampling heap profiler and writes a `.heapprofile` file when the process
  exits. Load it in the **Memory** panel of Chrome DevTools to see which functions allocated the
  most:

  ```bash
  node --heap-prof build-tree.mjs
  ```

  On Linux, `perf stat` counts cycles, instructions, and cache events around any process, including
  Node.js:

  ```bash
  perf stat -e cycles,instructions,cache-misses,cache-references node build-tree.mjs
  ```

  The counts include V8 startup and JIT compilation, so compare runs of the same script.
</TypeScript>

## Energy

<TypeScript>
  :::note
  The energy workload and helper scripts below are part of the Rust repository. The `perf`,
  `turbostat`, and `powertop` commands measure any process, so you can replace the `cargo bench`
  command with a Node.js command such as `node build-tree.mjs`.
  :::
</TypeScript>

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
