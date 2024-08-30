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
    message_buffer: TakeCell<'static, [u8; 32]>,
    signature_buffer: TakeCell<'static, [u8; 64]>,
    pub_key_buffer: TakeCell<'static, [u8; 64]>,
}

unsafe impl Sync for HmacTestCallback {}

impl<'a> HmacTestCallback {
    fn new(
        message_buffer: &'static mut [u8; 32],
        signature_buffer: &'static mut [u8; 64],
        pub_key_buffer: &'static mut [u8; 64],
    ) -> Self {
        HmacTestCallback {
            verify_done: Cell::new(false),
            message_buffer: TakeCell::new(message_buffer),
            signature_buffer: TakeCell::new(signature_buffer),
            pub_key_buffer: TakeCell::new(pub_key_buffer),
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
        self.message_buffer.replace(hash);
        self.signature_buffer.replace(signature);
        assert_eq!(result, Ok(true));
        self.verify_done.set(true);
    }
}

/// The below values are generated from the following
///
/// Generate a NIST P-256 keypair
/// https://emn178.github.io/online-tools/ecdsa/key-generator/?curve=secp256r1&key_type=pem_text&pem_format=PKCS8&cipher_algorithm=AES-256-CBC&passphrase_enabled=0
///
/// Use the private key to sign the data
/// https://emn178.github.io/online-tools/ecdsa/sign/?input=5468697320697320612074657374206d65737361676520746f207369676e2121&input_type=hex&output_type=hex&curve=secp256r1&algorithm=SHA256&private_key_input_type=pem_text&private_key=-----BEGIN%20PRIVATE%20KEY-----%0AMIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgpfYKru9pBdy699D5%0ABGw1TjonGiMCLLk2So8HX83PbyyhRANCAASut7QmJjXVsrnbyo%2BHw1jfdP%2B5Bno3%0A4DHmEh%2FPpKRt%2FrINyVgYLTRJb6cfZIbjfV6uLah7EkUc9yqGS8qPvFmg%0A-----END%20PRIVATE%20KEY-----%0A
///
/// Extract the public key X and Y values
/// `openssl ec -in pub_key.pem -pubin -text -noout`
///
/// Extract the signature R and S values
/// `echo -n "3045022100a9b933ef3fe5db7771f1c661a233b84db76745d70e1c14c920cc2bad447ded4002207911dfac1c2630ba1cf7c96efd4884f1b702a6a6076f3bb0cb1fad2d3e3b9886" | xxd -r -p | openssl asn1parse -inform DER`
///
/// The signature can be verified with:
/// https://emn178.github.io/online-tools/ecdsa/verify/?input=5468697320697320612074657374206d65737361676520746f207369676e2121&input_type=hex&curve=secp256r1&algorithm=SHA256&public_key_input_type=pem_text&public_key=-----BEGIN%20PUBLIC%20KEY-----%0AMFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAErre0JiY11bK528qPh8NY33T%2FuQZ6%0AN%2BAx5hIfz6Skbf6yDclYGC00SW%2BnH2SG431eri2oexJFHPcqhkvKj7xZoA%3D%3D%0A-----END%20PUBLIC%20KEY-----%0A&signature_input_type=hex&signature=3045022100a9b933ef3fe5db7771f1c661a233b84db76745d70e1c14c920cc2bad447ded4002207911dfac1c2630ba1cf7c96efd4884f1b702a6a6076f3bb0cb1fad2d3e3b9886
///
/// I also tried the following Python code, but the signatures never validated
///
/// ```python
/// import ecdsa
/// from ecdsa import SigningKey, NIST256p
/// from hashlib import sha256
///
/// sk = ecdsa.SigningKey.generate(curve=NIST256p, hashfunc=sha256)
///
/// # Public Key with X and Y values in a single 64-byte hex array
/// sk.verifying_key.to_string().hex()
///
/// # Private Key
/// sk.to_string().hex()
///
/// # Prints the R and the S value in a single 64-byte hex array
/// sk.sign_deterministic(b"This is a test message to sign!!").hex()
/// ```
macro_rules! static_init_test_cb {
    () => {{
        let message_data = static_init!(
            [u8; 32],
            [
            // The SHA256 of the message: 5468697320697320612074657374206d65737361676520746f207369676e2121
            // https://emn178.github.io/online-tools/sha256.html?input=5468697320697320612074657374206d65737361676520746f207369676e2121&input_type=hex&output_type=hex&hmac_enabled=0&hmac_input_type=utf-8
            0x61, 0xff, 0x79, 0x61, 0x27, 0xe5, 0xf8, 0xe4, 0x61, 0x8d, 0xde, 0x14, 0x4f, 0x5b, 0x91, 0xcc, 0xa4, 0x47, 0x16, 0xda, 0xc8, 0x75, 0x8b, 0xe2, 0x85, 0x9e, 0xbf, 0x1d, 0xb1, 0x2f, 0xe2, 0xc7,

                // 0x54, 0x68, 0x69, 0x73, 0x20, 0x69, 0x73, 0x20, 0x61, 0x20, 0x74, 0x65, 0x73, 0x74,
                // 0x20, 0x6d, 0x65, 0x73, 0x73, 0x61, 0x67, 0x65, 0x20, 0x74, 0x6f, 0x20, 0x73, 0x69,
                // 0x67, 0x6e, 0x21, 0x21

                // 0x26, 0x60, 0x20, 0xef, 0xe6, 0x6a, 0xf4, 0xfa, 0x02, 0x5c, 0x8f, 0x74, 0x78,
                // 0xdb, 0xb2, 0xce, 0x76, 0x4c, 0x4c, 0xe3, 0xf7, 0x30, 0xe4, 0x53, 0x59, 0xa7,
                // 0xdb, 0x09, 0xcb, 0x45, 0xc7, 0xfe
            ]
        );
        let signature_data = static_init!(
            [u8; 64],
            [
                0xA9, 0xB9, 0x33, 0xEF, 0x3F, 0xE5, 0xDB, 0x77, 0x71, 0xF1, 0xC6, 0x61,
                0xA2, 0x33, 0xB8, 0x4D, 0xB7, 0x67, 0x45, 0xD7, 0x0E, 0x1C, 0x14, 0xC9,
                0x20, 0xCC, 0x2B, 0xAD, 0x44, 0x7D, 0xED, 0x40,

                0x79, 0x11, 0xDF, 0xAC, 0x1C, 0x26, 0x30, 0xBA, 0x1C, 0xF7, 0xC9, 0x6E,
                0xFD, 0x48, 0x84, 0xF1, 0xB7, 0x02, 0xA6, 0xA6, 0x07, 0x6F, 0x3B, 0xB0,
                0xCB, 0x1F, 0xAD, 0x2D, 0x3E, 0x3B, 0x98, 0x86,

            // 0x71, 0x07, 0x7D, 0x35, 0x6F, 0xCD, 0x70, 0xD4, 0xCC, 0x47, 0x2A, 0xD0, 0x49, 0x0E, 0x75, 0xAB,
            // 0xC5, 0x41, 0x98, 0xEE, 0x6A, 0x96, 0x7B, 0x90, 0xF2, 0xC7, 0xE3, 0xC8, 0x2B, 0xBF, 0x54, 0x96,
            // 0x77, 0x8E, 0xFE, 0x0B, 0xF6, 0x9D, 0x15, 0xED, 0xA0, 0x71, 0xBD, 0xD3, 0xFE, 0x46, 0x99, 0x26,
            // 0x31, 0xF8, 0x80, 0x01, 0x13, 0x76, 0xCD, 0x45, 0x7C, 0x62, 0x55, 0x43, 0xC9, 0x7F, 0xCC, 0xD9
            ]
        );
// -----BEGIN PRIVATE KEY-----
// MIGHAgEAMBMGByqGSM49AgEGCCqGSM49AwEHBG0wawIBAQQgpfYKru9pBdy699D5
// BGw1TjonGiMCLLk2So8HX83PbyyhRANCAASut7QmJjXVsrnbyo+Hw1jfdP+5Bno3
// 4DHmEh/PpKRt/rINyVgYLTRJb6cfZIbjfV6uLah7EkUc9yqGS8qPvFmg
// -----END PRIVATE KEY-----
// 
// -----BEGIN PUBLIC KEY-----
// MFkwEwYHKoZIzj0CAQYIKoZIzj0DAQcDQgAErre0JiY11bK528qPh8NY33T/uQZ6
// N+Ax5hIfz6Skbf6yDclYGC00SW+nH2SG431eri2oexJFHPcqhkvKj7xZoA==
// -----END PUBLIC KEY-----
        let public_key = static_init!(
            [u8; 64],
            [
                0xae, 0xb7, 0xb4, 0x26, 0x26, 0x35, 0xd5, 0xb2, 0xb9, 0xdb, 0xca, 0x8f, 0x87, 0xc3, 0x58, 
                0xdf, 0x74, 0xff, 0xb9, 0x06, 0x7a, 0x37, 0xe0, 0x31, 0xe6, 0x12, 0x1f, 0xcf, 0xa4, 0xa4,
                0x6d, 0xfe,

                0xb2, 0x0d, 0xc9, 0x58, 0x18, 0x2d, 0x34, 0x49, 0x6f, 0xa7, 0x1f, 0x64, 0x86, 0xe3, 0x7d,
                0x5e, 0xae, 0x2d, 0xa8, 0x7b, 0x12, 0x45, 0x1c, 0xf7, 0x2a, 0x86, 0x4b, 0xca, 0x8f, 0xbc,
                0x59, 0xa0

        // 0x8D, 0x61, 0x7E, 0x65, 0xC9, 0x50, 0x8E, 0x64, 0xBC, 0xC5, 0x67, 0x3A, 0xC8, 0x2A, 0x67, 0x99,
        // 0xDA, 0x3C, 0x14, 0x46, 0x68, 0x2C, 0x25, 0x8C, 0x46, 0x3F, 0xFF, 0xDF, 0x58, 0xDF, 0xD2, 0xFA,
        // 0x3E, 0x6C, 0x37, 0x8B, 0x53, 0xD7, 0x95, 0xC4, 0xA4, 0xDF, 0xFB, 0x41, 0x99, 0xED, 0xD7, 0x86,
        // 0x2F, 0x23, 0xAB, 0xAF, 0x02, 0x03, 0xB4, 0xB8, 0x91, 0x1B, 0xA0, 0x56, 0x99, 0x94, 0xE1, 0x01
            ]
        );

        static_init!(
            HmacTestCallback,
            HmacTestCallback::new(message_data, signature_data, public_key)
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

    atecc508a.set_public_key(Some(callback.pub_key_buffer.take().unwrap()));

    assert_eq!(
        atecc508a.verify(
            callback.message_buffer.take().unwrap(),
            callback.signature_buffer.take().unwrap()
        ),
        Ok(())
    );

    run_kernel_op(1_000_000);
    assert_eq!(callback.verify_done.get(), true);

    debug!("    [ok]");
    run_kernel_op(100);
}
