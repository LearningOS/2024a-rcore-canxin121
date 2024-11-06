//! Implementation of syscalls
//!
//! The single entry point to all system calls, [`syscall()`], is called
//! whenever userspace wishes to perform a system call using the `ecall`
//! instruction. In this case, the processor raises an 'Environment call from
//! U-mode' exception, which is handled as one of the cases in
//! [`crate::trap::trap_handler`].
//!
//! For clarity, each single syscall is implemented as its own function, named
//! `sys_` then the name of the syscall. You can find functions like this in
//! submodules, and you should also implement syscalls this way.

/// write syscall
const SYSCALL_WRITE: usize = 64;
/// exit syscall
const SYSCALL_EXIT: usize = 93;

mod fs;
mod process;

use fs::*;
/// 该模块用于处理系统调用相关的功能。
/// 
/// 这里导入了 `process` 模块中的所有内容，
/// 以便在当前模块中使用进程相关的功能。
///
/// 具体的系统调用实现可以在这个模块中找到，
/// 包括进程管理、内存管理、文件系统操作等。
///
/// # 示例
///
/// ```rust
/// use process::*;
/// ```
///
/// 这个示例展示了如何导入 `process` 模块中的所有内容。
use process::*;
/// handle syscall exception with `syscall_id` and other arguments
pub fn syscall(syscall_id: usize, args: [usize; 3]) -> isize {
    match syscall_id {
        SYSCALL_WRITE => sys_write(args[0], args[1] as *const u8, args[2]),
        SYSCALL_EXIT => sys_exit(args[0] as i32),
        _ => panic!("Unsupported syscall_id: {}", syscall_id),
    }
}
