# Neuromorphic Coprocessor for Scientific Computing
## Technical Brief — AKD1000 Pure Rust Driver (rustChip)

**Date:** February 27, 2026
**Driver:** rustChip v0.1.0 — VFIO primary, kernel fallback
**Hardware:** BrainChip AKD1000 (PCIe x1 Gen2, BC.00.000.002)
**Repository:** https://github.com/ecoPrimal/rustChip
**License:** AGPL-3.0-or-later

---

## 1. Context: Where This Code Came From

This driver emerged from a production scientific computing pipeline that uses
the AKD1000 as a neuromorphic coprocessor for physics simulations. The pipeline
runs lattice QCD (SU(3) gauge theory), molecular dynamics, and transport
coefficient calculations on consumer hardware — GPU for heavy compute, NPU for
microsecond inference alongside the GPU without stealing compute cycles.

In building the integration, we discovered 10 cases where actual silicon behavior
differs from SDK documentation. We then built a pure Rust driver — bypassing the
Python SDK entirely — to access those capabilities directly.

**Experiment 022 results** (February 27, 2026 — the production run):
- 5,978 live AKD1000 inference calls over 24 hours
- 32⁴ lattice SU(3) β-scan with NPU adaptive steering
- 63% thermalization savings via NPU-guided early termination
- 80.4% HMC rejection prediction accuracy
- Susceptibility peak χ=32.41 at β_c=5.7797 (known: 5.692, error 0.4%)

This is not a toy benchmark. This is production physics.

---

## 2. Why a Rust Driver

### 2.1 The SDK Stack Is Too Heavy

The standard stack for AKD1000 use:
```
Python application
  → MetaTF (Python ML framework)
    → QuantizeML (quantization)
      → CNN2SNN (model compilation)
        → akida Python SDK
          → libakida.so (C++ engine)
            → akida_pcie.ko (C kernel module, requires per-kernel rebuild)
              → /dev/akida0
                → AKD1000 silicon
```

For a physics simulation that needs microsecond inference alongside a GPU,
this stack adds ~15% overhead, requires Python in the hot path, and binds
to a C++ ABI that varies between SDK versions.

### 2.2 The Rust Stack

```
Your Rust application
  → akida_driver::InferenceExecutor
    → VfioBackend (or KernelBackend fallback)
      → /dev/vfio/{IOMMU_GROUP}  (VFIO — no kernel module)
        → IOMMU → PCIe → AKD1000 silicon
```

One `cargo add akida-driver`. No Python. No C++. No kernel module required.

### 2.3 The AKD1500 Kernel Support Problem

The AKD1500 datasheet states: "Linux support ends at 6.8 with no planned
updates for newer kernels." The C kernel module requires a rebuild per kernel
version. Our VFIO backend requires no kernel module — it works on any kernel
with VFIO support (standard since Linux 3.6, 2012). Our queued Phase E is a
Rust kernel module using stable kernel Rust API bindings that don't break
between kernel versions.

---

## 3. Production Results

### 3.1 Lattice QCD Experiment 022

**Setup:** AMD Threadripper 3970X + NVIDIA RTX 3090 (GPU) + AKD1000 (NPU)
**Runtime:** 24 hours, 10 β-points, 5,900 measurement trajectories
**NPU model:** ESN readout — InputConv(1,1,50→128) → FC(128→1)

| Metric | Value |
|--------|-------|
| NPU calls | 5,978 |
| Inference latency | 54 µs (18,500 Hz) |
| Batch=8 throughput | 390 µs/sample (20,700 /s) |
| Weight swap (3 classifiers) | 86 µs |
| Thermalization savings | 63% (2.4h of 3.8h budget) |
| Rejection prediction | 80.4% accuracy |
| Energy per inference | 1.4 µJ |
| NPU overhead of GPU run | 0.003% |

### 3.2 Three-Substrate Architecture

```
┌─────────────────────────────────────────────────────┐
│ GPU: SU(3) HMC (RTX 3090, 24 GB)                   │
│   DF64 force calc → leapfrog → Metropolis           │
│   7.6 s/trajectory, 338W, 100% utilization          │
│   Output: (β, ⟨P⟩, ⟨|L|⟩) per configuration        │
└──────────────────────┬──────────────────────────────┘
                       │ PCIe (feature vector, ~200 bytes)
                       ▼
┌─────────────────────────────────────────────────────┐
│ NPU: ESN adaptive steering (AKD1000, ~30 mW chip)  │
│   Input: lattice observables (β, ⟨P⟩, ⟨|L|⟩)       │
│   Output: phase label + thermalization signal        │
│   54 µs per call. Batch=8 for throughput.           │
│   Zero GPU cycles stolen.                           │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
┌─────────────────────────────────────────────────────┐
│ CPU: Orchestration (Threadripper 3970X)            │
│   Steering decisions, anomaly detection             │
│   0.09% of wall time                               │
└─────────────────────────────────────────────────────┘
```

---

## 4. The 10 Hardware Discoveries

Full methodology in `../../docs/BEYOND_SDK.md`. Summary:

| # | SDK Claim | Silicon Reality | Confirmed |
|---|-----------|-----------------|-----------|
| 1 | InputConv: 1 or 3 channels only | Any channel count (1–64 tested) | Hardware |
| 2 | FC layers run independently | All FC layers merge via SkipDMA | Latency measurement |
| 3 | Batch=1 inference only | Batch=8 → 2.4× throughput | Timing sweep |
| 4 | One clock mode | 3 modes: Performance / Economy / LowPower | sysfs attribute |
| 5 | Max FC width ~hundreds | Tested to 8192+ neurons (SRAM-limited only) | Model compile + run |
| 6 | No direct weight mutation | set_variable() updates live (~14 ms) | Linearity test |
| 7 | "30 mW" chip power | Board floor 918 mW; chip below measurement noise | hwmon measurement |
| 8 | 8 MB SRAM limit | BAR1 exposes 16 GB address decode space | sysfs resource file |
| 9 | Program binary is opaque | FlatBuffer: program_info + program_data; weights via DMA | program_external() |
| 10 | Simple inference engine | SkipDMA, 51-bit threshold SRAM, 3 hardware variants | C++ symbol analysis |

**Discovery 2 (SkipDMA)** is architecturally important: deep FC chains execute
as a single hardware pass because NP-to-NP routing bypasses PCIe entirely.
8 layers costs only 3 µs more than 2 layers. Deep physics embedding networks
are effectively free once the PCIe transfer is paid.

**Discovery 9 (FlatBuffer format)** enables `program_external()` injection —
feeding hand-crafted program binaries directly to the hardware, bypassing
the entire SDK compilation pipeline. This is the foundation of `akida-models`.

---

## 5. Driver Architecture

### 5.1 Crate Map

```
akida-chip     — silicon model: register map, BAR layout, NP mesh (no deps)
akida-driver   — full driver: VFIO + kernel fallback + inference API
akida-models   — FlatBuffer parser + program_external() injection
akida-bench    — 10 benchmark binaries (one per BEYOND_SDK discovery)
akida-cli      — akida enumerate | info | bind-vfio | unbind-vfio
```

### 5.2 VFIO Backend (Primary)

No kernel module required. Linux VFIO/IOMMU maps the PCIe device into
userspace. The driver performs:

1. `VFIO_GROUP_GET_DEVICE_FD` → device file descriptor
2. `VFIO_DEVICE_GET_REGION_INFO` → BAR0 size and offset
3. `mmap()` on VFIO region → BAR0 control registers accessible as `*mut u32`
4. `alloc_zeroed()` + `mlock()` → pinned DMA buffer
5. `VFIO_IOMMU_MAP_DMA` → IOVA (device-visible address)
6. Write IOVA to `MODEL_ADDR_LO/HI`, `INPUT_ADDR_LO/HI` registers
7. Write 1 to `MODEL_LOAD` or `INFER_START`
8. Poll `STATUS` or `INFER_STATUS` register
9. Read result from pinned output buffer

All of this is in `crates/akida-driver/src/vfio/mod.rs`. Total unsafe surface:
VFIO ioctls (kernel-specific, not covered by rustix), `mmap()`, `mlock()`,
`alloc_zeroed()`. Every unsafe block has documented invariants.

### 5.3 Setup

```bash
# One-time IOMMU + vfio-pci bind (requires root once)
echo "intel_iommu=on iommu=pt" >> /etc/default/grub
sudo modprobe vfio-pci
sudo cargo run --bin akida -- bind-vfio 0000:a1:00.0
sudo chown $USER /dev/vfio/$(akida iommu-group 0000:a1:00.0)

# After setup — no root required
akida enumerate        # lists devices + capabilities
akida info 0           # detailed info including IOMMU group
cargo run --bin bench_latency   # 54 µs reference benchmark
```

---

## 6. GPU + NPU Co-location

The interface between GPU compute and NPU inference is `&[f32]` — a
CPU-resident float slice. The GPU side lives in your project; the NPU
side is here.

```rust
// GPU produces feature vector (your GPU code — wgpu, vulkano, ash, cuda)
let features: Vec<f32> = your_gpu_compute();

// NPU inference (this repo — standalone, no GPU dep)
let mut exec = akida_driver::InferenceExecutor::new(
    DeviceManager::discover()?.open_first()?
);
let result = exec.run(&features, Default::default())?;
```

**Latency budget for GPU→NPU pipeline:**
- GPU compute (your shader): variable
- GPU→CPU readback: ~1–10 µs (pinned memory)
- CPU→NPU DMA: ~14 µs for 512-float vector at 37 MB/s
- NPU inference: 54–390 µs depending on model + batch
- NPU→CPU: ~5 µs
- **Total NPU overhead: 70–430 µs** vs 7,000,000+ µs GPU trajectory

The NPU runs as an independent PCIe device. Zero GPU cycles are stolen.
Both substrates operate in parallel.

**Future: P2P DMA (GPU BAR → NPU IOVA directly)**

Linux VFIO supports peer-to-peer IOMMU mapping in principle. With both
devices in the same IOMMU group and VFIO peer-mapping enabled, GPU output
could DMA directly to NPU input without CPU copy. This would reduce latency
by ~15 µs and eliminate one memory copy. See
`../../explorations/GPU_NPU_PCIE.md` for the full analysis.

---

## 7. What BrainChip Could Open Up

Collaborative improvements that would accelerate the Rust stack:

| What | Why | Effort |
|------|-----|--------|
| DW eDMA register offset table | The eDMA descriptor layout is in the DesignWare databook; the offsets in `specs/SILICON_SPEC.md` are inferred | Low |
| Confirm/correct `inferred` register entries | 8 entries in `specs/SILICON_SPEC.md` labeled `inferred` | Low |
| AKD1500 hardware sample | 10 register map entries are AKD1500-specific; currently extrapolated | Medium |
| Akida IP licensing inquiry | Die-to-die integration (AKD1500 + GPU on same MCM) analysis in `../explorations/` | Long term |
| On-chip learning register path | Phase F: reservoir update on-chip, bypassing PCIe weight upload | Medium |

None of these are blockers. The current driver works on AKD1000 in production.
The items above would improve precision and enable AKD1500 validation.

---

## 8. Roadmap

| Phase | Status | Description |
|-------|--------|-------------|
| A | Complete | Python SDK → Rust FFI |
| B | Complete | C++ engine symbol analysis, `program_external()` |
| C | Complete | Direct `/dev/akida0` ioctl/mmap |
| D | **Active** | Pure Rust VFIO driver (this repo, primary path) |
| E | Queued | Rust `akida_pcie` kernel module (stable kernel Rust API) |

Phase E directly addresses the AKD1500 "no plans past Linux 6.8" problem.

---

## 9. References

- `../../docs/BEYOND_SDK.md` — 10 hardware discoveries with methodology
- `../../docs/HARDWARE.md` — NP mesh architecture, BAR layout, register map
- `../../docs/BENCHMARK_DATASHEET.md` — complete measurement dataset
- `../../specs/SILICON_SPEC.md` — full register map with confirmed/inferred labels
- `../../specs/DRIVER_SPEC.md` — driver architecture and API contract
- `../../specs/PHASE_ROADMAP.md` — Phase A–E sovereign driver roadmap
- `../../whitePaper/explorations/GPU_NPU_PCIE.md` — GPU+NPU P2P analysis
- `../../whitePaper/explorations/RUST_AT_SILICON.md` — long-term Rust vision
