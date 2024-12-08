use std::arch::asm;

#[derive(Debug, Default)]
#[repr(C)]
pub struct ThreadContext {
    pub rsp: u64,
}

pub(super) fn hello() -> ! {
    println!("Waking up on a new stack");
    loop {}
}

impl ThreadContext {
    const SIZE: u32 = 32;
    pub unsafe fn switch_on(&self) {
        /// no way to intervene with rti by hand
        /// so have to abuse callee-saved register to insert the needed address to rpi
        asm!(
            "mov rsp, [{0} + 0x00]",
            "ret",
            in(reg) self
        )
    }

    pub fn size(&self) -> u32 {
        Self::SIZE
    }
}

pub trait ABI {
    type Alignment = u32;
    fn alignment() -> Self::Alignment;
}

pub struct SystemVABI;
impl ABI for SystemVABI {
    type Alignment = u32;

    fn alignment() -> <SystemVABI as ABI>::Alignment {
        16
    }
}