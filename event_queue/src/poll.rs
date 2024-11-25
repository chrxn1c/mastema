//! A module containing a thin layer over epoll

use crate::ffi;
use std::fmt::{Display, Formatter};
use std::io;
use std::io::Result;
use std::net::TcpStream;
use std::os::fd::AsRawFd;

type Events = Vec<ffi::Event>;

/// Struct representing an event queue
pub struct Poll {
    registry: Registry,
}

impl Poll {
    pub fn new() -> Result<Self> {
        let result = unsafe { ffi::epoll_create(1) };
        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(Self {
            registry: Registry { raw_fd: result },
        })
    }

    pub fn registry(&self) -> &Registry {
        &self.registry
    }

    /// Block the current thread until event is ready or timeout has happened
    pub fn poll(&mut self, events: &mut Events, timeout: Option<i32>) -> Result<()> {
        let event_fd = self.registry.raw_fd;
        let timeout = timeout.unwrap_or(-1);
        let max_events = events.capacity() as i32;

        let result = unsafe { ffi::epoll_wait(event_fd, events.as_mut_ptr(), max_events, timeout) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        unsafe {
            events.set_len(result as usize);
        }

        Ok(())
    }
}

/// A handle that allows to register events in [Poll]
#[derive(Debug)]
pub struct Registry {
    raw_fd: i32,
}

impl Display for Registry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("{{ Registry: raw_fd = {} }}", self.raw_fd).as_str())
    }
}
impl Registry {
    pub fn register(&self, source: &TcpStream, token: usize, interests: i32) -> Result<()> {
        let mut event = ffi::Event {
            events: interests as u32,
            epoll_data: token,
        };

        let operation = ffi::EPOLL_CTL_ADD;
        let result =
            unsafe { ffi::epoll_ctl(self.raw_fd, operation, source.as_raw_fd(), &mut event) };

        if result < 0 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

impl Drop for Registry {
    fn drop(&mut self) {
        let result = unsafe { ffi::close(self.raw_fd) };

        if result < 0 {
            let error = io::Error::last_os_error();
            eprintln!("Error occurred when dropping Registry {self}: {error}");
        }
    }
}
