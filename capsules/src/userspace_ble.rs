//! Bluetooth Low Energy Driver
//!
//! This is a basic driver that exposes the read/write and subscribe interface
//! from a BLE device to a user space application. This is useful for allowing
//! a BLE stack to run in a userspace application.
//!

use core::cell::Cell;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::hil::raw_ble;
use kernel::hil::raw_ble::InterruptCause;
use kernel::{AppId, AppSlice, Callback, Driver, Grant, ReturnCode, Shared};

/// Syscall driver number.
use crate::driver;
pub const DRIVER_NUM: usize = driver::NUM::UserSpaceBLE as usize;

#[derive(Copy, Clone, PartialEq)]
enum LastOperation {
    NoOp,
    Read,
    Write,
}

pub struct BLE<'a, B: raw_ble::RawBleDriver<'a>> {
    radio: &'a B,

    read: TakeCell<'static, [u8]>,
    write: TakeCell<'static, [u8]>,

    active: Cell<bool>,
    app: kernel::Grant<App>,
    appid: OptionalCell<AppId>,

    last_op: Cell<LastOperation>,
}

impl<'a, B> BLE<'a, B>
where
    B: raw_ble::RawBleDriver<'a>,
{
    pub fn new(
        radio: &'a B,
        read: &'static mut [u8],
        write: &'static mut [u8],
        grant: Grant<App>,
    ) -> BLE<'a, B> {
        BLE {
            radio: radio,
            read: TakeCell::new(read),
            write: TakeCell::new(write),
            active: Cell::new(false),
            app: grant,
            appid: OptionalCell::empty(),
            last_op: Cell::new(LastOperation::NoOp),
        }
    }
}

impl<'a, B> raw_ble::Client<'a> for BLE<'a, B>
where
    B: raw_ble::RawBleDriver<'a>,
{
    fn interrupt(
        &'a self,
        result: Result<InterruptCause, ReturnCode>,
        data: Option<&'static mut [u8]>,
    ) {
        match data {
            Some(d) => {
                if self.last_op.get() == LastOperation::Read {
                    self.read.replace(d);
                } else if self.last_op.get() == LastOperation::Write {
                    self.write.replace(d);
                }
            }
            _ => {}
        }

        if let Ok(res) = result {
            self.appid.map(|appid| {
                let _ = self.app.enter(*appid, |app, _| {
                    app.callback.map(|cb| {
                        cb.schedule(res.into(), 0, 0);
                    });
                });
            });
        }
    }
}

// System Call implementation
impl<'a, B> Driver for BLE<'a, B>
where
    B: raw_ble::RawBleDriver<'a>,
{
    fn command(
        &self,
        command_num: usize,
        _data: usize,
        _interval: usize,
        appid: kernel::AppId,
    ) -> ReturnCode {
        let match_or_empty_or_nonexistant = self.appid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the BLE.
            if self.active.get() {
                owning_app == &appid
            } else {
                // Check the app still exists.
                //
                // If the `.enter()` succeeds, then the app is still valid, and
                // we can check if the owning app matches the one that called
                // the command. If the `.enter()` fails, then the owning app no
                // longer exists and we return `true` to signify the
                // "or_nonexistant" case.
                self.app
                    .enter(*owning_app, |_, _| owning_app == &appid)
                    .unwrap_or(true)
            }
        });

        if !match_or_empty_or_nonexistant {
            // Only 1 app can use the BLE device, return a failure
            return ReturnCode::EBUSY;
        }

        match command_num {
            // Start read command
            0 => {
                self.last_op.set(LastOperation::Read);
                self.appid.set(appid);
                if let Err(e) = self
                    .radio
                    .read(LeasableBuffer::new(self.read.take().unwrap()))
                {
                    self.read.replace(e.1);
                    self.last_op.set(LastOperation::NoOp);
                    return e.0;
                }
                ReturnCode::SUCCESS
            }

            // Start write command
            1 => self
                .app
                .enter(appid, |app, _| {
                    match app.write.as_ref() {
                        Some(d) => {
                            let mut copy_len = 0;
                            self.write.map(|buf| {
                                let data = d.as_ref();

                                // Determine the size of the static buffer we have
                                let static_buffer_len = buf.len();

                                // If we have more data then the static buffer we set how much data we are going to copy
                                if data.len() > static_buffer_len {
                                    copy_len = static_buffer_len;
                                } else {
                                    copy_len = data.len();
                                }

                                // Copy the data into the static buffer
                                buf[..copy_len].copy_from_slice(&data[..copy_len]);
                            });

                            let mut lease_buf = LeasableBuffer::new(self.write.take().unwrap());

                            lease_buf.slice(..copy_len);

                            self.last_op.set(LastOperation::Write);
                            self.appid.set(appid);
                            if let Err(e) = self.radio.write(lease_buf) {
                                self.write.replace(e.1);
                                self.last_op.set(LastOperation::NoOp);
                                return e.0;
                            }
                        }
                        None => {
                            return ReturnCode::ERESERVE;
                        }
                    };

                    ReturnCode::SUCCESS
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
            // Read buffer
            0 => self
                .app
                .enter(appid, |app, _| {
                    app.read = slice;
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),

            // Write buffer
            1 => self
                .app
                .enter(appid, |app, _| {
                    app.write = slice;
                    ReturnCode::SUCCESS
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
        appid: kernel::AppId,
    ) -> ReturnCode {
        match subscribe_num {
            // Callback for interrupts
            0 => self
                .app
                .enter(appid, |app, _| {
                    self.appid.set(appid);
                    app.callback.insert(callback);
                    ReturnCode::SUCCESS
                })
                .unwrap_or_else(|err| err.into()),
            _ => ReturnCode::ENOSUPPORT,
        }
    }
}

pub struct App {
    callback: OptionalCell<Callback>,
    read: Option<AppSlice<Shared, u8>>,
    write: Option<AppSlice<Shared, u8>>,
}

impl Default for App {
    fn default() -> App {
        App {
            callback: OptionalCell::empty(),
            read: None,
            write: None,
        }
    }
}
