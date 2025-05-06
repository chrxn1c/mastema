# Runtime

A simple implementation of non-work-stealing-runtime without waker (so both Executor and Reactor are aware of each other's implementation details)

> Note: All platforms are supported

## Running

> Note: Have to be in workspace root

> Note: On Windows platform, you will see
> ```rust
> Scheduling over tasks..
> Scheduling over tasks..
> ```
> The reason is that Windows emits an extra event when the
> TcpStream is dropped on the server end.

- Initialize `delay-server`: `$ cargo run --bin delay_server`
- Run the event queue: `$ cargo run --bin runtime`