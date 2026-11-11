# Changelog

All notable changes to LoopSettle. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and the project uses [semantic versioning](https://semver.org/). Each entry links the experiments
whose results changed.

## [Unreleased]

### Added
- Liquidity floor (tight form of the delay bound): `liquidity_floors`, new columns in
  `loopsettle bound` and in the `regimes` experiment, METHODOLOGY §9b, claim 1g.
- `specs/`: audit findings and the post-v0.1.0 roadmap as specifications.

### Fixed
- `QueueOnShortfall` gross retry runs to a fixed point (it released one payment per edge per window).
- `DeferredNet`: the last cycle of a day falls exactly at its end for any cycle count.
- The share settled within 2.8 s is counted exactly.

### Changed
- Strict per-payer FIFO as an option (`micro:shortfall:<w>:<d>:<len>:fifo`).
- Claim 1f revised: the window bound does not explain retail vs large-value savings.

### Added (earlier)
- `paper/`: Portuguese draft of the JOSS paper, `paper.bib` and a JOSS-readiness checklist.
- `CONTRIBUTING.md` (EN/PT) with support expectations; this changelog.

## [0.1.0] — 2026-11-05

First public version.

### Added
- Settlement core in integer cents: gross real-time, deferred netting (N cycles a day) and atomic
  micro-cycles (`QueueAll` and `QueueOnShortfall`), streaming over any payment source.
- Topology: Hodge decomposition (conjugate gradient), exact maximum CDFD (min-cost flow, checked
  against an LP oracle), cycle netting.
- Synthetic generator calibrated to public Pix aggregates (BCB Olinda API), intraday and weekday
  profiles from the SPI 2024 annual report, receiver bias calibrated to BCB WP 619.
- The delay bound for any mechanism with maximum delay *d* (`bound.rs`, METHODOLOGY §9) and
  funding-window analysis (METHODOLOGY §10).
- Experiments: calibration, baseline, potential-vs-realizable, microcycle-sweep, stress (3 targets
  × 3 start times), finland-replication, robustness, bias-calibration, heterogeneity,
  funding-windows, regimes.
- Real-data commands `loopsettle bound`, `settle` and `export`, with aggregate-only output.
- Web interface (Rust → WebAssembly in a Web Worker, Canvas 2D), Portuguese and English.
- Eight property-tested invariants; byte-for-byte reproducible results; bilingual documentation and
  a reference library.

[Unreleased]: https://github.com/fabiodrneles/loopsettle/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/fabiodrneles/loopsettle/releases/tag/v0.1.0
