//! Accellerator
//!
//! Usage
//! -----
//!
//! ```rust
//! ```

use crate::driver;
/// Syscall driver number.
pub const DRIVER_NUM: usize = driver::NUM::Accel as usize;

use core::cell::Cell;
use core::mem;
use kernel::common::cells::{OptionalCell, TakeCell};
use kernel::common::leasable_buffer::LeasableBuffer;
use kernel::hil::accel;
use kernel::{CommandReturn, Driver, ErrorCode, Grant, ProcessId, Read, ReadWriteAppSlice, Upcall};

#[derive(Copy, Clone)]
enum ProcessOp {
    LoadBinary,
    Run,
}

pub struct AccelDriver<'a, A: accel::Accel<'a, T>, const T: usize> {
    accel: &'a A,

    active: Cell<bool>,

    apps: Grant<App>,
    appid: OptionalCell<ProcessId>,

    in_buffer: TakeCell<'static, [u8]>,
    data_copied: Cell<usize>,
    out_buffer: TakeCell<'static, [u8; T]>,
}

impl<'a, A: accel::Accel<'a, T>, const T: usize> AccelDriver<'a, A, T> {
    pub fn new(
        accel: &'a A,
        in_buffer: &'static mut [u8],
        out_buffer: &'static mut [u8; T],
        grant: Grant<App>,
    ) -> AccelDriver<'a, A, T> {
        AccelDriver {
            accel: accel,
            active: Cell::new(false),
            apps: grant,
            appid: OptionalCell::empty(),
            in_buffer: TakeCell::new(in_buffer),
            data_copied: Cell::new(0),
            out_buffer: TakeCell::new(out_buffer),
        }
    }

    fn load_binary(&self) -> Result<(), ErrorCode> {
        self.appid.map_or(Err(ErrorCode::RESERVE), |appid| {
            self.apps
                .enter(*appid, |app| {
                    app.input.map_or(Err(ErrorCode::RESERVE), |d| {
                        self.in_buffer.map(|buf| {
                            let data = d.as_ref();

                            // Determine the size of the static buffer we have
                            let static_buffer_len = buf.len();

                            // If we have more data then the static buffer we set how much data we are going to copy
                            if data.len() > static_buffer_len {
                                self.data_copied.set(static_buffer_len);
                            }

                            // Copy the data into the static buffer
                            buf.copy_from_slice(&data[..static_buffer_len]);
                        });

                        // Add the data from the static buffer to the HMAC
                        if let Err(e) = self
                            .accel
                            .load_binary(LeasableBuffer::new(self.in_buffer.take().unwrap()))
                        {
                            self.in_buffer.replace(e.1);
                            return Err(e.0);
                        }
                        Ok(())
                    })
                })
                .unwrap_or_else(|err| Err(err.into()))
        })
    }

    fn run(&self) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn check_queue(&self) {
        for appiter in self.apps.iter() {
            let started_command = appiter.enter(|app| {
                // If an app is already running let it complete
                if self.appid.is_some() {
                    return true;
                }

                // If this app has a pending command let's use it.
                app.pending_run_app.take().map_or(false, |appid| {
                    // Mark this driver as being in use.
                    self.appid.set(appid);
                    app.pending_run_operation
                        .take()
                        .map_or(false, |op| match op {
                            ProcessOp::LoadBinary => self.load_binary() == Ok(()),
                            ProcessOp::Run => self.run() == Ok(()),
                        })
                })
            });
            if started_command {
                break;
            }
        }
    }
}

impl<'a, A: accel::Accel<'a, T>, const T: usize> accel::Client<'a, T> for AccelDriver<'a, A, T> {
    fn binary_load_done(&'a self, _result: Result<(), ErrorCode>, _input: &'static mut [u8]) {
        unimplemented!();
    }

    fn op_done(&'a self, _result: Result<(), ErrorCode>, _output: &'static mut [u8; T]) {
        unimplemented!();
    }
}

impl<'a, A: accel::Accel<'a, T>, const T: usize> Driver for AccelDriver<'a, A, T> {
    /// Specify memory regions to be used.
    ///
    /// ### `allow_num`
    ///
    /// - `0`: Allow a buffer for the input binary.
    ///        The kernel will read from this when running
    ///        This should not be changed after `load_binary` until the accelerator
    ///        has completed
    /// - `1`: Allow a buffer for storing the output result.
    ///        The kernel will write to this when running
    ///        This should not be changed after running `run` until the accelerator
    ///        has completed
    fn allow_readwrite(
        &self,
        appid: ProcessId,
        allow_num: usize,
        mut slice: ReadWriteAppSlice,
    ) -> Result<ReadWriteAppSlice, (ReadWriteAppSlice, ErrorCode)> {
        let res = match allow_num {
            // Pass buffer for the input binary to be in
            0 => self
                .apps
                .enter(appid, |app| {
                    mem::swap(&mut slice, &mut app.input);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // Pass buffer for the output result to be in
            1 => self
                .apps
                .enter(appid, |app| {
                    mem::swap(&mut slice, &mut app.output);
                    Ok(())
                })
                .unwrap_or(Err(ErrorCode::FAIL)),

            // default
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(slice),
            Err(e) => Err((slice, e)),
        }
    }

    /// Subscribe to AccelDriver events.
    ///
    /// ### `subscribe_num`
    ///
    /// - `0`: Subscribe to interrupts from Accel events.
    fn subscribe(
        &self,
        subscribe_num: usize,
        mut callback: Upcall,
        appid: ProcessId,
    ) -> Result<Upcall, (Upcall, ErrorCode)> {
        let res = match subscribe_num {
            0 => {
                // set callback
                self.apps
                    .enter(appid, |app| {
                        mem::swap(&mut app.callback, &mut callback);
                        Ok(())
                    })
                    .unwrap_or(Err(ErrorCode::FAIL))
            }

            // default
            _ => Err(ErrorCode::NOSUPPORT),
        };

        match res {
            Ok(()) => Ok(callback),
            Err(e) => Err((callback, e)),
        }
    }

    /// Setup and run the Accelerator hardware
    ///
    /// We expect userspace to setup buffers for the key, data and digest.
    /// These buffers must be allocated and specified to the kernel from the
    /// above allow calls.
    ///
    /// We expect userspace not to change the value while running. If userspace
    /// changes the value we have no guarentee of what is passed to the
    /// hardware. This isn't a security issue, it will just prove the requesting
    /// app with invalid data.
    ///
    /// The driver will take care of clearing data from the underlying impelemenation
    /// by calling the `clear_data()` function when the `hash_complete()` callback
    /// is called or if an error is encounted.
    ///
    /// ### `command_num`
    ///
    /// - `0`: check_driver
    /// - `1`: load_binary
    /// - `2`: run
    fn command(
        &self,
        command_num: usize,
        _data1: usize,
        _data2: usize,
        appid: ProcessId,
    ) -> CommandReturn {
        let match_or_empty_or_nonexistant = self.appid.map_or(true, |owning_app| {
            // We have recorded that an app has ownership of the HMAC.

            // If the HMAC is still active, then we need to wait for the operation
            // to finish and the app, whether it exists or not (it may have crashed),
            // still owns this capsule. If the HMAC is not active, then
            // we need to verify that that application still exists, and remove
            // it as owner if not.
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
                self.apps
                    .enter(*owning_app, |_| owning_app == &appid)
                    .unwrap_or(true)
            }
        });

        match command_num {
            // Driver exists
            0 => CommandReturn::success(),

            // Load binary data
            1 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let ret = self.load_binary();

                    if let Err(e) = ret {
                        self.accel.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.pending_run_operation = Some(ProcessOp::LoadBinary);
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // Start the operation
            2 => {
                if match_or_empty_or_nonexistant {
                    self.appid.set(appid);
                    let ret = self.run();

                    if let Err(e) = ret {
                        self.accel.clear_data();
                        self.appid.clear();
                        self.check_queue();
                        CommandReturn::failure(e)
                    } else {
                        CommandReturn::success()
                    }
                } else {
                    // There is an active app, so queue this request (if possible).
                    self.apps
                        .enter(appid, |app| {
                            // Some app is using the storage, we must wait.
                            if app.pending_run_app.is_some() {
                                // No more room in the queue, nowhere to store this
                                // request.
                                CommandReturn::failure(ErrorCode::NOMEM)
                            } else {
                                // We can store this, so lets do it.
                                app.pending_run_app = Some(appid);
                                app.pending_run_operation = Some(ProcessOp::Run);
                                CommandReturn::success()
                            }
                        })
                        .unwrap_or_else(|err| err.into())
                }
            }

            // default
            _ => CommandReturn::failure(ErrorCode::NOSUPPORT),
        }
    }
}

#[derive(Default)]
pub struct App {
    callback: Upcall,
    pending_run_app: Option<ProcessId>,
    pending_run_operation: Option<ProcessOp>,
    input: ReadWriteAppSlice,
    output: ReadWriteAppSlice,
}
