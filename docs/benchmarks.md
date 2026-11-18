# Benchmarks

Hot-path benchmarks for shabdakosh, ported from the Rust oracle
(`rust-old/benches/benchmarks.rs`, criterion) to `benches/hotpath.bcyr`.

Run with:

```sh
cyrius bench            # auto-discovers benches/
```

## Latest results

- **Date**: 2026-11-17
- **Toolchain**: cyrius 6.7.18
- **Revision**: 3.1.2 working tree (on `7642b9b`)
- **Platform**: Linux 7.3.4-arch1-2, x86_64
- **Dictionary**: base English (`_cmudict_data.cyr`), ~10,617 entries

| Benchmark | Avg | Min | Max | Iters |
|---|---:|---:|---:|---:|
| **static first lookup** (3.1.2) | **34.9 µs** | 34.9 µs | 34.9 µs | 1 |
| dict english construction | 8.184 ms | 8.184 ms | 8.184 ms | 10 |
| **dict lookup hit** | **144 ns** | 144 ns | 144 ns | 200,000 |
| **dict lookup miss** | **182 ns** | 182 ns | 182 ns | 200,000 |
| dict lookup hit, capitals (3.1.2) | 186 ns | 186 ns | 186 ns | 2,000 |
| **static lookup hit** (3.1.2) | **124 ns** | 124 ns | 124 ns | 200,000 |
| **static lookup miss** (3.1.2) | **125 ns** | 125 ns | 125 ns | 200,000 |
| static lookup hit, capitals (3.1.2) | 127 ns | 127 ns | 127 ns | 200,000 |
| trie construction | 7.347 ms | 7.347 ms | 7.347 ms | 10 |
| trie prefix search `'pre'` | 6.263 ms | 5.891 ms | 9.579 ms | 500 |
| binary serialize | 6.196 ms | 6.118 ms | 6.240 ms | 20 |
| binary deserialize | 6.859 ms | 6.828 ms | 6.884 ms | 20 |
| to_cmudict | 11.835 ms | 11.835 ms | 11.835 ms | 10 |
| to_ipa | 10.750 ms | 10.750 ms | 10.750 ms | 10 |
| to_json | 33.140 ms | 33.140 ms | 33.140 ms | 10 |

Rows whose min and max equal the average were timed by `lib/bench` as a single window.

`static first lookup` runs first in the process, before any fixture is built, and is timed once:
it is the cost the perfect hash removed. Run against the 3.1.1 `static_dict.cyr` on the same
machine, that row read 8.4–9.0 ms (it built the whole dictionary), and the static hit, miss and
capitals rows read 152, 184–262 and 184 ns.

Against 3.1.1, every row that existed then is within its usual spread. Construction read
8.2–8.4 ms across today's full runs (7.8 ms recorded for 3.1.1), but alternating runs of the
3.1.1 and 3.1.2 loaders on their own give ~7.8 ms for both, so it is the session, not the
loader. `dict lookup miss` stays the noisiest row (182–298 ns today), so a single reading of it
is not a regression signal.

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
