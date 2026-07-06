# Benchmarks

Hot-path benchmarks for shabdakosh, ported from the Rust oracle
(`rust-old/benches/benchmarks.rs`, criterion) to `benches/hotpath.bcyr`.

Run with:

```sh
cyrius bench            # auto-discovers benches/
```

## Latest results

- **Date**: 2026-07-06
- **Toolchain**: cyrius 6.4.8
- **Revision**: `159ed0e`
- **Platform**: Linux 7.0.14-arch1-1, x86_64
- **Dictionary**: base English (`_cmudict_data.cyr`), ~10,617 entries

| Benchmark | Avg | Min | Max | Iters |
|---|---:|---:|---:|---:|
| dict english construction | 9.626 ms | 9.494 ms | 9.925 ms | 10 |
| **dict lookup hit** | **135 ns** | 127 ns | 165 ns | 200,000 |
| **dict lookup miss** | **162 ns** | 152 ns | 198 ns | 200,000 |
| trie construction | 8.558 ms | 8.032 ms | 9.385 ms | 10 |
| trie prefix search `'pre'` | 8.364 ms | 7.923 ms | 10.504 ms | 500 |
| binary serialize | 2.366 ms | 2.311 ms | 3.004 ms | 20 |
| binary deserialize | 10.692 ms | 10.536 ms | 11.274 ms | 20 |

## Reading the numbers

- **Dictionary lookup is the headline** and meets the O(1) design goal: **~135 ns per hit**,
  ~162 ns per miss — a single `lib/hashmap` probe with **no per-call allocation** (the
  lowercase fast-path, `_shbdk_is_lower_cstr`, skips the lowercase copy for already-lowercase
  words). shabdakosh is dictionary-first: this is the path that matters.
- **Construction is a one-time ~9.6 ms** (load the ~10.6k entries into a fresh hashmap).
  `dictionary/static_dict.cyr` amortizes this to once-per-process via a cached singleton.
  Note: the Rust `phf` feature baked this table at compile time (near-zero construction); CYRIUS
  has no const-eval to reproduce that — see the filed cyrius proposal
  `2026-07-05-const-eval-comptime.md`. The static dict's *surface* and *lookup* match; only the
  construction cost differs.
- **Binary serialize (2.4 ms) / deserialize (10.7 ms)** and **trie build/search (~8.4 ms)** are
  alloc-heavy whole-dictionary operations, benched at modest iteration counts because the CYRIUS
  bump allocator never frees (high-N loops would exhaust the arena, not reflect steady state).

## Methodology

- The alloc-free **lookup** path is **batch-timed** (`bench_run_batch`, 200 rounds × 1000 calls)
  to amortize the ~40 ns per-call `clock_gettime` overhead — otherwise clock overhead would
  dominate a sub-200 ns operation.
- Alloc-heavy ops (construction, binary, trie, prefix search) use per-call timing at **modest N**;
  each iteration allocates (a fresh dict / a ~2 MB binary buffer / trie nodes / a results vec) and
  the bump allocator never frees, so N is kept small enough to stay within the arena while still
  giving a stable average.
- Numbers are single-machine and indicative, not a cross-platform guarantee. History is tracked in
  [`benches/history.csv`](../benches/history.csv).
