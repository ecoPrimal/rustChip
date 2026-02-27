# rustChip White Paper

Scientific write-ups, outreach materials, and exploratory analyses.

---

## Contents

| Directory | Contents |
|-----------|----------|
| [`outreach/akida/`](outreach/akida/) | Technical brief and benchmark datasheet for BrainChip team |
| [`explorations/`](explorations/) | Deep technical analyses of specific questions |

---

## Explorations Index

| Document | Question answered |
|----------|------------------|
| [`explorations/GPU_NPU_PCIE.md`](explorations/GPU_NPU_PCIE.md) | How does GPU+NPU co-location work over PCIe? Can data go directly from GPU BAR to NPU? |
| [`explorations/RUST_AT_SILICON.md`](explorations/RUST_AT_SILICON.md) | What would it take to go full Rust from application to silicon? What's the timeline? |
| [`explorations/VFIO_VS_KMOD.md`](explorations/VFIO_VS_KMOD.md) | VFIO userspace driver vs C kernel module — tradeoffs and migration strategy |

---

## Outreach

`outreach/akida/` is the standing technical brief directed at BrainChip's
engineering team. It documents:
- What we measured on their hardware
- What SDK assumptions were overturned by direct probing
- Why a Rust driver exists and what it achieves
- What collaboration would accelerate

This is not a sales pitch. It is a technical report from an independent
team running real physics workloads on real silicon, with all data published
under AGPL-3.0.

---

## Relationship to docs/

`docs/` contains the raw measurement documents (BEYOND_SDK.md, HARDWARE.md,
TECHNICAL_BRIEF.md, BENCHMARK_DATASHEET.md) — the ground truth.

`whitePaper/` contains derived analyses and outreach materials that build
on that ground truth.

When `docs/` is updated, relevant `whitePaper/` documents should be revisited.
