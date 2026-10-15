# Compile cost of `serde` + `serde_json` under the runner, 2026-10-15

Purpose: decide the worker's serialisation dependency with evidence, as the
protocol review asked, separating the format choice from its compilation
cost. Measurement only; no transport, no runner change, no limit change.

## Conditions

- Worktree `experiment-llm-normative-exploration-spec` at `3a20ad7`, lib test
  target (`cargo test --offline --locked --no-run --lib`) through
  `scripts/test_safe.py --compile-only --test-target lib`.
- `CARGO_BUILD_JOBS=1`, `CARGO_PROFILE_TEST_DEBUG=0`, 1 GiB cgroup, zero swap,
  stop threshold 768 MiB of cgroup memory, unchanged.
- Toolchain `rustc 1.87.0`, `cargo 1.87.0`.
- Variant: `serde = { version = "1", features = ["derive"] }`,
  `serde_json = "1"`, plus one probe test module
  (`src/experiments/serde_probe_tests.rs`, one round trip of a small DTO
  with escapes and non-ASCII, plus two malformed inputs). Dependencies
  fetched once with `cargo fetch` outside the runner, because the runner is
  offline; the compile itself ran inside the runner.
- Lockfile additions: `serde 1.0.229`, `serde_core 1.0.229`,
  `serde_derive 1.0.229`, `serde_json 1.0.154`, `itoa 1.0.18`,
  `memchr 2.8.3`, `zmij 1.0.23`, `syn 3.0.9` (a second `syn` beside the
  `2.0.119` that `zerocopy-derive` already uses). `proc-macro2`, `quote` and
  `unicode-ident` were rebuilt once because feature unification changed.
- Two figures are reported separately: process RSS (`ru_maxrss` of the
  children, as the runner records it) and the cgroup peak, which also counts
  page cache written during the build.

## Results

| Case | Procedure | Report | Elapsed | RSS | cgroup peak |
|---|---|---|---|---|---|
| B1 | baseline, `touch src/lib.rs`, incremental cache warm | `20261015T021756596505Z-2003349` | 1.8 s | 435 MiB | 435 MiB |
| B2 | same, repeated | `20261015T021758536119Z-2003645` | 1.8 s | 427 MiB | 423 MiB |
| V1 | variant, dependencies cold (one-off) | `20261015T021803709181Z-2003969` | 19.5 s | 696 MiB | 723 MiB |
| V2 | variant, `touch src/lib.rs`, incremental warm | `20261015T021823404167Z-2004717` | 1.9 s | 441 MiB | 438 MiB |
| V3 | same, repeated | `20261015T021825441960Z-2005059` | 1.8 s | 440 MiB | 441 MiB |
| V4 | variant, residuum rebuilt without incremental cache | `20261015T022942560002Z-2013198` | 12.6 s | 693 MiB | 664 MiB |
| V5 | same, repeated | `20261015T022955291617Z-2013627` | 12.5 s | 695 MiB | 657 MiB |
| B3 | baseline, residuum rebuilt without incremental cache | `20261015T023007983377Z-2014055` | 12.4 s | 684 MiB | 664 MiB |
| B4 | same, repeated | `20261015T023020564657Z-2014493` | 12.4 s | 682 MiB | 657 MiB |
| V6 | variant, rebuilt without incremental cache, after restoring | `20261015T023033156291Z-2014941` | 12.4 s | 699 MiB | 664 MiB |
| V7 | variant, `touch` only, incremental warm | `20261015T023045720055Z-2015369` | 1.9 s | 443 MiB | 439 MiB |
| probe | `serde_probe_round_trips_a_boundary_dto` | `20261015T021827428116Z-2005356` | — | — | 1 passed |

Artefact sizes: `libserde` 904 KiB, `libserde_json` 2.3 MiB,
`libserde_derive` 3.6 MiB (proc-macro).

## Reading

- Steady state, full rebuild of `residuum` without incremental cache: the
  variant adds about 10 to 15 MiB of RSS (682–684 → 693–699) and nothing
  measurable to the cgroup peak (657–664 in both) or to time (12.4 → 12.5 s).
- Steady state, incremental rebuild: about 8 to 15 MiB of RSS
  (427–435 → 438–443), no time difference.
- One-off dependency build: 19.5 s and a cgroup peak of 723 MiB, 45 MiB
  under the 768 MiB stop. It completed in one runner invocation.
- The earlier peaks of 790 MiB RSS recorded in the verification file came
  from incremental rebuilds after large edits of the test modules, not from
  the dependency: with or without `serde`, that mode is the one closest to
  the limit. A from-scratch rebuild sits lower.

Conclusion offered for the decision: the dependency is within budget in
every measured mode and its steady-state cost is below the variance of the
test-module edits already made. The constraint to keep is the one the review
set: serialisation localised at the worker boundary, with explicit versioned
DTOs, never `Serialize`/`Deserialize` across TypedAST, and byte and depth
limits checked before accepting work. The variant (Cargo.toml, Cargo.lock,
probe module and its registration) is left uncommitted pending that
decision; this record is committed on its own.
