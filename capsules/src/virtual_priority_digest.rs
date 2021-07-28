//! Virtualise the Digest interface to enable multiple users of an underlying
//! Digest hardware peripheral.
//! This is similar to the `VirtualMuxDigest` except it also allows
//! interruptions to the process using backup and restore.

use crate::virtual_digest::VirtualMuxDigest;
use kernel::hil::digest::{self, DigestBackup};
use kernel::utilities::cells::TakeCell;
use kernel::utilities::leasable_buffer::LeasableBuffer;
use kernel::ErrorCode;

#[derive(Clone, Copy, PartialEq)]
pub enum Operation {
    Sha256,
    Sha384,
    Sha512,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Mode {
    None,
    Hmac(Operation),
    Sha(Operation),
}

pub struct VirtualMuxPriorityDigest<
    'a,
    A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>,
    const L: usize,
> {
    vdigest: &'a VirtualMuxDigest<'a, A, L>,
    backup: TakeCell<'static, [u8; L]>,
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    VirtualMuxPriorityDigest<'a, A, L>
{
    pub fn new(
        virtual_digest: &'a VirtualMuxDigest<'a, A, L>,
        backup: &'static mut [u8; L],
    ) -> VirtualMuxPriorityDigest<'a, A, L> {
        VirtualMuxPriorityDigest {
            vdigest: virtual_digest,
            backup: TakeCell::new(backup),
        }
    }

    pub fn set_hmac_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.vdigest.set_hmac_client(client);
    }

    pub fn set_sha_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.vdigest.set_sha_client(client);
    }

    pub fn is_busy(&'a self) -> bool {
        self.vdigest.is_busy()
    }

    pub fn backup_op(&'a self) {
        self.backup(self.backup.take().unwrap()).unwrap();
    }

    pub fn restore_op(&'a self) {
        self.restore(self.backup.take().unwrap()).unwrap();
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::BackupClient<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn backup_done(&'a self, _result: Result<(), ErrorCode>, dest: &'static mut [u8; L]) {
        self.backup.replace(dest);
        unimplemented!()
    }

    fn restore_done(&'a self, _result: Result<(), ErrorCode>, source: &'static mut [u8; L]) {
        self.backup.replace(source);
        unimplemented!()
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::DigestData<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn add_data(
        &self,
        data: LeasableBuffer<'static, u8>,
    ) -> Result<usize, (ErrorCode, &'static mut [u8])> {
        self.vdigest.add_data(data)
    }

    fn clear_data(&self) {
        self.vdigest.clear_data()
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::DigestHash<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn run(
        &'a self,
        digest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        self.vdigest.run(digest)
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::DigestVerify<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn verify(
        &self,
        compare: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        self.vdigest.verify(compare)
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::Digest<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_client(&'a self, client: &'a dyn digest::Client<'a, L>) {
        self.vdigest.set_client(client)
    }
}

impl<'a, A: digest::Digest<'a, L> + digest::DigestBackup<'a, L>, const L: usize>
    digest::DigestBackup<'a, L> for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_client(&'a self, client: &'a dyn digest::BackupClient<'a, L>) {
        DigestBackup::set_client(self.vdigest.mux.digest, client)
    }

    fn backup(
        &'a self,
        dest: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        if self.vdigest.is_busy() {
            // Trigger a backup in the hardware
            self.vdigest.mux.digest.backup(dest)
        } else {
            Err((ErrorCode::ALREADY, dest))
        }
    }

    fn restore(
        &'a self,
        source: &'static mut [u8; L],
    ) -> Result<(), (ErrorCode, &'static mut [u8; L])> {
        if self.vdigest.is_busy() {
            // Trigger a restore in the hardware
            self.vdigest.mux.digest.restore(source)
        } else {
            Err((ErrorCode::ALREADY, source))
        }
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::HMACSha256,
        const L: usize,
    > digest::HMACSha256 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_hmacsha256(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_hmacsha256(key)
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::HMACSha384,
        const L: usize,
    > digest::HMACSha384 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_hmacsha384(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_hmacsha384(key)
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::HMACSha512,
        const L: usize,
    > digest::HMACSha512 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_hmacsha512(&self, key: &[u8]) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_hmacsha512(key)
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::Sha256,
        const L: usize,
    > digest::Sha256 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_sha256(&self) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_sha256()
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::Sha384,
        const L: usize,
    > digest::Sha384 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_sha384(&self) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_sha384()
    }
}

impl<
        'a,
        A: digest::Digest<'a, L> + digest::DigestBackup<'a, L> + digest::Sha512,
        const L: usize,
    > digest::Sha512 for VirtualMuxPriorityDigest<'a, A, L>
{
    fn set_mode_sha512(&self) -> Result<(), ErrorCode> {
        self.vdigest.set_mode_sha512()
    }
}
