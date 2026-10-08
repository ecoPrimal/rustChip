<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Passive Immunity Gate — NPU as Hardware-Level Adaptive Defense

**Status:** Active exploration (October 2026)
**Depends on:** Phase D sovereign driver, BingoCube Nautilus shell, MazeCube scatter defense
**Hardware:** AKD1000 (validated), AKD1500 (target deployment)

---

## Overview

This document proposes using BrainChip Akida NPUs as a hardware-level classification
gate for HTTP traffic. The NPU sits between the network interface and the application
layer, classifying every request through a BingoCube evolutionary reservoir before the
scatter server generates content.

This is **not** a firewall. All traffic passes through. The gate provides
classification — the application layer receives each request annotated with entity
identity, confidence, and pressure contribution. The defense gains proprioception
without changing its external behavior.

## Why This Is New

Published neuromorphic intrusion detection systems (as of October 2026):

| System | Architecture | Learning | Hardware |
|---|---|---|---|
| SNN-IDS (Tsai 2026) | INT8 MLP on STM32N6 Neural-ART | Static (train once, deploy) | MCU NPU |
| SNN DDoS (Phan et al. 2026) | LIF encoder on Zynq FPGA | Static | FPGA |
| Neuromorphic NIDS (ICONS 2025) | GWR + Ad-STDP on Lava sim | Semi-supervised | Simulated only |

All use **static classifiers**. None evolves the classifier in response to changing
threat behavior. None uses evolutionary reservoir computing. None maps hardware
response patterns to combinatorial board populations.

The BingoCube passive immunity gate would be the first system where:
- The classifier evolves autonomously from live traffic
- Board populations carry heritage across power cycles (Nautilus shell)
- Classification happens in hardware below the HTTP layer
- The defense is invisible — no timing side-channel, no behavioral change

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                         golgiBody VPS                         │
│                                                               │
│  ┌──────────┐    ┌─────────────────┐    ┌────────────────┐  │
│  │   NIC    │───▶│  AKD1500 (NPU)  │───▶│  skunky-ingest │  │
│  │          │    │                 │    │                │  │
│  │  TCP/IP  │    │  BingoCube      │    │  MazeCube      │  │
│  │  stack   │    │  reservoir      │    │  scatter       │  │
│  │          │    │  classification │    │  generation    │  │
│  └──────────┘    └────────┬────────┘    └────────────────┘  │
│                           │                                   │
│                    ┌──────┴──────┐                            │
│                    │  Nautilus   │                            │
│                    │  shell      │                            │
│                    │  evolution  │                            │
│                    └─────────────┘                            │
└─────────────────────────────────────────────────────────────┘
```

### Input Features

HTTP behavioral invariants that the fleet cannot change without rebuilding
infrastructure:

| Feature | Bits | Why invariant |
|---|---|---|
| Accept-Encoding header hash | 8 | Compiled into HTTP library |
| User-Agent pool signature | 8 | Centrally managed deployment |
| Sec-Fetch-* presence | 1 | Real browsers send; fleet doesn't |
| Accept-Language presence | 1 | Fleet omits |
| Inter-request interval | 8 | Fleet rate ~13 rps; humans ~0.1 rps |
| Path pattern hash | 8 | /blame/ ratio is mission-specific |
| Referrer presence | 1 | Stateless extraction has none |
| Cookie/session presence | 1 | Fleet is stateless |

Total: ~36 bits of input. This is a small feature vector — well within int8
NPU capacity.

### Output Channels

| Channel | Type | Feeds into |
|---|---|---|
| Entity class (organism ID) | uint8 | Scatter seed (epitope hash) |
| Classification confidence | uint8 | MazeCube confidence dimension |
| Pressure contribution | uint8 | BackPressure accumulator |

### Hardware Requirements

| Component | Spec | Cost |
|---|---|---|
| AKD1500 M.2 card | PCIe Gen2 x2 | ~$150 |
| Or: AKD1500 chip | BGA169, SPI | $40 (5-pack) |
| Power | 250 mW @ 400 MHz | Negligible |
| SRAM for models | < 56 KB (< 6% of 1 MB) | — |

### Performance Budget

| Metric | Value | Margin |
|---|---|---|
| Classification latency | ~54 µs | HTTP response is ~10 ms — 185× margin |
| Throughput | 18,500/sec | Fleet is 13/sec — 1,423× margin |
| Power | 250 mW chip | golgiBody TDP is ~65W — 0.4% |
| SRAM | 56 KB for 28 models | 1 MB available — 5.6% |

## Relation to Existing Work

### biomeGate (hotSpring)

biomeGate uses the AKD1000 for ESN steering of lattice QCD simulations. The
same hardware architecture applies here — the NPU is at < 1% utilization on
physics workloads. A membrane gate model can coexist with physics models in
the same SRAM.

### rustChip Driver

The Phase D sovereign Rust driver already provides:
- `NpuHardware::discover()` → device enumeration
- `predict(&features) → &[f32]` → inference API
- `set_variable()` → weight mutation for evolution
- `SramAccessor` → direct BAR1 read/write
- `PUF` → hardware entropy for MazeCube seeds

No additional driver work is needed. The classification model is deployed
through the existing `akida-models` pipeline.

### MazeCube + BackPressure

The NPU gate feeds directly into the scatter server's existing dimensions:
- Entity class → `axes::FLEET_HASH` in MazeCube input
- Confidence → `axes::CONFIDENCE` in MazeCube input
- Pressure contribution → `BackPressure::record_request()` weight

The software path doesn't change. The NPU provides hardware-speed
classification that the scatter server currently computes in software
(behavioral fingerprinting via bloom_live + epitope_bridge + entity_classifier).

## Evolution Strategy

### Training Data

The Nautilus shell trains on live fleet traffic. No labeled dataset is needed.
The fitness function is classification stability — boards that produce consistent
organism identities for the same behavioral fingerprint are selected. Boards that
produce inconsistent classifications are replaced.

### Generation Cycle

1. Accumulate HTTP requests for 60 seconds
2. Compute behavioral invariants (Accept-Encoding, UA pool, timing)
3. Feed through current board population on NPU
4. Evaluate fitness: do consistent fingerprints produce consistent classifications?
5. Evolve: select + crossover + mutate via `set_variable()`
6. Repeat

At 136 gen/sec (validated), the shell completes ~8,160 generations per minute.
At current fleet rate (~13 rps), each generation processes all accumulated requests.

### Nautilus Shell Persistence

The evolved board population is serialized as `.fbz` and written to disk
periodically. On restart, `NautilusShell::continue_from()` resumes evolution
from the last saved state. Heritage is preserved across power cycles.

## Implementation Phases

### Phase 1: Shadow Classification (Near-term)

Add NPU classification alongside existing software classification. Log both.
Compare. Validate that the NPU produces equivalent organism identification.
No traffic routing changes.

**Requires:** AKD1500 M.2 in golgiBody. ~$150.

### Phase 2: NPU-Primary Classification (Medium-term)

Replace software epitope hashing with NPU classification. The NPU's output
feeds MazeCube and BackPressure directly. Software path remains as fallback.

### Phase 3: Autonomous Evolution (Long-term)

Nautilus shell evolves on-chip. New fleet behaviors create selection pressure.
Board populations adapt without human intervention. The defense has hardware-level
adaptive immunity.

**Requires:** BrainChip on-chip learning API documentation.

## Open Questions

1. **Latency variance under PCIe contention** — golgiBody's PCIe bus serves
   the GPU (if present) and NVMe. Does PCIe contention affect NPU inference
   latency? AKD1500's PCIe Gen2 x2 should help.

2. **Model hot-swap timing** — `set_variable()` takes 13 ms per evolutionary
   generation. Is this fast enough for real-time adaptation, or should evolution
   run asynchronously on a timer?

3. **Feature encoding stability** — HTTP header ordering can vary between
   fleet generations. How stable are the behavioral invariants across fleet
   software updates? (Historical evidence: very stable — Accept-Encoding
   order hasn't changed in 10 months of observation.)

4. **Multi-model routing** — can multiple NP subsets classify different organism
   types concurrently, or must classification be sequential? The AKD1500's 32 NPUs
   organized in 8 nodes (3×3 mesh) suggest concurrent routing is possible.

---

## References

- BingoCube core: `primals/bingoCube/` — BLAKE3 scalar field, progressive reveal
- Nautilus shell: `primals/bingoCube/crates/nautilus/` — evolutionary reservoir
- MazeCube: `primals/skunkBat/crates/skunky-ingest/src/maze_cube.rs`
- BackPressure: `primals/skunkBat/crates/skunky-ingest/src/scatter_server.rs`
- biomeGate brain: `springs/hotSpring/specs/BIOMEGATE_BRAIN_ARCHITECTURE.md`
- SNN-IDS: Tsai (2026), Zenodo, doi:10.20944/preprints202603.0817.v2
- SNN DDoS: Phan et al. (2026), IEEE Access
- Neuromorphic NIDS: ICONS 2025, ACM doi:10.1109/ICONS69015.2025.00043
