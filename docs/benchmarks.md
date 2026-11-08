# Benchmarks

Hot-path benchmarks for shabdakosh, ported from the Rust oracle
(`rust-old/benches/benchmarks.rs`, criterion) to `benches/hotpath.bcyr`.

Run with:

```sh
cyrius bench            # auto-discovers benches/
```

## Latest results

- **Date**: 2026-11-07
- **Toolchain**: cyrius 6.7.18
- **Revision**: 3.1.1 working tree (on `88cea94`)
- **Platform**: Linux 7.2.14-arch1-1, x86_64
- **Dictionary**: base English (`_cmudict_data.cyr`), ~10,617 entries

| Benchmark | Avg | Min | Max | Iters |
|---|---:|---:|---:|---:|
| dict english construction | 7.783 ms | 7.783 ms | 7.783 ms | 10 |
| **dict lookup hit** | **146 ns** | 146 ns | 146 ns | 200,000 |
| **dict lookup miss** | **168 ns** | 168 ns | 168 ns | 200,000 |
| trie construction | 6.443 ms | 6.443 ms | 6.443 ms | 10 |
| trie prefix search `'pre'` | 6.411 ms | 6.048 ms | 10.354 ms | 500 |
| binary serialize | 6.409 ms | 6.196 ms | 6.435 ms | 20 |
| binary deserialize | 7.129 ms | 7.034 ms | 7.141 ms | 20 |
| to_cmudict | 11.986 ms | 11.986 ms | 11.986 ms | 10 |
| to_ipa | 10.942 ms | 10.942 ms | 10.942 ms | 10 |
| to_json | 33.969 ms | 33.969 ms | 33.969 ms | 10 |

Rows whose min and max equal the average were timed by `lib/bench` as a single window.

Against 3.1.0 on cyrius 6.7.9, five alternating runs of each build put every row within a few
percent. `dict lookup miss` is the noisiest row: it ranged 176-355 ns on 6.7.9 and 177-237 ns on
6.7.18 across those runs, so a single reading of it is not a regression signal.

## Reading the numbers

- **Dictionary lookup is the headline** and meets the O(1) design goal: **~145-160 ns per hit**,
  ~170-180 ns per miss — a single `lib/hashmap` probe with **no per-call allocation** (the
  lowercase fast-path, `_shbdk_is_lower_cstr`, skips the lowercase copy for already-lowercase
  words). shabdakosh is dictionary-first: this is the path that matters.
- **Construction is a one-time ~8 ms** (load the ~10.6k entries into a fresh hashmap).
  `dictionary/static_dict.cyr` amortizes this to once-per-process via a cached singleton.
  Note: the Rust `phf` feature baked this table at compile time (near-zero construction); CYRIUS
  has no const-eval to reproduce that — see the filed cyrius proposal
  `2026-07-05-const-eval-comptime.md`. The static dict's *surface* and *lookup* match; only the
  construction cost differs.
- **Every export sorts its keys** (since 3.1.0 for binary and JSON), so the same dictionary gives
  the same bytes in any process. The sort has its own merge sort, about 2.4 ms for the 10,617
  keys (9.5 ms through the generic one it replaced). That is most of the gap between
  **binary serialize (6.4 ms)**, which writes sorted, and the 2.4 ms recorded here in July, when it
  wrote in hash order. It is a smaller share of `to_cmudict` / `to_ipa` (~11-12 ms) and of
  `to_json` (~34 ms), whose cost is mostly building the JSON tree.
- **Binary deserialize (7.1 ms)** and **trie build/search (~6.4 ms)** are
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
