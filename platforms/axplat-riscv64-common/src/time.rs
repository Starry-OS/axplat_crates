#[cfg(feature = "rtc")]
use axplat::mem::VirtAddr;

/// RTC wall time offset in nanoseconds at monotonic time base.
static mut RTC_EPOCHOFFSET_NANOS: u64 = 0;

#[cfg(feature = "rtc")]
pub fn init_early(rtc_base: VirtAddr) {
    use axplat::time::{current_ticks, ticks_to_nanos};
    use riscv_goldfish::Rtc;

    if rtc_base.as_usize() != 0 {
        // Get the current time in microseconds since the epoch (1970-01-01) from the riscv RTC.
        // Subtract the timer ticks to get the actual time when ArceOS was booted.
        let epoch_time_nanos = Rtc::new(rtc_base.as_usize()).get_unix_timestamp() * 1_000_000_000;

        unsafe {
            RTC_EPOCHOFFSET_NANOS = epoch_time_nanos - ticks_to_nanos(current_ticks());
        }
    }
}

pub fn init_percpu() {
    #[cfg(feature = "irq")]
    sbi_rt::set_timer(0);
}

#[doc(hidden)]
pub fn current_ticks() -> u64 {
    riscv::register::time::read() as u64
}

#[doc(hidden)]
pub fn epochoffset_nanos() -> u64 {
    unsafe { RTC_EPOCHOFFSET_NANOS }
}

#[doc(hidden)]
pub fn set_oneshot_timer(deadline: u64) {
    sbi_rt::set_timer(deadline);
}

#[macro_export]
macro_rules! time_if_impl {
    ($name:ident, $freq:expr) => {
        struct $name;

        const NANOS_PER_TICK: u64 = axplat::time::NANOS_PER_SEC / $freq as u64;

        #[impl_plat_interface]
        impl axplat::time::TimeIf for $name {
            /// Returns the current clock time in hardware ticks.
            fn current_ticks() -> u64 {
                $crate::time::current_ticks()
            }

            /// Converts hardware ticks to nanoseconds.
            fn ticks_to_nanos(ticks: u64) -> u64 {
                ticks * NANOS_PER_TICK
            }

            /// Converts nanoseconds to hardware ticks.
            fn nanos_to_ticks(nanos: u64) -> u64 {
                nanos / NANOS_PER_TICK
            }

            /// Return epoch offset in nanoseconds (wall time offset to monotonic
            /// clock start).
            fn epochoffset_nanos() -> u64 {
                $crate::time::epochoffset_nanos()
            }

            /// Set a one-shot timer.
            ///
            /// A timer interrupt will be triggered at the specified monotonic time
            /// deadline (in nanoseconds).
            fn set_oneshot_timer(deadline_ns: u64) {
                $crate::time::set_oneshot_timer(Self::nanos_to_ticks(deadline_ns));
            }
        }
    };
}
