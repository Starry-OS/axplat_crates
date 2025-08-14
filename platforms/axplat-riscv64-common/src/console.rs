use axplat::mem::VirtAddr;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use ns16550a::Uart;

static UART: LazyInit<SpinNoIrq<Uart>> = LazyInit::new();

/// Early stage initialization of the 16550 UART driver.
pub fn init_early(uart_base: VirtAddr) {
    UART.init_once(SpinNoIrq::new(Uart::new(uart_base.as_usize())));
    unsafe {
        uart_base.as_mut_ptr().byte_add(1).write_volatile(1);
    }
}

/// Writes bytes to the console from input u8 slice.
pub fn write_bytes(bytes: &[u8]) {
    for &c in bytes {
        let uart = UART.lock();
        match c {
            b'\n' => {
                let _ = uart.put(b'\r');
                let _ = uart.put(b'\n');
            }
            c => {
                let _ = uart.put(c);
            }
        }
    }
}

/// Reads bytes from the console into the given mutable slice.
/// Returns the number of bytes read.
pub fn read_bytes(bytes: &mut [u8]) -> usize {
    let uart = UART.lock();
    for (i, byte) in bytes.iter_mut().enumerate() {
        match uart.get() {
            Some(c) => *byte = c,
            None => return i,
        }
    }
    bytes.len()
}

#[macro_export]
macro_rules! console_if_impl {
    ($name:ident) => {
        struct $name;

        #[axplat::impl_plat_interface]
        impl axplat::console::ConsoleIf for $name {
            /// Writes given bytes to the console.
            fn write_bytes(bytes: &[u8]) {
                $crate::console::write_bytes(bytes);
            }

            /// Reads bytes from the console into the given mutable slice.
            ///
            /// Returns the number of bytes read.
            fn read_bytes(bytes: &mut [u8]) -> usize {
                $crate::console::read_bytes(bytes)
            }
        }
    };
}
