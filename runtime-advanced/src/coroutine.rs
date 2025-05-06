use crate::future::{Future, PollState};
use crate::http::HttpClient;

enum State {
    Start,
    FirstAwaitPoint(Box<dyn Future<Output = String>>),
    SecondAwaitPoint(Box<dyn Future<Output = String>>),
    Resolved,
}

pub struct Coroutine(State);

impl Coroutine {
    pub fn new() -> Self {
        Self { 0: State::Start }
    }
}

impl Future for Coroutine {
    type Output = ();

    fn poll(&mut self) -> PollState<Self::Output> {
        loop {
            match self.0 {
                State::Start => {
                    println!("Initiating Coroutine");
                    let future = Box::new(HttpClient::get_request("/600/HelloWorld1".into()));
                    self.0 = State::FirstAwaitPoint(future)
                }
                State::FirstAwaitPoint(ref mut future) => match future.poll() {
                    PollState::Ready(text) => {
                        println!("Got the following text inside the first await point: {text}");
                        let future = Box::new(HttpClient::get_request("/400/HelloWorld2".into()));
                        self.0 = State::SecondAwaitPoint(future);
                    }
                    PollState::Pending => break PollState::Pending,
                },
                State::SecondAwaitPoint(ref mut future) => match future.poll() {
                    PollState::Ready(text) => {
                        println!("Got the following text inside the second await point: {text}");
                        self.0 = State::Resolved;
                        break PollState::Ready(());
                    }
                    PollState::Pending => break PollState::Pending,
                },
                State::Resolved => panic!("Bro this future is not fuse"),
            }
        }
    }
}
