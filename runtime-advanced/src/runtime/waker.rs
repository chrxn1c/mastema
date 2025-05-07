use std::sync::{Arc, Mutex};
use std::thread::Thread;

#[derive(Clone)]
pub struct Waker {
    pub(crate) thread: Thread,
    pub(crate) task_id: usize,
    pub(crate) ready_queue: Arc<Mutex<Vec<usize>>>,
}

impl Waker {
    pub fn wake(&self) {
        self.ready_queue
            .lock()
            .map(|mut x| x.push(self.task_id))
            .unwrap();
        self.thread.unpark()
    }
}
