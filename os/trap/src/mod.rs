use riscv::register::stvec::TrapMode;

mod context;
mod kernel_trap;
mod user_trap;

// 初始化，启动时，main函数中要首先初始化trap
pub fn init() {
    set_kernel_trap();
}

extern "C" {
    fn __trap_from_kernel();
    fn __trap_from_user();
}

fn set_trap(addr : usize) {
    unsafe {
        stvec::write(addr, TrapMode::Direct);
    }
}

// 修改stvec寄存器，使stvec指向正确的 'user Trap' 地址
pub fn set_user_trap() {
    set_trap(__trap_from_user as usize);
}

// 修改stvec寄存器，使stvec指向正确的 'kernel Trap' 地址
pub fn set_kernel_trap() {
    set_trap(__trap_from_kernel as usize);
}