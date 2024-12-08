#![feature(associated_type_defaults)]
#![feature(naked_functions)]

use std::arch::asm;
const DEFAULT_STACK_SIZE: usize = 1024 * 1024 * 2;
const MAX_THREADS: usize = 4;
static mut RUNTIME: usize = 0;
pub struct Runtime {
    threads: Vec<Thread>,
    current: usize,
}

impl Runtime {
    pub fn new() -> Self {
        let base_thread = Thread::new_with_state(State::Running);
        let mut threads = vec![base_thread];

        let mut available_threads: Vec<Thread> = (1..MAX_THREADS).map(|_| Thread::new()).collect();
        threads.append(&mut available_threads);
        Runtime {
            threads,
            current: 0,
        }
    }

    pub fn init(&self) {
        unsafe {
            let raw_pointer: *const Runtime = self;
            RUNTIME = raw_pointer as usize;
        }
    }

    pub fn run(&mut self) -> ! {
        while self.coro_yield() {}
        std::process::exit(0);
    }

    fn coro_return(&mut self) {
        // if calling thread is not a base thread
        if self.current != 0 {
            self.threads[self.current].state = State::Available;
            self.coro_yield();
        }
    }

    #[inline(never)]
    fn coro_yield(&mut self) -> bool {
        let mut position = self.current;

        while self.threads[position].state != State::Ready {
            position += 1;
            if position == self.threads.len() {
                position = 0;
            }
            if position == self.current {
                return false;
            }
        }

        if self.threads[self.current].state != State::Available {
            self.threads[self.current].state = State::Ready;
        }

        self.threads[position].state = State::Running;
        let old_position = self.current;

        unsafe {
            let old: *mut ThreadContext = &mut self.threads[old_position].context;
            let new: *const ThreadContext = &self.threads[position].context;

            asm!("call switch", in("rdi") old, in("rsi") new, clobber_abi("C"));
        }

        // self.threads.len() > 0
        unreachable!()
    }

    pub fn spawn(&mut self, func: fn()) {
        let available_thread = self
            .threads
            .iter_mut()
            .find(|thread| thread.state == State::Available)
            .expect("No thread with state 'Available' found");

        let thread_size = available_thread.stack.len();

        unsafe {
            let stack_pointer = available_thread
                .stack
                .as_mut_ptr()
                .offset(thread_size as isize);

            // 16-bytes alignment
            let stack_pointer = (stack_pointer as usize & !15) as *mut u8;

            std::ptr::write(stack_pointer.offset(-16) as *mut u64, guard as u64);
            std::ptr::write(stack_pointer.offset(-24) as *mut u64, skip as u64);
            std::ptr::write(stack_pointer.offset(-32) as *mut u64, func as u64);
        }

        available_thread.state = State::Ready;
    }
}
fn guard() {
    unsafe {
        let runtime_pointer = RUNTIME as *mut Runtime;
        (*runtime_pointer).coro_return();
    }
}

#[naked]
unsafe extern "C" fn skip() {
    asm!("ret", options(noreturn))
}

#[naked]
#[no_mangle]
unsafe extern "C" fn switch() {
    asm!(
        "mov [rdi + 0x00], rsp",
        "mov [rdi + 0x08], r15",
        "mov [rdi + 0x10], r14",
        "mov [rdi + 0x18], r13",
        "mov [rdi + 0x20], r12",
        "mov [rdi + 0x28], rbx",
        "mov [rdi + 0x30], rbp",
        "mov rsp, [rsi + 0x00]",
        "mov r15, [rsi + 0x08]",
        "mov r14, [rsi + 0x10]",
        "mov r13, [rsi + 0x18]",
        "mov r12, [rsi + 0x20]",
        "mov rbx, [rsi + 0x28]",
        "mov rbp, [rsi + 0x30]",
        "ret",
        options(noreturn)
    );
}
#[derive(Eq, PartialEq, Debug)]
enum State {
    Available,
    Running,
    Ready,
}

struct Thread {
    stack: Vec<u8>,
    context: ThreadContext,
    state: State,
}

impl Thread {
    fn new() -> Self {
        Thread {
            // SAFETY: pushing/popping from the vector can cause
            // reallocating, therefore Vec<T>::into_boxed_slice()
            // is to be considered.
            stack: vec![0_u8; DEFAULT_STACK_SIZE],

            context: Default::default(),
            state: State::Available,
        }
    }

    fn new_with_state(state: State) -> Self {
        Thread {
            stack: vec![0_u8; DEFAULT_STACK_SIZE],
            context: Default::default(),
            state,
        }
    }
}
#[derive(Debug, Default)]
#[repr(C)]
struct ThreadContext {
    rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}
fn main() {
    let mut runtime = Runtime::new();
    runtime.init();

    runtime.spawn(|| {
        println!("Thread 1 is starting..");
        let thread_id = 1;
        for counter in 0..10 {
            println!("Thread: {thread_id}, counter: {counter}");
            yield_thread();
        }

        println!("Thread 1 has finished..");
    });

    runtime.spawn(|| {
        println!("Thread 2 is starting..");
        let thread_id = 2;

        for counter in 0..15 {
            println!("Thread: {thread_id}, counter: {counter}");
            yield_thread();
        }

        println!("Thread 2 has finished..");
    });

    runtime.run();
}

pub fn yield_thread() {
    // SAFETY: Can cause UB if Runtime is not initialized
    // Or if Runtime is dropped.
    unsafe {
        let runtime_pointer = RUNTIME as *mut Runtime;
        (*runtime_pointer).coro_yield();
    }
}
