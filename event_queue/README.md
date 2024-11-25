# Event Queue

A simple implementation of event queue using [epoll](https://man7.org/linux/man-pages/man7/epoll.7.html).

- Inspiration: [Metal I/O](https://github.com/chrxn1c/mastema/blob/main/README.md)

> Note: Only Linux is supported as a platform, since epoll is not POSIX-compliant.

## Running

> Note: Have to be in workspace root

- Initialize `delay-server`: `$ cargo run --bin delay_server`
- Run the event queue: `$ cargo run --bin event_queue`