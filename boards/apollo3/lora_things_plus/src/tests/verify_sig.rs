// Licensed under the Apache License, Version 2.0 or the MIT License.
// SPDX-License-Identifier: Apache-2.0 OR MIT
// Copyright Tock Contributors 2022.

use crate::tests::run_kernel_op;
use crate::ATECC508A;
use core::cell::Cell;
use kernel::hil::public_key_crypto::signature::ClientVerify;
use kernel::hil::public_key_crypto::signature::SignatureVerify;
use kernel::static_init;
use kernel::utilities::cells::TakeCell;
use kernel::{debug, ErrorCode};

struct HmacTestCallback {
    verify_done: Cell<bool>,
    hash_buffer: TakeCell<'static, [u8; 32]>,
    signature_buffer: TakeCell<'static, [u8; 64]>,
}

unsafe impl Sync for HmacTestCallback {}

impl<'a> HmacTestCallback {
    fn new(hash_buffer: &'static mut [u8; 32], signature_buffer: &'static mut [u8; 64]) -> Self {
        HmacTestCallback {
            verify_done: Cell::new(false),
            hash_buffer: TakeCell::new(hash_buffer),
            signature_buffer: TakeCell::new(signature_buffer),
        }
    }

    fn reset(&self) {
        self.verify_done.set(false);
    }
}

impl<'a> ClientVerify<32, 64> for HmacTestCallback {
    fn verification_done(
        &self,
        result: Result<bool, ErrorCode>,
        hash: &'static mut [u8; 32],
        signature: &'static mut [u8; 64],
    ) {
        self.hash_buffer.replace(hash);
        self.signature_buffer.replace(signature);
        assert_eq!(result, Ok(true));
        self.verify_done.set(true);
    }
}

/// Static init an HmacTestCallback, with
/// respective buffers allocated for data fields.
macro_rules! static_init_test_cb {
    () => {{
        let hash_data = static_init!([u8; 32], [32; 32]);
        let signature_data = static_init!(
            [u8; 64],
            [
                0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd, 0xe, 0x2, 0x82, 0xf7,
                0x8b, 0xe1, 0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa, 0x11, 0x3f, 0xf6, 0xeb,
                0xaf, 0x58, 0x57, 0x40, 0xdc, 0x55, 0x51, 0x5e, 0x30, 0xac, 0x50, 0xc7, 0x65, 0xbd,
                0xe, 0x2, 0x82, 0xf7, 0x8b, 0xe1, 0xef, 0xd1, 0xb, 0xdc, 0xa8, 0xba, 0xe1, 0xfa,
                0x11, 0x3f, 0xf6, 0xeb, 0xaf, 0x58, 0x57, 0x40,
            ]
        );

        static_init!(
            HmacTestCallback,
            HmacTestCallback::new(hash_data, signature_data)
        )
    }};
}

#[test_case]
fn hmac_check_load_binary() {
    let atecc508a = unsafe { ATECC508A.unwrap() };

    let callback = unsafe { static_init_test_cb!() };

    debug!("check signature verify... ");
    run_kernel_op(100);

    SignatureVerify::set_verify_client(atecc508a, callback);
    callback.reset();

    assert_eq!(
        atecc508a.verify(
            callback.hash_buffer.take().unwrap(),
            callback.signature_buffer.take().unwrap()
        ),
        Ok(())
    );

    run_kernel_op(1_000_000);
    assert_eq!(callback.verify_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
