use nnsdk::web::offlinewebsession::*;

use crate::PageResult;

pub use nnsdk::web::{
    offlinewebsession::OfflineWebSession, OfflineBackgroundKind as Background,
    OfflineBootDisplayKind as BootDisplay, WebSessionBootMode as Visibility,
};

pub struct WebSession(pub(crate) OfflineWebSession);

impl WebSession {
    /// Blocks until a message is recieved
    pub fn recv(&self) {
        let mut buffer = vec![0u8; 0x10000];

        loop {
            if let Some(message) = self
                .inner_recv(&mut buffer)
                .map(|size| {
                    if size != 0 {
                        buffer.truncate(size - 1);
                        buffer.shrink_to_fit();
                        match String::from_utf8(buffer).map(|string| string) {
                            Ok(message) => Some(message),
                            Err(_) => {
                                buffer = vec![0u8; 0x10000];
                                None
                            }
                        }
                    } else {
                        None
                    }
                })
                .flatten()
            {
                break message;
            }
        }
    }

    /// Attempts to recieve a message without blocking
    pub fn try_recv(&self) -> Option<String> {
        let mut buffer = vec![0u8; 0x10000];

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
