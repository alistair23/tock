//! Component for BLE radio on Apollo3 based platforms.
//!
//! Usage
//! -----
//! ```rust
//! let ble_radio = BLEComponent::new(board_kernel, &apollo3::ble::BLE, mux_alarm).finalize();
//! ```

#![allow(dead_code)] // Components are intended to be conditionally included

use capsules;
use capsules::virtual_alarm::VirtualMuxAlarm;
use kernel::capabilities;
use kernel::component::Component;
use kernel::hil;
use kernel::{create_capability, static_init};

/// BLE component for Apollo3 BLE
pub struct BLEComponent {
    board_kernel: &'static kernel::Kernel,
    radio: &'static apollo3::ble::Ble,
}

/// BLE component for Apollo3 BLE
impl BLEComponent {
    /// New instance
    pub fn new(
        board_kernel: &'static kernel::Kernel,
        radio: &'static apollo3::ble::Ble,
    ) -> BLEComponent {
        BLEComponent {
            board_kernel: board_kernel,
            radio: radio,
        }
    }
}

impl Component for BLEComponent {
    type StaticInput = ();
    type Output = &'static capsules::ble_advertising_driver::BLE<
        'static,
        apollo3::ble::Ble,
    >;

    unsafe fn finalize(self, _s: Self::StaticInput) -> Self::Output {
        let grant_cap = create_capability!(capabilities::MemoryAllocationCapability);

        let ble_radio = static_init!(
            capsules::ble_advertising_driver::BLE<
                'static,
                apollo3::ble::Ble,
            >,
            capsules::ble_advertising_driver::BLE::new(
                self.radio,
                self.board_kernel.create_grant(&grant_cap),
                &mut capsules::ble_advertising_driver::BUF
            )
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_receive_client(
            self.radio, ble_radio,
        );
        kernel::hil::ble_advertising::BleAdvertisementDriver::set_transmit_client(
            self.radio, ble_radio,
        );

        ble_radio
    }
}
