use axplat::mem::{MemIf, PAGE_SIZE_4K, PhysAddr, RawRange, VirtAddr, pa, va};

use crate::config::devices::MMIO_RANGES;
use crate::config::plat::{PHYS_MEMORY_BASE, PHYS_MEMORY_SIZE, PHYS_VIRT_OFFSET};

struct MemIfImpl;

#[allow(dead_code)]
pub const fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
    va!(paddr.as_usize() + PHYS_VIRT_OFFSET)
}

#[impl_plat_interface]
impl MemIf for MemIfImpl {
    /// Returns all physical memory (RAM) ranges on the platform.
    ///
    /// All memory ranges except reserved ranges (including the kernel loaded
    /// range) are free for allocation.
    fn phys_ram_ranges() -> &'static [RawRange] {
        // 256 MiB
        const LOWRAM_SIZE: usize = 0x1000_0000;
        if PHYS_MEMORY_SIZE <= LOWRAM_SIZE {
            &[(PAGE_SIZE_4K, PHYS_MEMORY_SIZE - PAGE_SIZE_4K)]
        } else {
            &[
                (PAGE_SIZE_4K, LOWRAM_SIZE - PAGE_SIZE_4K),
                (PHYS_MEMORY_BASE, PHYS_MEMORY_SIZE - LOWRAM_SIZE),
            ]
        }
    }

    /// Returns all reserved physical memory ranges on the platform.
    ///
    /// Reserved memory can be contained in [`phys_ram_ranges`], they are not
    /// allocatable but should be mapped to kernel's address space.
    ///
    /// Note that the ranges returned should not include the range where the
    /// kernel is loaded.
    fn reserved_phys_ram_ranges() -> &'static [RawRange] {
        &[]
    }

    /// Returns all device memory (MMIO) ranges on the platform.
    fn mmio_ranges() -> &'static [RawRange] {
        &MMIO_RANGES
    }

    /// Translates a physical address to a virtual address.
    fn phys_to_virt(paddr: PhysAddr) -> VirtAddr {
        phys_to_virt(paddr)
    }

    /// Translates a virtual address to a physical address.
    fn virt_to_phys(vaddr: VirtAddr) -> PhysAddr {
        pa!(vaddr.as_usize() - PHYS_VIRT_OFFSET)
    }
}
