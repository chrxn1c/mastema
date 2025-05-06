mod coroutine;
mod future;
mod http;
mod runtime;
use crate::coroutine::Coroutine;
use crate::future::Future;
use crate::runtime::Runtime;

pub fn async_main() {
    let coroutine = Coroutine::new();
    let mut runtime = Runtime::new();
    runtime.block_on(coroutine);
}
