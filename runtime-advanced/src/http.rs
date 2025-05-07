use crate::future::{Future, PollState};

use crate::runtime::reactor;
use crate::runtime::waker::Waker;
use mio::Interest;
use std::io::{ErrorKind, Read, Write};

fn get_request(path: impl AsRef<str> + std::fmt::Display) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
    Host: localhost\r\n\
    Connection: close\r\n\
    \r\n"
    )
}

pub struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
    task_id: usize,
}

impl HttpGetFuture {
    pub fn new(path: String) -> Self {
        let task_id = reactor::reactor().next_task_id();
        Self {
            stream: None,
            buffer: vec![],
            path,
            task_id,
        }
    }
}

impl HttpGetFuture {
    fn write_request(&mut self) {
        let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut mio_stream = mio::net::TcpStream::from_std(stream);
        mio_stream
            .write_all(get_request(&self.path).as_bytes())
            .unwrap();
        self.stream = Some(mio_stream)
    }
}

impl Future for HttpGetFuture {
    type Output = String;

    fn poll(&mut self, waker: &Waker) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("Polling HTTP Future");
            self.write_request();

            let stream = self.stream.as_mut().unwrap();
            reactor::reactor().register(stream, Interest::READABLE, self.task_id);
            reactor::reactor().set_waker(waker, self.task_id);
        }

        let mut buffer = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buffer) {
                // Everything read from a socket
                Ok(0) => {
                    let buffer = String::from_utf8_lossy(&self.buffer).to_string();
                    reactor::reactor().deregister(self.stream.as_mut().unwrap(), self.task_id);
                    println!("HTTP Future completed. Response: \n{buffer}");
                    break PollState::Ready(buffer);
                }

                // Have more bytes to read
                Ok(n) => {
                    self.buffer.extend(&buffer[0..n]);
                    continue;
                }

                // Data not ready or have more data to receive
                Err(err) if err.kind() == ErrorKind::WouldBlock => {
                    reactor::reactor().set_waker(waker, self.task_id);
                    break PollState::Pending;
                }

                // Interrupted by signal => retry
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,

                Err(err) => panic!("Unexpected error: {err:#?}"),
            }
        }
    }
}
