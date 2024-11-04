use crate::time::boot_time;
use crate::utils::async_utils::SendWrapper;
use self::MutexSupport;
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

mod myutil;

// 对 SpinMutex 的锁定表示
// 当持有该结构体时，意味着对应的 SpinMutex 处于锁定状态
struct MutexGuard<'a, T: ?Sized, S: MutexSupport> {
    mutex: &'a SpinMutex<T, S>,
    support_guard: S::GuardData,
}

// 保护共享数据的互斥锁
pub struct SpinMutex<T: ?Sized, S: MutexSupport> {
    lock: AtomicBool,         // 互斥锁的状态（锁定或未锁定）
    _marker: PhantomData<S>,
    data: UnsafeCell<T>,
}

// 禁止 MutexGuard 结构体在多线程环境中自动同步或发送，防止数据竞争
impl<'a, T: ?Sized, S: MutexSupport> !Sync for MutexGuard<'a, T, S> {}
impl<'a, T: ?Sized, S: MutexSupport> !Send for MutexGuard<'a, T, S> {}

unsafe impl<T: ?Sized + Send, S: MutexSupport> Sync for SpinMutex<T, S> {}
unsafe impl<T: ?Sized + Send, S: MutexSupport> Send for SpinMutex<T, S> {}

impl<'a, T, S: MutexSupport> SpinMutex<T, S> {
    /// Construct a SpinMutex
    pub const fn new(user_data: T) -> Self {
        SpinMutex {
            lock: AtomicBool::new(false),
            _marker: PhantomData,
            data: UnsafeCell::new(user_data),
        }
    }

    // 自旋等待锁的释放, 如果自旋超过 上限时间 则检测死锁
    #[inline(always)]
    fn wait_unlock(&self) {
        let lock_time = cpu_time();
        while self.lock.load(Ordering::Relaxed) {
            core::hint::spin_loop();// 提示 CPU 处于自旋状态
            if cpu_time() - lock_time > Duration::from_secs(15) {
                println!("spin mutex dead lock!!");
                panic!("spinMutex: deadlock detected!\n");
            }
        }
    }

    /// Note that the locked data cannot step over `await`,
    /// i.e. cannot be sent between thread.
    #[inline(always)]
    pub fn lock(&self) -> impl DerefMut<Target = T> + '_ {
        let _guard = S::before_lock();
        while self.lock.load(Ordering::Relaxed){
            self.wait_unlock();
        }
        self.lock.store(true, Ordering::Acquire);
        MutexGuard {
            mutex: self,
            support_guard: _guard,
        }
    }

    /// This is highly unsafe.
    /// You should ensure that context switch won't happen during
    /// the locked data's lifetime.
    #[inline(always)]
    pub unsafe fn sent_lock(&self) -> impl DerefMut<Target = T> + '_ {
        SendWrapper::new(self.lock())
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Deref for MutexGuard<'a, T, S> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &T {
        unsafe { &*self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> DerefMut for MutexGuard<'a, T, S> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<'a, T: ?Sized, S: MutexSupport> Drop for MutexGuard<'a, T, S> {
    /// The dropping of the MutexGuard will release the lock it was created from.
    #[inline(always)]
    fn drop(&mut self) {
        // debug_assert!(self.mutex.lock.load(Ordering::Relaxed));
        self.mutex.lock.store(false, Ordering::Release);
        S::after_unlock(&mut self.support_guard);
    }
}
