use plic::{Mode, PLIC};

use crate::config::{
    devices::PLIC_PADDR,
    plat::{CPU_NUM, PHYS_VIRT_OFFSET},
};

static PLIC: PLIC<CPU_NUM> = unsafe { PLIC::new(PHYS_VIRT_OFFSET + PLIC_PADDR, [2; CPU_NUM]) };

pub fn init_plic() {
    for hart in 0..(CPU_NUM as u32) {
        PLIC.set_threshold(hart, Mode::Supervisor, 0);
    }
}

axplat_riscv64_common::irq_if_impl!(IrqIfImpl, PLIC);
