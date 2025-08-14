/// Bootstraps the given CPU core with the given initial stack (in physical
/// address).
///
/// Where `cpu_id` is the logical CPU ID (0, 1, ..., N-1, N is the number of
/// CPU cores on the platform).
pub fn cpu_boot(cpu_id: usize, entry_point: usize, arg: usize) {
    if sbi_rt::probe_extension(sbi_rt::Hsm).is_unavailable() {
        warn!("HSM SBI extension is not supported for current SEE.");
        return;
    }
    sbi_rt::hart_start(cpu_id, entry_point, arg);
}

/// Shutdown the whole system.
pub fn system_off() -> ! {
    info!("Shutting down...");
    sbi_rt::system_reset(sbi_rt::Shutdown, sbi_rt::NoReason);
    warn!("It should shutdown!");
    loop {
        axcpu::asm::halt();
    }
}
