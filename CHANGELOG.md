# Changelog

## [Unreleased] — divergent evolution from Brainchip-Inc/akida_dw_edma

### Added

**`crates/akida-chip`** — silicon model crate (no dependencies)
- `pcie`: vendor/device IDs for AKD1000 (`0x1E7C:0xBCA1`) and AKD1500 (`0x1E7C:0xA500`)
- `bar`: BAR layout (BAR0 16 MB, BAR1 16 GB NP mesh window, BAR3 32 MB)
- `regs`: BAR0 register map — confirmed addresses from direct probing + C++ symbol analysis
- `mesh`: NP mesh topology (5×8×2, 78 functional, SkipDMA routing model)
- `program`: FlatBuffer `program_info` / `program_data` format (reverse-engineered)

**`crates/akida-driver`** — full pure Rust driver
- VFIO backend: complete DMA (mlock, IOVA mapping, scatter-gather), BAR0 MMIO,
  inference trigger/poll, power measurement via hwmon
- Kernel backend: `/dev/akida*` read/write (fallback when C module present)
- Userspace backend: BAR mmap, development/register probing
- `vfio::bind_to_vfio()` / `unbind_from_vfio()` — replace C `install.sh`
- `vfio::iommu_group()` — IOMMU group discovery from sysfs
- Runtime capability discovery: `MeshTopology`, `ClockMode`, `BatchCapabilities`,
  `WeightMutationSupport`, `PcieConfig` — all from sysfs, nothing hardcoded
- Phase C sovereign driver: direct ioctl/mmap on `/dev/akida0` (Feb 26, 2026)

**`crates/akida-models`** — FlatBuffer model layer
- `.fbz` parser (FlatBuffer + Snappy)
- `program_external()` path: direct program binary injection, bypass SDK compilation
- Model zoo: ESN readout, transport predictor, phase classifier

**`crates/akida-bench`** — BEYOND_SDK reproduction suite
- `bench_channels` — Discovery 1: any input channel count works (1–64)
- `bench_fc_depth` — Discovery 2: FC chains merge via SkipDMA (8 layers ≈ 2 layers)
- `bench_batch` — Discovery 3: batch=8 sweet spot (390 µs/sample, 2.4× speedup)
- `bench_clock_modes` — Discovery 4: Economy = 19% slower, 18% less power
- `bench_fc_width` — Discovery 5: PCIe-dominated below 512 neurons
- `bench_weight_mut` — Discovery 6: weight mutation overhead ~14 ms
- `bench_dma` — Production: 37 MB/s sustained DMA
- `bench_latency` — Production: 54 µs / 18,500 Hz single inference
- `bench_bar` — Discovery 8: BAR layout probe (16 GB BAR1)

**`crates/akida-cli`** — `akida` command-line tool
- `akida enumerate` — list all devices with capabilities
- `akida info <addr>` — detailed single-device info including IOMMU group
- `akida bind-vfio <addr>` — bind to vfio-pci
- `akida unbind-vfio <addr>` — unbind and re-bind to akida driver
- `akida iommu-group <addr>` — show IOMMU group and /dev/vfio path

**Docs**
- `BEYOND_SDK.md` (root + `docs/`) — 10 hardware discoveries, raw measurements
- `docs/HARDWARE.md` — NP mesh architecture, BAR layout, per-NP capabilities
- `docs/TECHNICAL_BRIEF.md` — production use in lattice QCD (Exp 022)
- `docs/BENCHMARK_DATASHEET.md` — complete measurement dataset
- `DEPRECATED.md` — migration guide from C kernel module to Rust VFIO path

### Changed

- `akida-pcie-core.c` and related C files: marked deprecated. Kept at root
  for upstream reference; not part of the Rust build.

### Removed

- Dependency on Python SDK (MetaTF) — replaced by `akida-models` FlatBuffer parser
- Dependency on C++ libakida.so — replaced by direct VFIO + register access
- Dependency on kernel module for operation — VFIO backend requires no C code

---

## Origin — Brainchip-Inc/akida_dw_edma (master)

The original repository contained:
- `akida-pcie-core.c` — Linux PCIe driver wrapping DesignWare eDMA controller
- `install.sh` — kernel module build and load script
- `build_kernel_w_cma.sh` — custom kernel build for CMA support (AKD1500)

These files are preserved at the repository root unchanged.
