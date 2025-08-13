// Ref: https://elixir.bootlin.com/linux/v6.16/source/drivers/irqchip/irq-loongson-pch-pic.c

use axplat::mem::{PhysAddr, VirtAddr};

use crate::{config::devices::PLATIC_BASE, mem::phys_to_virt};

const PIC_COUNT_PER_REG: usize = 32;
const PIC_REG_COUNT: usize = 2;

const PCH_PIC_MASK: usize = 0x20;
const PCH_PIC_EDGE: usize = 0x60;
const PCH_PIC_POL: usize = 0x3e0;
const PCH_INT_HTVEC: usize = 0x200;

const MMIO_BASE: usize = phys_to_virt(PhysAddr::from_usize(PLATIC_BASE)).as_usize();

fn read_w(addr: usize) -> u32 {
    unsafe { ((MMIO_BASE + addr) as *mut u32).read_volatile() }
}
fn write_w(addr: usize, val: u32) {
    unsafe {
        ((MMIO_BASE + addr) as *mut u32).write_volatile(val);
    }
}

pub fn init() {
    // High level triggered
    for i in 0..PIC_REG_COUNT {
        write_w(PCH_PIC_EDGE, 0);
        write_w(PCH_PIC_POL, 0);
    }
}

fn split_bit(irq: usize) -> (usize, u32) {
    (irq / PIC_COUNT_PER_REG * 4, 1 << (irq % PIC_COUNT_PER_REG))
}

pub fn enable_irq(irq: usize) {
    let (offset, bit) = split_bit(irq);

    let addr = PCH_PIC_MASK + offset;
    write_w(addr, read_w(addr) & !bit);

    let addr = PCH_INT_HTVEC + irq;
    unsafe {
        ((MMIO_BASE + addr) as *mut u8).write_volatile(irq as _);
    }
}
pub fn disable_irq(irq: usize) {
    let (offset, bit) = split_bit(irq);
    let addr = PCH_PIC_MASK + offset;
    write_w(addr, read_w(addr) | bit);
}
