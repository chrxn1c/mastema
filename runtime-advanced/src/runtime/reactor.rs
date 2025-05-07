use crate::runtime::Waker;
use mio::net::TcpStream;
use mio::{Events, Interest, Poll, Registry, Token};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
type Wakers = Arc<Mutex<HashMap<usize, Waker>>>;
pub static REACTOR: OnceLock<Reactor> = OnceLock::new();

pub fn reactor() -> &'static Reactor {
    REACTOR.get().expect("Called outside of runtime context")
}

pub struct Reactor {
    wakers: Wakers,
    registry: Registry,
    next_task_id: AtomicUsize,
}

impl Reactor {
    pub fn new(wakers: Wakers, registry: Registry, next_task_id: AtomicUsize) -> Self {
        Self {
            wakers,
            registry,
            next_task_id,
        }
    }
    pub fn register(&self, stream: &mut TcpStream, interest: Interest, task_id: usize) {
        self.registry
            .register(stream, Token(task_id), interest)
            .unwrap();
    }

    pub fn deregister(&self, stream: &mut TcpStream, task_id: usize) {
        self.wakers.lock().map(|mut x| x.remove(&task_id)).unwrap();
        self.registry.deregister(stream).unwrap();
    }

    /// we should always store the most recent Waker
    pub fn set_waker(&self, waker: &Waker, task_id: usize) {
        let _ = self
            .wakers
            .lock()
            .map(|mut x| x.insert(task_id, waker.clone()).is_none())
            .unwrap();
    }

    pub fn next_task_id(&self) -> usize {
        self.next_task_id.fetch_add(1, Ordering::Relaxed)
    }
}
pub fn init_reactor() {
    let wakers = Arc::new(Mutex::new(HashMap::new()));
    let poll = Poll::new().unwrap();
    let registry = poll.registry().try_clone().unwrap();
    let next_task_id = AtomicUsize::new(1);

    let reactor = Reactor::new(wakers.clone(), registry, next_task_id);

    REACTOR
        .set(reactor)
        .ok()
        .expect("Reactor is already running");
    std::thread::spawn(move || event_loop(poll, wakers));

    println!("Reactor event loop started");
}

pub(crate) fn event_loop(mut poll: Poll, wakers: Wakers) {
    let mut events = Events::with_capacity(100);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            let Token(task_id) = event.token();
            let wakers = wakers.lock().unwrap();

            if let Some(waker) = wakers.get(&task_id) {
                waker.wake();
            }
        }
    }
}
