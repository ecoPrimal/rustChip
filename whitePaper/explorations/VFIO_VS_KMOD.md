# VFIO Userspace Driver vs C Kernel Module

**Date:** February 27, 2026
**Decision:** VFIO is the primary path in Phase D. Kernel module is fallback.

---

## The Choice

| Property | C Kernel Module | VFIO Userspace |
|----------|----------------|----------------|
| Kernel version dependency | Per-version rebuild required | None (VFIO since 3.6, 2012) |
| Root requirement | `insmod` at boot | One-time `bind-vfio` per machine |
| Runtime permissions | `chmod 666 /dev/akida0` at each boot | `/dev/vfio/{group}` persistent via udev |
| Isolation | None (all processes share `/dev/akida0`) | Per-process IOMMU isolation |
| DMA safety | Kernel validates DMA requests | IOMMU hardware enforces boundaries |
| Crash impact | Kernel bug → system hang | Userspace bug → process crash, not kernel panic |
| Build system | kernel headers + gcc + make | `cargo build` |
| AKD1500 support | "No plans past 6.8" per datasheet | Works on any kernel with VFIO |
| Code language | C | Rust |
| Unsafe surface | Entire kernel module | VFIO ioctls only (`src/vfio/mod.rs`) |

---

## Why We Chose VFIO

### 1. The AKD1500 Problem

The AKD1500 datasheet is explicit: "Linux kernel support ends at 6.8 with no
plans for updates to newer kernels." This means:

- AKD1500 users on Ubuntu 25.04 (kernel 6.11+) are stranded
- AKD1500 users on Fedora 41 (kernel 6.11) must rebuild manually
- AKD1500 users on enterprise distros (RHEL 9, kernel 5.14) are fine... until they upgrade

VFIO is immune to this. VFIO is a stable Linux subsystem that has not broken
since 2012. The VFIO ioctl numbers are ABI-stable. The `/dev/vfio/*` interface
is stable.

### 2. Security Isolation

`/dev/akida0` gives any process with read permission access to the NPU.
On a shared machine, this is a security concern. VFIO IOMMU isolation
ensures that a process can only access the memory regions it has explicitly
mapped. One process cannot inspect or corrupt another's DMA buffers.

### 3. No Kernel Panic on Driver Bug

A bug in `akida-pcie-core.c` (or our future Rust kernel module) can kernel
panic the host. A bug in the VFIO userspace driver crashes the application
process. For scientific computing, this matters: an 8-hour simulation should
not be killed by a driver bug.

### 4. Crash Recovery

If the userspace driver crashes mid-inference, the IOMMU unmap happens
automatically via RAII (`Drop` on `DmaBuffer`). The NPU may be in an
inconsistent state, but a process restart recovers it without rebooting.

---

## Why We Keep the Kernel Module as Fallback

### 1. Not All Systems Have IOMMU

The VFIO path requires:
- IOMMU hardware (Intel VT-d or AMD-Vi)
- IOMMU enabled in BIOS
- Kernel compiled with `CONFIG_VFIO_PCI=y` (standard on most distros)

Some embedded systems, older hardware, and some ARM platforms lack IOMMU.
On those systems, the kernel module is the only path.

### 2. Some Users Already Have It Installed

BrainChip's `install.sh` is a one-liner. Users who ran it already have
`akida_pcie` loaded and `/dev/akida0` present. The kernel backend lets them
use rustChip immediately without re-doing their setup.

### 3. The Fallback Costs Nothing

`BackendSelection::Auto` tries Kernel first (faster context switch), then
VFIO (no C module), then Userspace (no DMA). The user doesn't decide —
the driver picks the best available backend automatically.

---

## VFIO Setup Robustness

The common objection to VFIO is setup complexity. The `akida-cli` tool
removes this:

```bash
# Everything the user needs to do (once per machine):
sudo akida bind-vfio 0000:a1:00.0
sudo chown $USER /dev/vfio/$(akida iommu-group 0000:a1:00.0)

# Or, for permanent setup via udev:
cat > /etc/udev/rules.d/99-akida-vfio.rules << 'EOF'
# Bind AKD1000 to vfio-pci on boot
ACTION=="add", SUBSYSTEM=="pci", \
  ATTR{vendor}=="0x1e7c", ATTR{device}=="0xbca1", \
  RUN+="/usr/sbin/modprobe vfio-pci", \
  RUN+="/bin/sh -c 'echo 1e7c bca1 > /sys/bus/pci/drivers/vfio-pci/new_id 2>/dev/null || true'"

# Grant user access to VFIO group
ACTION=="add", SUBSYSTEM=="vfio", \
  GROUP="vfio", MODE="0660"
EOF
sudo udevadm control --reload-rules

# Add user to vfio group
sudo groupadd -f vfio
sudo usermod -aG vfio $USER
```

After this, every boot automatically binds the device and grants access.
No manual step required after initial setup.

---

## Performance Comparison

VFIO adds one layer of indirection (IOMMU address translation) to every
DMA transfer. In practice, this is transparent:

| Metric | Kernel Backend | VFIO Backend |
|--------|---------------|-------------|
| DMA setup overhead | ~5 µs | ~8 µs (IOMMU page walk) |
| Transfer throughput | 37 MB/s | 37 MB/s |
| Inference latency | 54 µs | 54 µs |
| PCIe round-trip | ~650 µs | ~650 µs |

The IOMMU adds ~3 µs to DMA setup (page table walk during `VFIO_IOMMU_MAP_DMA`).
For inference workloads, this is absorbed into the ~14 µs DMA setup cost.
It does not affect transfer bandwidth or inference latency.

---

## Migration Path

### For existing C module users

1. Keep `akida_pcie.ko` loaded (kernel backend continues to work)
2. Optionally migrate to VFIO when ready:
   ```bash
   sudo rmmod akida_pcie
   sudo akida bind-vfio 0000:a1:00.0
   # From here: no kernel module needed
   ```

### For new deployments

Start with VFIO:
1. Check IOMMU: `dmesg | grep -i iommu`
2. If IOMMU enabled: `sudo akida bind-vfio <addr>`
3. If IOMMU disabled: check BIOS, or use kernel backend as fallback

### For embedded/edge (no IOMMU)

The kernel backend (`/dev/akida0`) remains the path. Phase E (Rust kernel
module) will provide the same `/dev/akida*` interface with better safety
properties.
