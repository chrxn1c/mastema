mod coroutine;
mod future;
mod http;

use crate::coroutine::Coroutine;
use crate::future::{Future, PollState};
use std::thread;
use std::time::Duration;

pub fn async_main() {
    let mut coroutine = Coroutine::new();

    loop {
        match coroutine.poll() {
            PollState::Ready(_) => {
                break;
            }
            PollState::Pending => {
                println!("Scheduling over tasks...")
            }
        }
        thread::sleep(Duration::from_millis(100));
    }
}
