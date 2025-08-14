use core::sync::atomic::{AtomicPtr, Ordering};

use axplat::irq::{HandlerTable, IrqHandler};
use plic::{Mode, PLIC};
use riscv::register::sie;

/// `Interrupt` bit in `scause`
const INTC_IRQ_BASE: usize = 1 << (usize::BITS - 1);

/// Supervisor software interrupt in `scause`
#[allow(unused)]
const S_SOFT: usize = INTC_IRQ_BASE + 1;

/// Supervisor timer interrupt in `scause`
const S_TIMER: usize = INTC_IRQ_BASE + 5;

/// Supervisor external interrupt in `scause`
const S_EXT: usize = INTC_IRQ_BASE + 9;

static TIMER_HANDLER: AtomicPtr<()> = AtomicPtr::new(core::ptr::null_mut());

/// The maximum number of IRQs.
const MAX_IRQ_COUNT: usize = 1024;

static IRQ_HANDLER_TABLE: HandlerTable<MAX_IRQ_COUNT> = HandlerTable::new();

macro_rules! with_cause {
    ($cause: expr, @S_TIMER => $timer_op: expr, @S_EXT => $ext_op: expr, @EX_IRQ => $plic_op: expr $(,)?) => {
        match $cause {
            S_TIMER => $timer_op,
            S_EXT => $ext_op,
            other => {
                if other & INTC_IRQ_BASE == 0 {
                    // Device-side interrupts read from PLIC
                    $plic_op
                } else {
                    // Other CPU-side interrupts
                    panic!("Unknown IRQ cause: {}", other);
                }
            }
        }
    };
}

pub fn init_percpu() {
    // enable soft interrupts, timer interrupts, and external interrupts
    unsafe {
        sie::set_ssoft();
        sie::set_stimer();
        sie::set_sext();
    }
}

#[doc(hidden)]
pub fn set_enable<const H: usize>(plic: &PLIC<H>, irq: usize, enabled: bool) {
    with_cause!(
        irq,
        @S_TIMER => {
            unsafe {
                if enabled {
                    sie::set_stimer();
                } else {
                    sie::clear_stimer();
                }
            }
        },
        @S_EXT => {
            warn!("External IRQ should be got from PLIC, not scause");
        },
        @EX_IRQ => {
            if enabled {
                plic.set_priority(irq as _, 6);
                for hart in 0..(H as u32) {
                    plic.enable(hart, Mode::Supervisor, irq as _);
                }
            } else {
                for hart in 0..(H as u32) {
                    plic.disable(hart, Mode::Supervisor, irq as _);
                }
            }
        }
    );
}

#[doc(hidden)]
pub fn register<const H: usize>(plic: &PLIC<H>, irq: usize, handler: IrqHandler) -> bool {
    with_cause!(
        irq,
        @S_TIMER => TIMER_HANDLER.compare_exchange(core::ptr::null_mut(), handler as *mut _, Ordering::AcqRel, Ordering::Acquire).is_ok(),
        @S_EXT => {
            warn!("External IRQ should be got from PLIC, not scause");
            false
        },
        @EX_IRQ => {
            if IRQ_HANDLER_TABLE.register_handler(irq, handler) {
                set_enable(plic, irq, true);
                true
            } else {
                false
            }
        }
    )
}

#[doc(hidden)]
pub fn unregister<const H: usize>(plic: &PLIC<H>, irq: usize) -> Option<IrqHandler> {
    with_cause!(
        irq,
        @S_TIMER => {
            let handler = TIMER_HANDLER.swap(core::ptr::null_mut(), Ordering::AcqRel);
            if !handler.is_null() {
                Some(unsafe { core::mem::transmute::<*mut (), IrqHandler>(handler) })
            } else {
                None
            }
        },
        @S_EXT => {
            warn!("External IRQ should be got from PLIC, not scause");
            None
        },
        @EX_IRQ => IRQ_HANDLER_TABLE.unregister_handler(irq).inspect(|_| set_enable(plic, irq, false))
    )
}

#[doc(hidden)]
pub fn handle<const H: usize>(plic: &PLIC<H>, irq: usize) {
    with_cause!(
        irq,
        @S_TIMER => {
            trace!("IRQ: timer");
            let handler = TIMER_HANDLER.load(Ordering::Acquire);
            if !handler.is_null() {
                // SAFETY: The handler is guaranteed to be a valid function pointer.
                unsafe { core::mem::transmute::<*mut (), IrqHandler>(handler)(irq) };
            }
        },
        @S_EXT => {
            // TODO: hart
            let hart = 0;
            let irq = plic.claim(hart, Mode::Supervisor);
            if !IRQ_HANDLER_TABLE.handle(irq as _) {
                trace!("Unhandled IRQ {irq}");
            }
            plic.complete(hart, Mode::Supervisor, irq);
        },
        @EX_IRQ => {
            unreachable!("Device-side IRQs should be handled by triggering the External Interrupt.");
        }
    )
}

#[macro_export]
macro_rules! irq_if_impl {
    ($name:ident, $plic:expr) => {
        struct $name;

        #[impl_plat_interface]
        impl axplat::irq::IrqIf for $name {
            /// Enables or disables the given IRQ.
            fn set_enable(irq: usize, enabled: bool) {
                $crate::irq::set_enable(&$plic, irq, enabled);
            }

            /// Registers an IRQ handler for the given IRQ.
            ///
            /// It also enables the IRQ if the registration succeeds. It returns `false`
            /// if the registration failed.
            ///
            /// The `irq` parameter has the following semantics
            /// 1. If its highest bit is 1, it means it is an interrupt on the CPU side. Its
            /// value comes from `scause`, where [`S_SOFT`] represents software interrupt
            /// and [`S_TIMER`] represents timer interrupt. If its value is [`S_EXT`], it
            /// means it is an external interrupt, and the real IRQ number needs to
            /// be obtained from PLIC.
            /// 2. If its highest bit is 0, it means it is an interrupt on the device side,
            /// and its value is equal to the IRQ number provided by PLIC.
            fn register(irq: usize, handler: axplat::irq::IrqHandler) -> bool {
                $crate::irq::register(&$plic, irq, handler)
            }

            /// Unregisters the IRQ handler for the given IRQ.
            ///
            /// It also disables the IRQ if the unregistration succeeds. It returns the
            /// existing handler if it is registered, `None` otherwise.
            fn unregister(irq: usize) -> Option<axplat::irq::IrqHandler> {
                $crate::irq::unregister(&$plic, irq)
            }

            /// Handles the IRQ.
            ///
            /// It is called by the common interrupt handler. It should look up in the
            /// IRQ handler table and calls the corresponding handler. If necessary, it
            /// also acknowledges the interrupt controller after handling.
            fn handle(irq: usize) {
                $crate::irq::handle(&$plic, irq)
            }
        }
    };
}
