use crate::future::{Future, PollState};
use std::io::{ErrorKind, Read, Write};

fn get_request(path: impl AsRef<str> + std::fmt::Display) -> String {
    format!(
        "GET {path} HTTP/1.1\r\n\
    Host: localhost\r\n\
    Connection: close\r\n\
    \r\n"
    )
}

pub struct HttpClient;
impl HttpClient {
    pub fn get_request(path: String) -> impl Future<Output = String> {
        HttpGetFuture::new(path)
    }
}

struct HttpGetFuture {
    stream: Option<mio::net::TcpStream>,
    buffer: Vec<u8>,
    path: String,
}

impl HttpGetFuture {
    pub fn new(path: String) -> Self {
        Self {
            stream: None,
            buffer: vec![],
            path,
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

    fn poll(&mut self) -> PollState<Self::Output> {
        if self.stream.is_none() {
            println!("Initiating poll..");
            self.write_request();
            return PollState::Pending;
        }

        let mut buffer = vec![0u8; 4096];
        loop {
            match self.stream.as_mut().unwrap().read(&mut buffer) {
                // Everything read from a socket
                Ok(0) => {
                    let buffer = String::from_utf8_lossy(&self.buffer);
                    break PollState::Ready(buffer.to_string());
                }

                // Have more bytes to read
                Ok(n) => {
                    self.buffer.extend(&buffer[0..n]);
                    continue;
                }

                // Data not ready or have more data to receive
                Err(err) if err.kind() == ErrorKind::WouldBlock => break PollState::Pending,

                // Interrupted by signal => retry
                Err(err) if err.kind() == ErrorKind::Interrupted => continue,

                Err(err) => panic!("Unexpected error: {err:#?}"),
            }
        }
    }
}
