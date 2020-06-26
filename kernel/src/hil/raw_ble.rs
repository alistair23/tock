//! Bluetooth Low Energy HIL
//!
//! BLE stacks are complex. It's likely that on a large number of Tock
//! applications the BLE stack is the most complex piece of software and
//! the most vulnerable in terms of attack surface.
//!
//! This HIL is designed for running the full BLE stack on top of the HIL.
//! This HIL exposes raw functions with the expectation that the layer on top
//! will handle the more complex parts of a BLE stack.
//!
//! This HIL exposes the raw BLE read/write functions as well as a callback
//! when an interrupt is received. When combied with an alarm this should
//! be enough to implement a full BLE stack on top of this HIL.

use crate::common::leasable_buffer::LeasableBuffer;
use crate::returncode::ReturnCode;

#[allow(dead_code)]
/// This contains information on the interrupt cause,
/// It's a struct and not an enum so multiple causes can be set
/// This is passed to userspace as a bit mask
pub struct InterruptCause {
    /// The BLE device has a read avaliable
    /// When passed to userspace this is (1 << 0)
    pub read_avalialbe: bool,
    /// The BLE device is ready for a write
    /// When passed to userspace this is (1 << 1)
    pub write_ready: bool,
    /// The last command has completed
    /// When passed to userspace this is (1 << 2)
    pub command_complete: bool,
}

impl Into<usize> for InterruptCause {
    fn into(self) -> usize {
        let mut ret: usize = 0;

        if self.read_avalialbe {
            ret = ret | (1 << 0);
        }

        if self.write_ready {
            ret = ret | (1 << 1);
        }

        if self.command_complete {
            ret = ret | (1 << 2);
        }

        ret
    }
}

/// Expose the physical BLE device to a BLE stack
pub trait RawBleDriver<'a> {
    /// Set the client instance which will receive the `interrupt()` callback.
    /// This callback is called when an interrupt is received from the BLE
    /// device.
    fn set_client(&'a self, client: &'a dyn Client<'a>);

    /// Read data from the BLE device and store it in the `data` buffer.
    /// On error the return value will contain a return code and the original
    /// buffer
    fn read(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<(), (ReturnCode, &'static mut [u8])>;

    /// Write data to the BLE device from the `data` buffer.
    /// On error the return value will contain a return code and the original
    /// data
    fn write(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ReturnCode, &'static mut [u8])>;
}

pub trait Client<'a> {
    /// This callback is called when an interrupt occurs with the BLE device.
    /// On error or success `data` will contain a reference to the original
    /// data supplied to `read()` or `write()`.
    /// On success the InterruptCause struct will contain information as to why
    /// and interrupt occured.
    fn interrupt(&'a self, result: Result<InterruptCause, ReturnCode>);

    fn read_complete(&'a self, result: Result<usize, ReturnCode>, data: Option<&'static mut [u8]>);

    fn write_complete(&'a self, result: Result<usize, ReturnCode>, data: Option<&'static mut [u8]>);
}
