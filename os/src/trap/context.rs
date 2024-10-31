use riscv::register::sstatus::Sstatus;
use riscv::register::sstatus::SPP;
use riscv::register::sstatus;

#[repr(C)]
#[derive(Clone, Debug)]
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
    // todo
    // pub flaot_context : FloatContext;
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
            // flaot_context: FloatContext::new(),
        };
        context.set_sp(sp);

        context
    }
}