use crate::runtime::reactor::init_reactor;
pub use executor::Executor;
pub use waker::Waker;

mod executor;
pub(crate) mod reactor;
pub(crate) mod waker;

pub fn init_runtime() -> Executor {
    init_reactor();
    let executor = Executor::default();
    println!("Runtime started");
    executor
}
