use crate::poll::Poll;
use std::io;

mod ffi;
mod poll;

use crate::ffi::Event;
use std::io::{Read, Result, Write};
use std::net::TcpStream;
fn main() -> Result<()> {
    let mut poll = Poll::new()?;
    let n_events = 5;
    let mut streams = vec![];
    let address = "localhost:8080";
    for i in 0..n_events {
        let delay = (n_events - i) * 1000;
        let url_path = format!("/{delay}/request-{i}");
        let request = get_raw_request(&url_path);
        let mut stream = std::net::TcpStream::connect(address)?;
        stream.set_nonblocking(true)?;
        stream.write_all(request.as_slice())?;
        poll.registry()
            .register(&stream, i, ffi::EPOLLIN | ffi::EPOLLET)?;
        streams.push(stream);
    }

    let mut handled_events = 0;
    while handled_events < n_events {
        let mut events = Vec::with_capacity(10);
        poll.poll(&mut events, None)?;
        if events.is_empty() {
            println!("Timeout or spurious event notification");
            continue;
        }
        println!("Event processed");
        handled_events += handle_events(&events, &mut streams)?;
    }
    println!("Finished processing events");
    Ok(())
}

fn get_raw_request(url_path: &str) -> Vec<u8> {
    format!(
        "GET {url_path} HTTP/1.1\r\n\
 Host: localhost\r\n\
 Connection: close\r\n\
 \r\n"
    )
    .into_bytes()
}

fn handle_events(events: &[Event], streams: &mut [TcpStream]) -> Result<usize> {
    let mut handled_events = 0;
    for event in events {
        let index = event.token();
        let mut data = vec![0u8; 4096];
        loop {
            match streams[index].read(&mut data) {
                Ok(n) if n == 0 => {
                    handled_events += 1;
                    break;
                }
                Ok(n) => {
                    let txt = String::from_utf8_lossy(&data[..n]);
                    println!("RECEIVED: {:?}", event);
                    println!("{txt}\n------\n");
                }
                // Not ready to read in a non-blocking manner. This could
                // happen even if the event was reported as ready
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            }
        }
    }
    Ok(handled_events)
}
