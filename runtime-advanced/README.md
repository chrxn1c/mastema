# Runtime with Reactor-Waker-Executor components

Loose coupling between Executor and Reactor by adding Waker component. Executor is still single-threaded.

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
- Run the event queue: `$ cargo run --bin runtime-advanced`