# rustChip

Pure Rust software stack for BrainChip Akida neuromorphic processors (AKD1000, AKD1500).

Forked from [Brainchip-Inc/akida_dw_edma](https://github.com/Brainchip-Inc/akida_dw_edma).
C kernel module → deprecated (see [DEPRECATED.md](DEPRECATED.md)).
All active development is in `rust/` and these top-level crates.

No Python. No C++ SDK. No MetaTF. No kernel module required.

---

## What this is

A fruiting body from the [ecoPrimals](https://github.com/ecoPrimals) project —
self-contained, carries everything it needs to replicate, designed to be handed
to the BrainChip engineering team as a standalone working system.

It emerged from `toadStool`, the shared compute library behind five scientific
validation suites (lattice QCD, microbial ecology, atmospheric physics, neural
architectures, uncertainty quantification). The AKD1000 was used in production
physics simulation — 5,978 live hardware calls, 24 hours, lattice SU(3).
This is the distillation of what we learned.

---

## Workspace

```
rustChip/
├── crates/
│   ├── akida-chip/      silicon model — register map, NP mesh, BAR layout (no deps)
│   ├── akida-driver/    full driver — VFIO primary, kernel fallback, DMA, inference
│   ├── akida-models/    FlatBuffer model parser + program_external() injection
│   ├── akida-bench/     benchmark suite — reproduces all 10 BEYOND_SDK discoveries
│   └── akida-cli/       `akida` command-line tool
├── docs/
│   ├── BEYOND_SDK.md    10 hardware discoveries overturning SDK assumptions
│   ├── HARDWARE.md      NP mesh architecture, BAR layout, register map deep-dive
│   ├── TECHNICAL_BRIEF.md  production use in lattice QCD (Exp 022)
│   └── BENCHMARK_DATASHEET.md  full measurement dataset
├── BEYOND_SDK.md        (also at root — the most important document)
└── DEPRECATED.md        why the C code at root is no longer the primary path
```

---

## Build

```sh
# Prerequisites: Rust stable, hardware connected
cd rustChip/
cargo build --release

# List devices
cargo run --bin akida -- enumerate

# Full benchmark suite (reproduces BEYOND_SDK.md)
cargo run --bin bench_dma
cargo run --bin bench_latency
cargo run --bin bench_batch
cargo run --bin bench_clock_modes
cargo run --bin bench_fc_width
cargo run --bin bench_fc_depth
cargo run --bin bench_channels
cargo run --bin bench_weight_mut
cargo run --bin bench_bar
```

---

## Backend selection

```text
Primary — VFIO (no kernel module):
  akida bind-vfio 0000:a1:00.0      # once, requires root/CAP_SYS_ADMIN
  cargo run --bin akida enumerate    # no root needed after binding

Fallback — C kernel module (if installed):
  sudo insmod akida-pcie.ko          # existing module
  cargo run --bin akida enumerate    # Rust driver opens /dev/akida*
```

The VFIO backend provides full DMA, IOMMU isolation, and works on any
kernel version. The kernel fallback is available when the C module is loaded.

---

## Measured results (AKD1000, PCIe x1 Gen2, Feb 2026)

| Metric | Measured |
|--------|----------|
| DMA throughput, sustained | 37 MB/s |
| Single inference | 54 µs / 18,500 Hz |
| Batch=8 inference | 390 µs/sample / 20,700 /s |
| Energy per inference | 1.4 µJ |
| Online weight swap (3 models) | 86 µs |
| Production calls (Exp 022, 24 h lattice QCD) | 5,978 |

---

## The 10 hardware discoveries

Full details in [`BEYOND_SDK.md`](BEYOND_SDK.md).

| # | SDK claim | Actual hardware |
|---|-----------|-----------------|
| 1 | InputConv: 1 or 3 channels only | Any channel count (1–64 tested) |
| 2 | FC layers run independently | All FC layers merge via SkipDMA (single HW pass) |
| 3 | Batch=1 only | Batch=8 amortises PCIe: 948→390 µs/sample (2.4×) |
| 4 | One clock mode | 3 modes: Performance / Economy / LowPower |
| 5 | Max FC width ~hundreds | Tested to 8192+ neurons (SRAM-limited only) |
| 6 | Weight updates require reprogram | `set_variable()` updates live (~14 ms overhead) |
| 7 | "30 mW" chip power | Board floor 900 mW; chip compute below noise floor |
| 8 | 8 MB SRAM limit | BAR1 exposes 16 GB address space |
| 9 | Program binary is opaque | FlatBuffer: `program_info` + `program_data`; weights via DMA |
| 10 | Simple inference engine | C++ engine: SkipDMA, 51-bit threshold SRAM, `program_external()` |

---

## Driver roadmap

```
Phase A: Python SDK → Rust FFI wrapper          ✅ done
Phase B: C++ Engine → Rust FFI to libakida.so   ✅ done
Phase C: Direct ioctl/mmap on /dev/akida0        ✅ done  (Feb 26, 2026)
Phase D: Pure Rust VFIO driver (this repo)       ✅ in progress
Phase E: Rust akida_pcie kernel module           queued
```

Phase D (VFIO) is the primary path in this repository. Phase E (Rust kernel module)
would use the stable kernel Rust bindings to permanently replace `akida-pcie-core.c`
without the kernel version ceiling.

---

## AKD1500 compatibility

All BEYOND_SDK findings transfer directly to AKD1500 (same Akida 1.0 IP).
One constant changes in `akida-chip/src/pcie.rs`: `AKD1500 = 0xA500`.

AKD1500 adds: PCIe x2 Gen2 (2× bandwidth), SPI master/slave, hardware SLEEP pin,
7×7 mm BGA169, 24 GPIO. The VFIO driver handles all of these without code changes
(PCIe x2 is transparent to software; SPI/GPIO use different interfaces).

---

## Scientific context

rustChip emerged from using the AKD1000 as a neuromorphic coprocessor in lattice
QCD simulations. The chip ran Echo State Network inference to steer HMC sampling
— 5,978 live calls over 24 hours, achieving 63% thermalization savings and 80.4%
rejection prediction accuracy on a 32⁴ SU(3) lattice.

That work lives at [syntheticChemistry/hotSpring](https://github.com/syntheticChemistry/hotSpring).
The full technical writeup is in [`docs/TECHNICAL_BRIEF.md`](docs/TECHNICAL_BRIEF.md).

---

## License

AGPL-3.0-or-later.
The original C kernel module files at the repository root are GPL-2.0 (BrainChip Inc.).
