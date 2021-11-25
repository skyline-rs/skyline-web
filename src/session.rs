use nnsdk::web::offlinewebsession::*;
use std::ffi::CString;

use crate::PageResult;

pub use nnsdk::web::{
    offlinewebsession::OfflineWebSession, OfflineBackgroundKind as Background,
    OfflineBootDisplayKind as BootDisplay, WebSessionBootMode as Visibility,
};

pub struct WebSession(pub(crate) OfflineWebSession);

impl WebSession {
    /// Sends a message, blocking until it succeeds
    pub fn send(&self, message: &str) {
        let len = message.len() + 1;
        let message = CString::new(message).unwrap();

        while unsafe { !TrySendContentMessage(&self.0, message.as_ptr() as _, len) } {}
    }

    /// Attempts to send a message, returning true if it succeeds
    pub fn try_send(&self, message: &str) -> bool {
        let len = message.len() + 1;
        let message = CString::new(message).unwrap();

        unsafe { TrySendContentMessage(&self.0, message.as_ptr() as _, len) }
    }

    /// Blocks until a message is recieved
    ///
    /// Up to 4 KiB in size, for larger or more efficient sizes use [`recv_max`]
    pub fn recv(&self) -> String {
        self.recv_max(0x10000)
    }

    /// Blocks until a message is recieved, up to `max_size` bytes
    pub fn recv_max(&self, max_size: usize) -> String {
        let mut buffer = vec![0u8; max_size];

        loop {
            if let Some(size) = self.inner_recv(&mut buffer) {
                if size != 0 {
                    buffer.truncate(size - 1);
                    buffer.shrink_to_fit();
                    let message = String::from_utf8(buffer).map(|string| string).unwrap();

                    break message;
                }
            }
        }
    }

    /// Attempts to recieve a message without blocking
    ///
    /// Up to 4 KiB in size, for larger or more efficient sizes use [`try_recv_max`]
    pub fn try_recv(&self) -> Option<String> {
        self.try_recv_max(0x10000)
    }

    /// Attempts to recieve a message without blocking, up to `max_size` bytes
    pub fn try_recv_max(&self, max_size: usize) -> Option<String> {
        let mut buffer = vec![0u8; max_size];

        self.inner_recv(&mut buffer)
            .map(|size| {
                if size != 0 {
                    buffer.truncate(size - 1);
                    buffer.shrink_to_fit();
                    String::from_utf8(buffer).map(|string| string).ok()
                } else {
                    None
                }
            })
            .flatten()
    }

    fn inner_recv<T: AsMut<[u8]>>(&self, buffer: &mut T) -> Option<usize> {
        let buffer = buffer.as_mut();
        let mut out_size = 0;

        unsafe {
            if skyline::nn::web::offlinewebsession::TryReceiveContentMessage(
                &self.0,
                &mut out_size,
                buffer.as_mut_ptr(),
                buffer.len(),
            ) != false
            {
                Some(out_size)
            } else {
                None
            }
        }
    }

    /// Show a previously hidden web session
    pub fn show(&self) {
        unsafe { Appear(&self.0) };
    }

    /// Wait until the page has been exited
    pub fn wait_for_exit(&self) -> PageResult {
        let return_value = PageResult::new();
        unsafe { WaitForExit(&self.0, return_value.as_ref()) };
        return_value
    }
}
