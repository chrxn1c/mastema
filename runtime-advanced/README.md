# Runtime with Reactor-Waker-Executor components

Loose coupling between Executor and Reactor by adding Waker component. Executor is still single-threaded.

> Note: All platforms are supported

## Running

> Note: Have to be in workspace root

- Initialize `delay-server`: `$ cargo run --bin delay_server`
- Run the event queue: `$ cargo run --bin runtime-advanced`