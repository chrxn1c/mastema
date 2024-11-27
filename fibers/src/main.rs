use stack_swap::ThreadContext;

mod stack_swap;

fn main() {
    let mut context = ThreadContext::default();
    let mut stack = vec![0u8; context.size()];

    unsafe {
        let stack_bottom = stack.as_mut_ptr().offset(context.size());
        let stack_bottom_aligned = (stack_bottom as usize & !15) as *mut u8;

        std::ptr::write(stack_bottom_aligned.offset())
    }
}
