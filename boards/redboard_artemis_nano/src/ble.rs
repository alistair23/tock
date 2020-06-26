//! Component for BLE radio on Apollo3 based platforms.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio = BLEComponent::new(board_kernel, &apollo3::ble::BLE).finalize();
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::{create_capability, static_init};

/// BLE component for Apollo3 BLE
pub struct BLEComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static apollo3::ble::Ble<'static>,
    read_buffer: &'static mut [u8],
    write_buffer: &'static mut [u8],
}

/// BLE component for Apollo3 BLE
impl BLEComponent {
    /// New instance
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static apollo3::ble::Ble,
        read_buffer: &'static mut [u8],
        write_buffer: &'static mut [u8],
    ) -> BLEComponent {
        BLEComponent {
            board_kernel: board_kernel,
            radio: radio,
            read_buffer,
            write_buffer,
        }
    }
}

impl Component for BLEComponent {
    type StaticInput = ();
    type Output = &'static capsules::userspace_ble::BLE<'static, apollo3::ble::Ble<'static>>;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio = static_init!(
            capsules::userspace_ble::BLE<'static, apollo3::ble::Ble>,
            capsules::userspace_ble::BLE::new(
                self.radio,
                self.read_buffer,
                self.write_buffer,
                self.board_kernel.create_grant(&grant_cap),
            )
        );
        kernel::hil::raw_ble::RawBleDriver::set_client(self.radio, ble_radio);

        ble_radio
    }
}
