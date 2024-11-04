pub mod spin;
pub mod spin_utils;

use self::spin_utils::NormalSpin;
use self::spin_utils::NoInterruptSpin;
use self::spin::SpinMutex;

// 各种各样的spin：不能被中断的 + 可以被中断的=============================
pub type SpinLock<T> = SpinMutex<T, NormalSpin>;
pub type NoInterruptSpinLock<T> = SpinMutex<T, NoInterruptSpin>;