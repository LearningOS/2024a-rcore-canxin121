//! App management syscalls
/// 调用 `run_next_app` 函数以运行下一个应用程序。
///
/// 该函数位于 `crate::batch` 模块中，负责切换并运行下一个待执行的应用程序。
///
/// # 示例
///
/// ```rust
/// use crate::batch::run_next_app;
///
/// // 运行下一个应用程序
/// run_next_app();
/// ```
///
/// # 注意
///
/// 确保在调用此函数之前，所有当前应用程序的状态已正确保存，以避免数据丢失或状态不一致。
use crate::batch::run_next_app;

/// task exits and submit an exit code
pub fn sys_exit(exit_code: i32) -> ! {
    trace!("[kernel] Application exited with code {}", exit_code);
    run_next_app()
}
