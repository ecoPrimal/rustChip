# AI Developer Context

This file is the entry point for AI coding assistants and new developers.
Read this before touching any code.

---

## What this project is

`rustChip` is a **standalone, pure Rust driver and benchmark suite** for
BrainChip Akida neuromorphic processors (AKD1000, AKD1500). It has
**no runtime dependencies** on any other ecoPrimals project (toadStool,
hotSpring, wetSpring). When you clone this repository and run `cargo build`,
you get a fully functional system.

This is intentional. The project is designed to be handed to BrainChip's
engineering team as a complete, self-contained artifact.

---

## Crate graph

```
akida-chip                  ← no deps (pure silicon model)
    ↑
akida-driver                ← depends on akida-chip + rustix + libc (VFIO ioctls)
    ↑               ↑
akida-models      akida-bench     akida-cli
(FlatBuffer)      (10 benchmarks) (CLI tool)
```

Do not create circular dependencies. `akida-chip` must remain zero-dependency.

---

## Naming conventions

| Convention | Rule |
|------------|------|
| Crate names | `akida-{noun}` — kebab-case |
| Module names | `snake_case` |
| Hardware constants | `SCREAMING_SNAKE_CASE` |
| `confirmed` labels | Any constant measured directly from hardware |
| `inferred` labels | Consistent with behavior, not directly read |
| `hypothetical` labels | Assumed from spec/databook, unverified |

Do not remove or downgrade `confirmed` / `inferred` / `hypothetical` labels.
They are the provenance record for the register map.

---

## Hardware discovery rules

**Never hardcode a device path.** All of these are wrong:

```rust
// WRONG
let dev = File::open("/dev/akida0")?;
let addr = "0000:a1:00.0";
let group = 5u32;
```

Always use `DeviceManager::discover()` which scans sysfs at runtime:

```rust
// CORRECT
let mgr = DeviceManager::discover()?;
let dev = mgr.open_first()?;
```

The only acceptable hardcoded values are PCIe vendor/device IDs in
`akida-chip/src/pcie.rs` — those are silicon constants, not configuration.

---

## Safety rules

`akida-chip` has `#![forbid(unsafe_code)]`. Keep it that way.

`akida-driver` has unsafe code in exactly one place: `src/vfio/mod.rs`.
Every unsafe block must have:
1. A comment explaining **why** unsafe is necessary (what kernel API requires it)
2. **Invariants** the code maintains
3. **Caller guarantees** needed

Do not add unsafe code outside `vfio/mod.rs` without a documented reason.

---

## Error handling

All public functions return `Result<T, AkidaError>`. Never use `.unwrap()`
or `.expect()` in library code. In binaries (bench/cli), `?` with `anyhow`
is acceptable.

When adding a new error case, add a variant to `AkidaError` in
`src/error.rs` — don't use `anyhow::Error` in the library crate.

---

## Testing philosophy

Hardware may not be present. Tests that require hardware must:
1. Call `DeviceManager::discover()` at the start
2. Skip gracefully if zero devices found:

```rust
#[test]
fn test_needs_hardware() {
    let mgr = DeviceManager::discover().expect("discover should not fail");
    if mgr.device_count() == 0 {
        eprintln!("No Akida hardware — skipping");
        return;
    }
    // ... hardware test ...
}
```

Unit tests in `akida-chip` require no hardware (pure constants and math).

---

## Benchmark philosophy

Each binary in `akida-bench/src/bin/` reproduces exactly one BEYOND_SDK
discovery. The file header must state:
- Which discovery it reproduces (e.g. "Discovery 4 from BEYOND_SDK.md")
- The reference measurement
- What SDK claim is being overturned

Reference measurements are **not** test assertions — hardware will vary.
Print the measured value and compare to reference for human judgment.

---

## Documentation philosophy

Every public item needs a doc comment. For hardware-facing constants:

```rust
/// Optimal batch size for PCIe amortisation (Discovery 3).
///
/// `batch=8` gives 2.4× throughput over `batch=1` by spreading the
/// ~650 µs PCIe round-trip cost across 8 inference samples.
/// Source: BEYOND_SDK.md Discovery 3, Feb 2026.
pub const OPTIMAL_BATCH_SIZE: usize = 8;
```

For implementation notes that explain hardware behavior (not just the API):
use `//!` module-level docs or inline `//` comments. Do not write comments
that just describe the code — only document **why**, not **what**.

---

## Extension patterns

### Adding a new backend

1. Create `src/backends/{name}.rs`
2. Implement `NpuBackend` trait from `src/backend.rs`
3. Add variant to `BackendType` enum
4. Add `BackendSelection::{Name}` variant
5. Add arm to `select_backend()` in `src/backend.rs`
6. Export from `src/backends/mod.rs`

### Adding a new benchmark

1. Create `crates/akida-bench/src/bin/{name}.rs`
2. Add `[[bin]]` entry to `crates/akida-bench/Cargo.toml`
3. File header must cite the BEYOND_SDK.md discovery it reproduces
4. Print reference measurement, measured value, and comparison

### Adding a new hardware constant

1. Determine if it belongs in `akida-chip` (silicon model) or `akida-driver` (driver behavior)
2. Add to appropriate module with `confirmed`/`inferred`/`hypothetical` label
3. Add test in `#[cfg(test)]` block validating the constant value

### Supporting AKD1500

The only required changes:
1. `pcie.rs`: `AKD1500 = 0xA500` is already there
2. `bar.rs`: Verify BAR sizes (PCIe x2 may change layout)
3. `mesh.rs`: AKD1500 has different NP count/topology
4. `capabilities.rs`: Handle additional GPIO/SPI capabilities in sysfs

---

## What NOT to do

- Do not add a dependency on `toadstool`, `barracuda`, `hotspring`, or any
  other ecoPrimals project. This repo must be standalone.
- Do not add Python bindings. If a Python consumer wants to use this,
  they can call `akida-cli` as a subprocess.
- Do not implement model training or weight optimization. This is an
  inference driver. Training belongs in the scientific computing projects.
- Do not assume the C kernel module is present. All code paths must work
  without it (using VFIO backend as fallback).
- Do not add tokio as a required (non-feature) dependency. The `async` feature
  gate exists for a reason.
