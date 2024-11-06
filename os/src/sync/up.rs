//! 单处理器内部可变性原语
use core::cell::{RefCell, RefMut};

/// 将一个静态数据结构包装在里面，这样我们就可以在没有任何 `unsafe` 的情况下访问它。
///
/// 我们应该只在单处理器中使用它。
///
/// 为了获得内部数据的可变引用，调用 `exclusive_access` 方法。
pub struct UPSafeCell<T> {
    /// 内部数据
    inner: RefCell<T>,
}

// 为 UPSafeCell 实现 Sync trait，这样它可以安全地在线程间共享。
// 由于我们只在单处理器环境中使用它，所以这是安全的。
unsafe impl<T> Sync for UPSafeCell<T> {}

impl<T> UPSafeCell<T> {
    /// 用户有责任保证内部结构只在单处理器中使用。
    ///
    /// # Safety
    ///
    /// 这个方法是 `unsafe` 的，因为它假设调用者保证在单处理器环境中使用。
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// 如果数据已经被借用，则会触发 panic。
    ///
    /// 这个方法返回一个可变引用，用于访问内部数据。
    pub fn exclusive_access(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
