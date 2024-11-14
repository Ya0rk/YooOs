use riscv::register::sstatus::Sstatus;
use riscv::register::sstatus::SPP;
use riscv::register::sstatus;
use bitflags::bitflags;
use riscv::register::sstatus::FS;
use core::arch::asm;

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct UserFloatContext{
    pub user_fx   : [f64; 32], // 50-81
    pub fcsr      : u32,       // 
    pub status    : FloatStatus
}

bitflags! {
    #[repr(transparent)]
    #[derive(Clone, Debug)]
    struct FloatStatus: u32 {
        const NEED_SAVE      = 1 << 0;
        const NEED_RESTORE   = 1 << 1;
        const SIGNAL_DIRTY   = 1 << 2;
    }
}

impl UserFloatContext {
    pub fn new() -> Self {
        Self {
            user_fx: [0f64; 32],
            fcsr: FS::Clean as u32,
            status: FloatStatus::empty(),
        }
    }

    pub fn mark_save_if_needed(&mut self, sstatus: Sstatus) {
        if sstatus.fs() == FS::Dirty{
            self.status |= FloatStatus::NEED_SAVE;
            self.status |= FloatStatus::SIGNAL_DIRTY;
        }
    }

    pub fn yield_task(&mut self) {
        self.save();
        self.status.insert(FloatStatus::NEED_RESTORE);
    }

    pub fn encounter_signal(&mut self) {
        self.save();
    }

    /// Save reg -> mem
    pub fn save(&mut self) {
        if !self.status.contains(FloatStatus::NEED_SAVE) {
            return;
        }
        self.status.remove(FloatStatus::NEED_SAVE);
        unsafe {
            let mut _t: usize = 1; // alloc a register but not zero.
            asm!("
            fsd  f0,  0*8({0})
            fsd  f1,  1*8({0})
            fsd  f2,  2*8({0})
            fsd  f3,  3*8({0})
            fsd  f4,  4*8({0})
            fsd  f5,  5*8({0})
            fsd  f6,  6*8({0})
            fsd  f7,  7*8({0})
            fsd  f8,  8*8({0})
            fsd  f9,  9*8({0})
            fsd f10, 10*8({0})
            fsd f11, 11*8({0})
            fsd f12, 12*8({0})
            fsd f13, 13*8({0})
            fsd f14, 14*8({0})
            fsd f15, 15*8({0})
            fsd f16, 16*8({0})
            fsd f17, 17*8({0})
            fsd f18, 18*8({0})
            fsd f19, 19*8({0})
            fsd f20, 20*8({0})
            fsd f21, 21*8({0})
            fsd f22, 22*8({0})
            fsd f23, 23*8({0})
            fsd f24, 24*8({0})
            fsd f25, 25*8({0})
            fsd f26, 26*8({0})
            fsd f27, 27*8({0})
            fsd f28, 28*8({0})
            fsd f29, 29*8({0})
            fsd f30, 30*8({0})
            fsd f31, 31*8({0})
            csrr {1}, fcsr
            sw  {1}, 32*8({0})
        ", in(reg) self,
                inout(reg) _t
            );
        };
    }

    /// Restore mem -> reg
    pub fn restore(&mut self) {
        if !self.status.contains(FloatStatus::NEED_RESTORE) {
            return;
        }
        self.status.remove(FloatStatus::NEED_RESTORE);
        unsafe {
            asm!("
            fld  f0,  0*8({0})
            fld  f1,  1*8({0})
            fld  f2,  2*8({0})
            fld  f3,  3*8({0})
            fld  f4,  4*8({0})
            fld  f5,  5*8({0})
            fld  f6,  6*8({0})
            fld  f7,  7*8({0})
            fld  f8,  8*8({0})
            fld  f9,  9*8({0})
            fld f10, 10*8({0})
            fld f11, 11*8({0})
            fld f12, 12*8({0})
            fld f13, 13*8({0})
            fld f14, 14*8({0})
            fld f15, 15*8({0})
            fld f16, 16*8({0})
            fld f17, 17*8({0})
            fld f18, 18*8({0})
            fld f19, 19*8({0})
            fld f20, 20*8({0})
            fld f21, 21*8({0})
            fld f22, 22*8({0})
            fld f23, 23*8({0})
            fld f24, 24*8({0})
            fld f25, 25*8({0})
            fld f26, 26*8({0})
            fld f27, 27*8({0})
            fld f28, 28*8({0})
            fld f29, 29*8({0})
            fld f30, 30*8({0})
            fld f31, 31*8({0})
            lw  {0}, 32*8({0})
            csrw fcsr, {0}
        ", in(reg) self
            );
        }
    }
}


#[repr(C)]
#[derive(Clone, Debug, Copy)]
pub struct TrapContext {
    pub user_reg : [usize; 32], // 0 - 31
    pub sstatus  : Sstatus,     // 32
    pub sepc     : usize,       // 33
    
    // 内核 -> 用户,要保存的
    pub k_sp : usize,           // 34
    pub k_ra : usize,           // 35
    pub k_s  : [usize; 12],     // 36 - 47
    pub k_fp : usize,           // 48
    pub k_tp : usize,           // 49

    // flaot regs
    pub user_fx : UserFloatContext
}

impl TrapContext {
    // 设置sp寄存器
    pub fn set_sp(&mut self, sp: usize) {
        self.user_reg[2] = sp;
    }

    pub fn new(entry: usize, sp : usize) -> Self {
        let mut sstatus = sstatus::read();
        sstatus.set_spp(SPP::User);
        // 禁止处理器在特权模式下响应外部中断
        sstatus.set_sie(false);
        // 从特权模式返回到用户模式时，禁用中断
        sstatus.set_spie(false);
        let mut context = Self {
            user_reg: [0; 32],
            sstatus,
            sepc: entry,
            k_sp : 0, 
            k_ra : 0,
            k_s  : [0; 12],
            k_fp : 0,
            k_tp : 0,
            user_fx: UserFloatContext::new(),
        };
        context.set_sp(sp);

        context
    }
}