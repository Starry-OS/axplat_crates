use axplat::{console::ConsoleIf, mem::VirtAddr};
use kspin::SpinNoIrq;
use lazyinit::LazyInit;
use ns16550a::Uart;

static UART: LazyInit<SpinNoIrq<Uart>> = LazyInit::new();

pub fn init_early(uart_base: VirtAddr) {
    UART.init_once(SpinNoIrq::new(Uart::new(uart_base.as_usize())));
    unsafe {
        uart_base.as_mut_ptr().byte_add(1).write_volatile(1);
    }
}

struct ConsoleIfImpl;

#[impl_plat_interface]
impl ConsoleIf for ConsoleIfImpl {
    /// Writes bytes to the console from input u8 slice.
    fn write_bytes(bytes: &[u8]) {
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
    fn read_bytes(bytes: &mut [u8]) -> usize {
        let uart = UART.lock();
        for (i, byte) in bytes.iter_mut().enumerate() {
            match uart.get() {
                Some(c) => *byte = c,
                None => return i,
            }
        }
        bytes.len()
    }

    // fn enable_rx_interrupt() -> Option<usize> {
    //     unsafe {
    //         ((UART_BASE + 1) as *mut u8).write_volatile(1);
    //     }
    //     Some(crate::config::devices::UART_INTERRUPT)
    // }
}
