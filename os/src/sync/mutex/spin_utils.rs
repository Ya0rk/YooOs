use riscv::register::sstatus;
use self::spin::SpinMutex;

// 函数申明============================================================
// 判断sie是否开启：开启->允许supervisor mode中断
fn sie_is_enabled() -> bool;
fn disable_sie();
fn enable_sie();

// 保留中断之前sie寄存器状态==============================================
pub struct SiePreserver {
    sie_old : bool
}

impl SiePreserver {
    pub fn new() -> SiePreserver {
        let sie_status : bool = sie_is_enabled();
        disable_sie();
        SiePreserver{ sie_old : sie_status}
    }
}

impl Drop for SiePreserver {
    fn drop(&mut self) {
        // 如果之前保存的 SIE 状态是开启的，则在销毁时重新开启 SIE
        if self.sie_old { enable_sie(); }
    }
}

// 辅助函数：查看+控制 sie 寄存器状态========================================
#[inline]
#[cfg(target_arch = "riscv64")]
pub fn sie_is_enabled() -> bool {
    sstatus.read().sie()
}

#[inline]
#[cfg(target_arch = "riscv64")]
pub fn disable_sie() {
    unsafe {
        sstatus.modify(|s| s.set_sie(false));
    }
}

#[inline]
#[cfg(target_arch = "riscv64")]
pub fn enable_sie() {
    unsafe {
        sstatus.modify(|s| s.set_sie(true));
    }
}

// mutex trait ========================================================
pub trait MutexSupport {
    type GuardData;
    fn before_lock() -> Self::GuardData;
    fn after_unlock(_ : &mut Self::GuardData);
}

// 普通的spin，可以被中断的===============================================
pub struct NormalSpin ;

impl MutexSupport for NormalSpin {
    type GuardData = ();
    fn before_lock() -> () {
        ()
    }
    fn after_unlock(_ : &mut ()) {}
}

// spin with no interrupt===============================================
pub struct NoInterruptSpin ;

impl MutexSupport for NoInterruptSpin {
    type GuardData = SiePreserver;
    #[inline(always)]
    fn before_lock() -> Self::GuardData {
        SiePreserver::new()
    }
    #[inline(always)]
    fn after_unlock(_ : &mut Self::GuardData) {}
}
