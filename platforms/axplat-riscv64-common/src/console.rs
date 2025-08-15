use axplat::mem::VirtAddr;
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use uart_16550::MmioSerialPort;

static UART: LazyInit<SpinNoIrq<MmioSerialPort>> = LazyInit::new();

/// Early stage initialization of the 16550 UART driver.
pub fn init_early(uart_base: VirtAddr) {
    UART.init_once(SpinNoIrq::new({
        let mut uart = unsafe { MmioSerialPort::new(uart_base.as_usize()) };
        uart.init();
        uart
    }));
}

/// Writes bytes to the console from input u8 slice.
pub fn write_bytes(bytes: &[u8]) {
    for &c in bytes {
        let mut uart = UART.lock();
        match c {
            b'\n' => {
                uart.send(b'\r');
                uart.send(b'\n');
            }
            c => {
                uart.send(c);
            }
        }
    }
}

/// Reads bytes from the console into the given mutable slice.
/// Returns the number of bytes read.
pub fn read_bytes(bytes: &mut [u8]) -> usize {
    let mut uart = UART.lock();
    for (i, byte) in bytes.iter_mut().enumerate() {
        match uart.try_receive() {
            Ok(c) => *byte = c,
            Err(_) => return i,
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
