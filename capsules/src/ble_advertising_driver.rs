//! Bluetooth Low Energy Advertising Driver
//!
//! A system call driver that exposes the Bluetooth Low Energy advertising
//! channel. The driver generates a unique static address for each process,
//! allowing each process to act as its own device and send or scan for
//! advertisements. Timing of advertising or scanning events is handled by the
//! driver but processes can request an advertising or scanning interval.
//! Processes can also control the TX power used for their advertisements.
//!
//! Data payloads are limited to 31 bytes since the maximum advertising channel
//! protocol data unit (PDU) is 37 bytes and includes a 6-byte header.
//!
//! ### Allow system call
//!
//! The allow systems calls are used for buffers from allocated by userland
//!
//! There are two different buffers:
//! * 0: Advertising data
//! * 1: Passive scanning buffer
//!
//! The possible return codes from the 'allow' system call indicate the following:
//!
//! * SUCCESS: The buffer has successfully been filled
//! * ENOMEM: No sufficient memory available
//! * EINVAL: Invalid address of the buffer or other error
//! * EBUSY: The driver is currently busy with other tasks
//! * ENOSUPPORT: The operation is not supported
//! * ERROR: Operation `map` on Option failed
//!
//! ### Subscribe system call
//!
//!  The `subscribe` system call supports two arguments `subscribe number' and `callback`.
//!  The `subscribe` is used to specify the specific operation, currently:
//!
//! * 0: provides a callback user-space when a device scanning for advertisements
//!      and the callback is used to invoke user-space processes.
//!
//! The possible return codes from the `allow` system call indicate the following:
//!
//! * ENOMEM:    Not sufficient amount memory
//! * EINVAL:    Invalid operation
//!
//! ### Command system call
//!
//! The `command` system call supports two arguments `command number` and `subcommand number`.
//! `command number` is used to specify the specific operation, currently
//! the following commands are supported:
//!
//! * 0: start advertisement
//! * 1: stop advertisement or scanning
//! * 5: start scanning
//!
//! The possible return codes from the `command` system call indicate the following:
//!
//! * SUCCESS:      The command was successful
//! * EBUSY:        The driver is currently busy with other tasks
//! * ENOSUPPORT:   The operation is not supported
//!
//! Usage
//! -----
//!
//! You need a device that provides the `kernel::BleAdvertisementDriver` trait along with a virtual
//! timer to perform events and not block the entire kernel
//!
//! ```rust
//! # use kernel::static_init;
//! # use capsules::virtual_alarm::VirtualMuxAlarm;
//!
//! let ble_radio = static_init!(
//! nrf5x::ble_advertising_driver::BLE<
//!     'static,
//!     nrf52::radio::Radio, VirtualMuxAlarm<'static, Rtc>
//! >,
//! nrf5x::ble_advertising_driver::BLE::new(
//!     &mut nrf52::radio::RADIO,
//!     board_kernel.create_grant(&grant_cap),
//!     &mut nrf5x::ble_advertising_driver::BUF,
//!     ble_radio_virtual_alarm));
//! nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_rx_client(&nrf52::radio::RADIO,
//!                                                                   ble_radio);
//! nrf5x::ble_advertising_hil::BleAdvertisementDriver::set_tx_client(&nrf52::radio::RADIO,
//!                                                                   ble_radio);
//! ble_radio_virtual_alarm.set_client(ble_radio);
//! ```
//!
//! ### Authors
//! * Niklas Adolfsson <niklasadolfsson1@gmail.com>
//! * Fredrik Nilsson <frednils@student.chalmers.se>
//! * Date: June 22, 2017

// # Implementation
//
// Advertising virtualization works by implementing a virtual periodic timer for each process. The
// timer is configured to fire at each advertising interval, as specified by the process. When a
// timer fires, we serialize the advertising packet for that process (using the provided AdvData
// payload, generated address and PDU type) and perform one advertising event (on each of three
// channels).
//
// This means that advertising events can collide. In this case, we just defer one of the
// advertisements. Because we add a pseudo random pad to the timer interval each time (as required
// by the Bluetooth specification) multiple collisions of the same processes are highly unlikely.

use core::cell::Cell;
use core::cmp;
use kernel::common::cells::OptionalCell;
use kernel::debug;
use kernel::hil::ble_advertising;
use kernel::hil::ble_advertising::RadioChannel;
use kernel::hil::time::Frequency;
use kernel::ReturnCode;

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::BleAdvertising as usize;

/// Advertisement Buffer
pub static mut BUF: [u8; PACKET_LENGTH] = [0; PACKET_LENGTH];

const PACKET_ADDR_LEN: usize = 6;
const PACKET_LENGTH: usize = 39;
const ADV_HEADER_TXADD_OFFSET: usize = 6;

#[derive(PartialEq, Debug)]
enum BLEState {
    NotInitialized,
    Initialized,
    ScanningIdle,
    Scanning(RadioChannel),
    AdvertisingIdle,
    Advertising(RadioChannel),
}

#[derive(Copy, Clone)]
enum Expiration {
    Disabled,
    Abs(u32),
}

#[derive(Copy, Clone)]
struct AlarmData {
    t0: u32,
    expiration: Expiration,
}

impl AlarmData {
    fn new() -> AlarmData {
        AlarmData {
            t0: 0,
            expiration: Expiration::Disabled,
        }
    }
}

type AdvPduType = u8;

// BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part B], section 2.3.3
const ADV_IND: AdvPduType = 0b0000;
#[allow(dead_code)]
const ADV_DIRECTED_IND: AdvPduType = 0b0001;
const ADV_NONCONN_IND: AdvPduType = 0b0010;
#[allow(dead_code)]
const SCAN_REQ: AdvPduType = 0b0011;
#[allow(dead_code)]
const SCAN_RESP: AdvPduType = 0b0100;
#[allow(dead_code)]
const CONNECT_IND: AdvPduType = 0b0101;
const ADV_SCAN_IND: AdvPduType = 0b0110;

/// Process specific memory
pub struct App {
    process_status: Option<BLEState>,
    alarm_data: AlarmData,

    // Advertising meta-data
    adv_data: Option<kernel::AppSlice<kernel::Shared, u8>>,
    address: [u8; PACKET_ADDR_LEN],
    pdu_type: AdvPduType,
    advertisement_interval_ms: u32,
    tx_power: u8,
    /// The state of an app-specific pseudo random number.
    ///
    /// For example, it can be used for the pseudo-random `advDelay` parameter.
    /// It should be read using the `random_number` method, which updates it as
    /// well.
    random_nonce: u32,

    // Scanning meta-data
    scan_buffer: Option<kernel::AppSlice<kernel::Shared, u8>>,
    scan_callback: Option<kernel::Callback>,
}

impl Default for App {
    fn default() -> App {
        App {
            alarm_data: AlarmData::new(),
            adv_data: None,
            scan_buffer: None,
            address: [0; PACKET_ADDR_LEN],
            pdu_type: ADV_NONCONN_IND,
            scan_callback: None,
            process_status: Some(BLEState::NotInitialized),
            tx_power: 0,
            advertisement_interval_ms: 200,
            // Just use any non-zero starting value by default
            random_nonce: 0xdeadbeef,
        }
    }
}

pub struct BLE<'a, B>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
{
    radio: &'a B,
    busy: Cell<bool>,
    app: kernel::Grant<App>,
    kernel_tx: kernel::common::cells::TakeCell<'static, [u8]>,
    sending_app: OptionalCell<kernel::AppId>,
    receiving_app: OptionalCell<kernel::AppId>,
}

impl<'a, B> BLE<'a, B>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
{
    pub fn new(
        radio: &'a B,
        container: kernel::Grant<App>,
        tx_buf: &'static mut [u8],
    ) -> BLE<'a, B> {
        BLE {
            radio: radio,
            busy: Cell::new(false),
            app: container,
            kernel_tx: kernel::common::cells::TakeCell::new(tx_buf),
            sending_app: OptionalCell::empty(),
            receiving_app: OptionalCell::empty(),
        }
    }
}

// Callback from the radio once a RX event occur
impl<'a, B> ble_advertising::RxClient for BLE<'a, B>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
{
    fn receive_event(&self, buf: &'static mut [u8], len: u8, result: ReturnCode) {
        debug!("receive_event");
    }
}

// Callback from the radio once a TX event occur
impl<'a, B> ble_advertising::TxClient for BLE<'a, B>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
{
    // The ReturnCode indicates valid CRC or not, not used yet but could be used for
    // re-transmissions for invalid CRCs
    fn transmit_event(&self, _buf: &'static mut [u8], _crc_ok: ReturnCode) {

        self.receiving_app.map(|appid| {
            let _ = self.app.enter(*appid, |app, _| {
                app.scan_callback.map(|mut cb| {
                    cb.schedule(0, 0, 0);
                });
            });
        });
    }
}

// System Call implementation
impl<'a, B> kernel::Driver for BLE<'a, B>
where
    B: ble_advertising::BleAdvertisementDriver + ble_advertising::BleConfig,
{
    fn command(
        &self,
        command_num: usize,
        data: usize,
        interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        match command_num {
            // Start periodic advertisements
            0 => self
                .app
                .enter(appid, |app, _| {
                    debug!("*** 0 command");
                    ReturnCode::EBUSY
                })
                .unwrap_or_else(|err| err.into()),

            // Stop periodic advertisements or passive scanning
            1 => self
                .app
                .enter(appid, |app, _| match app.process_status {
                    Some(BLEState::AdvertisingIdle) | Some(BLEState::ScanningIdle) => {
                        debug!("*** 1 command");
                        app.process_status = Some(BLEState::Initialized);
                        ReturnCode::SUCCESS
                    }
                    _ => ReturnCode::EBUSY,
                })
                .unwrap_or_else(|err| err.into()),

            // Configure transmitted power
            // BLUETOOTH SPECIFICATION Version 4.2 [Vol 6, Part A], section 3
            //
            // Minimum Output Power:    0.01 mW (-20 dBm)
            // Maximum Output Power:    10 mW (+10 dBm)
            //
            // data - Transmitting power in dBm
            2 => {
                self.app
                    .enter(appid, |app, _| {
                        debug!("*** 2 command");
                        ReturnCode::EBUSY
                    })
                    .unwrap_or_else(|err| err.into())
            }

            // Passive scanning mode
            5 => self
                .app
                .enter(appid, |app, _| {
                    debug!("*** 5 command");
                    ReturnCode::EBUSY
                })
                .unwrap_or_else(|err| err.into()),

            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn allow(
        &self,
        appid: kernel::AppId,
        allow_num: usize,
        slice: Option<kernel::AppSlice<kernel::Shared, u8>>,
    ) -> ReturnCode {
        match allow_num {
            // Advertisement buffer
            0 => self
                .app
                .enter(appid, |app, _| {
                    app.adv_data = slice;
                    ReturnCode::FAIL
                })
                .unwrap_or_else(|err| err.into()),

            // Passive scanning buffer
            1 => self
                .app
                .enter(appid, |app, _| {
                    ReturnCode::FAIL
                })
                .unwrap_or_else(|err| err.into()),

            // Operation not supported
            _ => ReturnCode::ENOSUPPORT,
        }
    }

    fn subscribe(
        &self,
        subscribe_num: usize,
        callback: Option<kernel::Callback>,
        app_id: kernel::AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Callback for scanning
            0 => self
                .app
                .enter(app_id, |app, _| {
                        debug!("Running subscribe");
                        self.receiving_app.set(app_id);
                        app.scan_callback = callback;
                        ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}
