# Akida Open Outreach — Technical Brief

**Date**: February 27, 2026
**Project**: rustChip (ecoPrimal)
**License**: AGPL-3.0-or-later
**Repository**: https://github.com/ecoPrimal/rustChip

---

## What This Is

A standing technical brief for BrainChip's engineering team. It documents
what has been built, what has been measured, and what would benefit from
deeper collaboration — with enough data for an engineering team to evaluate
independently.

Everything referenced here is published under AGPL-3.0. The repository is
public. The benchmarks are reproducible. The findings are version-controlled.

This is not a sales inquiry. We already use your hardware in production.
This is an engineering-to-engineering conversation.

---

## The Situation

We run a pure Rust scientific computing pipeline that uses the AKD1000 as a
neuromorphic coprocessor for physics simulations (lattice QCD, molecular
dynamics, transport coefficients). The pipeline has 664 passing tests, 39
validated suites, and 5,978 live NPU hardware calls in a 24-hour production
run (Experiment 022, February 27, 2026).

In building this system, we probed the hardware independently of the Python
SDK and found 10 cases where actual silicon behavior differs from SDK
documentation. These are documented in `../../docs/BEYOND_SDK.md` with
reproduction scripts.

We then built a pure Rust driver — no Python, no C++ SDK, no vendor kernel
module required — and published it as this repository.

---

## What We Give

- `cargo build --release` — a working driver for your hardware
- 10 benchmark binaries that reproduce every hardware discovery
- FlatBuffer model format documentation and injection code
- VFIO setup documentation (replaces per-kernel C module rebuilds)
- Phase D VFIO driver (active), Phase E Rust kernel module (queued)
- All measurement data published under AGPL-3.0

## What You Can Give

- Confirmation or correction of the register map in `specs/SILICON_SPEC.md`
  (the `inferred` and `hypothetical` entries specifically)
- The DW eDMA register offset table (subset of DesignWare eDMA databook)
- AKD1500 hardware sample for validation (currently extrapolated from AKD1000)
- Technical contact for Phase E (Rust kernel module) questions

## What You Get

- A Rust driver that works on any kernel version (VFIO path)
- Phase E kernel module that doesn't require per-kernel rebuilds
- The only public scientific computing validation suite for Akida hardware
- Novel use cases: lattice QCD phase detection, transport prediction,
  adaptive HMC steering — not vision, not keyword spotting
- Published benchmarks under open license that serve as technical writing
  for your platform

---

## Documents

| Document | Description |
|----------|-------------|
| [`TECHNICAL_BRIEF.md`](TECHNICAL_BRIEF.md) | Full technical findings: production results, 10 SDK discoveries, AKD1500 analysis, roadmap, Phase D/E driver story |
| [`BENCHMARK_DATASHEET.md`](BENCHMARK_DATASHEET.md) | Raw numbers: latency, throughput, power, precision parity, production (Exp 022) |
| `../../docs/BEYOND_SDK.md` | 10 hardware discoveries with methodology and reference values |
| `../../docs/HARDWARE.md` | NP mesh architecture, BAR layout, register map deep-dive |

---

## Key Numbers (AKD1000, PCIe x1 Gen2, Feb 2026)

| Metric | Value |
|--------|-------|
| DMA throughput (sustained) | **37 MB/s** |
| Single inference | **54 µs / 18,500 Hz** |
| Batch=8 inference | **390 µs/sample / 20,700 /s** |
| Energy per inference | **1.4 µJ** |
| Weight swap (3 classifiers) | **86 µs** |
| Production calls (24h) | **5,978** |
| Thermalization savings | **63%** |
| Rejection prediction accuracy | **80.4%** |

---

## Contact

Technical discussions conducted in the open. File a GitHub issue or submit
a pull request. All findings will be published.
