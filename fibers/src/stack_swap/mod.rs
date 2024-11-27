#![feature(associated_type_defaults)]
use std::arch::asm;

#[derive(Debug)]
#[repr(C)]
pub struct ThreadContext {
    rsp: u64,
}

fn hello() -> ! {
    println!("Waking up on a new stack");
    loop {}
}

impl ThreadContext {
    const SIZE: u32 = 32;
    pub unsafe fn switch_to_stack(new_stack: *const Self) {
        /// no way to intervene with rti by hand
        /// so have to abuse callee-saved register to insert the needed address to rpi
        asm!(
            "mov rsp, [{0} + 0x00]",
            "ret",
            in(reg) new_stack,
        )
    }

    pub fn size(&self) -> u32 {
        Self::SIZE
    }
}

pub trait ABI {
    type Alignment = u32;
}

/*impl<T> ABI<T> {
    pub fn alignment() -> Self::
}

impl SystemVAlignment {
    pub const
}*/
