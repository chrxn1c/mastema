pub enum PollState<T> {
    Ready(T),
    Pending,
}

pub trait Future {
    type Output;
    fn poll(&mut self) -> PollState<Self::Output>;
}
