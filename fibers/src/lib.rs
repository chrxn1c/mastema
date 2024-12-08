#![feature(associated_type_defaults)]

use crate::stack_swap::{hello, SystemVABI, ThreadContext, ABI};

mod stack_swap;
fn do_thread_swap() {
    let mut context = ThreadContext::default();

    let mut stack = vec![0u8; context.size() as usize];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(context.size() as isize);
        let stack_bottom_aligned = (stack_bottom as usize & !15) as *mut u8;

        std::ptr::write(
            stack_bottom_aligned.offset((SystemVABI::alignment() as i32 * -1) as isize) as *mut u64,
            hello as u64,
        );
        context.rsp =
            stack_bottom_aligned.offset((SystemVABI::alignment() as i32 * -1) as isize) as u64;
        context.switch_on();
    }
}
