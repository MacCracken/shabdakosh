# Benchmarks

Hot-path benchmarks for shabdakosh, ported from the Rust oracle
(`3.1.3:rust-old/benches/benchmarks.rs`, criterion) to `benches/hotpath.bcyr`.

Run with:

```sh
cyrius bench            # auto-discovers benches/
```

## Latest results

- **Date**: 2026-11-17
- **Toolchain**: cyrius 6.7.18
- **Revision**: 3.1.3 (the M3 sweep)
- **Platform**: Linux 7.3.4-arch1-2, x86_64
- **Dictionary**: base English (`_cmudict_data.cyr`), ~10,617 entries

| Benchmark | Avg | Min | Max | Iters |
|---|---:|---:|---:|---:|
| static first lookup | 39.0 µs | 39.0 µs | 39.0 µs | 1 |
| dict english construction | 8.822 ms | 8.822 ms | 8.822 ms | 10 |
| **dict lookup hit** | **150 ns** | 150 ns | 150 ns | 200,000 |
| dict lookup miss | 228 ns | 228 ns | 228 ns | 200,000 |
| dict lookup hit, capitals | 192 ns | 192 ns | 192 ns | 2,000 |
| **static lookup hit** | **116 ns** | 116 ns | 116 ns | 200,000 |
| static lookup miss | 125 ns | 125 ns | 125 ns | 200,000 |
| static lookup hit, capitals | 117 ns | 117 ns | 117 ns | 200,000 |
| trie construction | 8.042 ms | 8.042 ms | 8.042 ms | 10 |
| trie prefix search `'pre'` | 6.821 ms | 6.312 ms | 8.250 ms | 500 |
| trie search, prebuilt (`comp`, `th`, `psych`) (new) | 32.2 µs | 31.6 µs | 33.2 µs | 500 |
| binary serialize | 6.862 ms | 6.781 ms | 6.869 ms | 20 |
| binary deserialize | 7.928 ms | 7.887 ms | 7.987 ms | 20 |
| to_cmudict | 13.046 ms | 13.046 ms | 13.046 ms | 10 |
| to_ipa | 12.673 ms | 12.673 ms | 12.673 ms | 10 |
| to_json | 34.963 ms | 34.963 ms | 34.963 ms | 10 |

Rows whose min and max equal the average were timed by `lib/bench` as a single window.

**Against 3.1.2**, from alternating runs of this tree and a `git archive` of the 3.1.2 commit on
the same machine (five pairs). The comparison is the trustworthy part: single runs moved a whole
column by ~10% at times today.

- **Construction, dictionary lookups, trie construction and `to_json`** are unchanged within
  their spread. `dict lookup miss` remains the noisiest row (3.1.2 itself read 167–389 ns today).
- **Static lookups are faster** (hit ~113–116 ns against ~125–131). A single pass now finds the
  word's length and whether it needs Unicode lowercasing, where `strlen` used to make its own
  pass.
- **The format layer costs more, and the cost is the new checks** (ADR 005):
  - `to_ipa` +12%: the writer checks whether adjacent symbols would merge, through a precomputed
    continuation bitmap;
  - `to_cmudict` +8–10% and the binary serializer +1–8%: they validate each word before
    writing it;
  - the binary deserializer +6–14%: it checks every value as well as every length.
- **The new prebuilt-trie row** is the Rust bench's own workload: three prefix searches on one
  trie. The `'pre'` row rebuilds the trie per call, so it is mostly construction.

## Reading the numbers

- **Dictionary lookup is the headline** and meets the O(1) design goal: **~145-160 ns per hit**,
  ~170-180 ns per miss — a single `lib/hashmap` probe with **no per-call allocation** (the
  lowercase fast-path, `_shbdk_is_lower_cstr`, skips the lowercase copy for already-lowercase
  words). shabdakosh is dictionary-first: this is the path that matters.
- **Construction is a one-time ~8 ms** (load the ~10.6k entries into a fresh hashmap).
- **The static dictionary skips construction** (3.1.2). Like the Rust `phf` feature, it looks
  words up through a perfect hash baked at build time (`gen_cmudict` writes it), so nothing is
  loaded: the first lookup in a process costs ~35 µs, in which it allocates and clears the 85 KB
  slot cache and decodes one entry. (Clearing it with the stdlib's byte-wise `memset` made that
  ~160 µs; one 8-byte store a slot brought it here.) A hit is ~124 ns, faster than the hashmap's ~145 ns, because it is
  one hash, one table read and one compare with no probing. A miss is the same cost as a hit
  (~125 ns, against ~180 ns), and a capitalised word costs nothing extra: the hash and the compare
  fold A-Z themselves, where the dictionary lookup copies the word to lowercase first (~185 ns).
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
