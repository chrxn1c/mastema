use crate::Future;
use crate::future::PollState;
use crate::runtime::waker::Waker;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

type Task = Box<dyn Future<Output = String>>;

thread_local! {
    static THREAD_LOCAL_EXECUTOR: Executor = Executor::default();
}

#[derive(Default)]
pub struct Executor {
    tasks: RefCell<HashMap<usize, Task>>,
    ready_queue: Arc<Mutex<Vec<usize>>>,
    next_id: Cell<usize>,
}
impl Executor {
    fn pop_ready(&self) -> Option<usize> {
        THREAD_LOCAL_EXECUTOR.with(|this| this.ready_queue.lock().map(|mut x| x.pop()).unwrap())
    }

    fn get_future(&self, task_id: usize) -> Option<Task> {
        THREAD_LOCAL_EXECUTOR.with(|this| this.tasks.borrow_mut().remove(&task_id))
    }

    fn get_waker(&self, task_id: usize) -> Waker {
        Waker {
            task_id,
            thread: thread::current(),
            ready_queue: THREAD_LOCAL_EXECUTOR.with(|this| this.ready_queue.clone()),
        }
    }

    fn insert_task(&self, task_id: usize, task: Task) {
        THREAD_LOCAL_EXECUTOR.with(|this| this.tasks.borrow_mut().insert(task_id, task));
    }

    fn task_count(&self) -> usize {
        THREAD_LOCAL_EXECUTOR.with(|this| this.tasks.borrow().len())
    }

    pub fn spawn(&self, future: impl Future<Output = String> + 'static) {
        THREAD_LOCAL_EXECUTOR.with(|this| {
            let task_id = this.next_id.get();
            this.tasks.borrow_mut().insert(task_id, Box::new(future));
            this.ready_queue
                .lock()
                .map(|mut x| x.push(task_id))
                .unwrap();
            this.next_id.set(task_id + 1)
        });
    }

    pub fn block_on(&mut self, future: impl Future<Output = String> + 'static) {
        self.spawn(future);
        loop {
            while let Some(id) = self.pop_ready() {
                let mut future = match self.get_future(id) {
                    None => continue,
                    Some(future) => future,
                };

                let waker = self.get_waker(id);

                match future.poll(&waker) {
                    PollState::Ready(_) => continue,
                    PollState::Pending => self.insert_task(id, future),
                }
            }

            let task_count = self.task_count();
            let thread_name = thread::current().name().unwrap_or_default().to_string();

            if task_count > 0 {
                println!("Thread {thread_name}: {task_count} pending tasks. Sleep until notified.");
                thread::park();
            } else {
                println!("Thread {thread_name}: all tasks are finished");
                break;
            }
        }
    }
}
