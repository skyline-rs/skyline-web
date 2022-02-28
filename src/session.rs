use nnsdk::web::offlinewebsession::*;
use std::ffi::CString;

use crate::PageResult;

pub use nnsdk::web::{
    offlinewebsession::OfflineWebSession, OfflineBackgroundKind as Background,
    OfflineBootDisplayKind as BootDisplay, WebSessionBootMode as Visibility,
};

extern "C" {
    #[link_name = "\u{1}_ZN2nn3web17OfflineWebSession11RequestExitEv"]
    pub fn request_exit(session: &OfflineWebSession);
}

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
    /// Up to 4 KiB in size, for larger or more efficient sizes use `recv_max`
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
    /// Up to 4 KiB in size, for larger or more efficient sizes use `try_recv_max`
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

    // Exit the browser forcefully
    pub fn exit(&self) {
        unsafe { request_exit(&self.0); }
    }
}



#[cfg(feature = "json")]
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "json")]
impl WebSession {
    /// Send a type as a JSON value, blocking until it sends
    pub fn send_json<T: Serialize>(&self, obj: &T) {
        self.send(&serde_json::to_string(obj).unwrap())
    }

    /// Attempt to send a type as a JSON value, returning false if it doesn't succeed
    pub fn try_send_json<T: Serialize>(&self, obj: &T) -> bool {
        self.try_send(&serde_json::to_string(obj).unwrap())
    }

    /// Receive a given type as a JSON message, blocking until one is ready
    pub fn recv_json<T: DeserializeOwned>(&self) -> serde_json::Result<T> {
        serde_json::from_str(&self.recv())
    }

    /// Receive a given type as a JSON message, returning None if a message is not ready
    pub fn try_recv_json<T: DeserializeOwned>(&self) -> Option<serde_json::Result<T>> {
        self.try_recv().map(|msg| serde_json::from_str(&msg))
    }

    /// Receive a given type as a JSON message, blocking until one is ready, setting a custom max
    /// payload size.
    pub fn recv_json_max<T: DeserializeOwned>(&self, max_size: usize) -> serde_json::Result<T> {
        serde_json::from_str(&self.recv_max(max_size))
    }

    /// Receive a given type as a JSON message, returning None if a message is not ready, with a
    /// max message size of `max_size`
    pub fn try_recv_json_max<T: DeserializeOwned>(
        &self,
        max_size: usize,
    ) -> Option<serde_json::Result<T>> {
        self.try_recv_max(max_size)
            .map(|msg| serde_json::from_str(&msg))
    }
}
