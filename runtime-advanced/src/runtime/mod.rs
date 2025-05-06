use crate::Future;
use crate::future::PollState;
pub use executor::Executor;
use mio::{Events, Poll, Registry};
use std::sync::OnceLock;
pub use waker::Waker;
mod executor;
mod reactor;
pub(crate) mod waker;

static REGISTRY: OnceLock<Registry> = OnceLock::new();

pub fn registry() -> &'static Registry {
    REGISTRY.get().expect("Called outside of runtime context")
}

pub struct Runtime {
    poll: Poll,
}

impl Runtime {
    pub fn new() -> Self {
        let poll = Poll::new().unwrap();
        let registry = poll.registry().try_clone().unwrap();
        REGISTRY.set(registry).unwrap();
        Self { poll }
    }

    pub fn block_on(&mut self, future: impl Future<Output = ()>) {
        let mut future = future;
        loop {
            match future.poll() {
                PollState::Ready(_) => break,
                PollState::Pending => {
                    println!("Scheduling over tasks..");
                    let mut events = Events::with_capacity(100);
                    self.poll.poll(&mut events, None).unwrap()
                }
            }
        }
    }
}

pub fn init() -> Executor {
    reactor::start();
    Executor::default()
}
