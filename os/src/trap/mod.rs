// mod.rs
//! 中断处理功能
//!
//! 对于 rCore，我们有一个单一的中断入口，即 `__alltraps`。在 [`init()`] 初始化时，我们将 `stvec` CSR 设置为指向它。
//!
//! 所有中断都通过 `__alltraps`，它在 `trap.S` 中定义。汇编代码仅做足够的工作来恢复内核空间上下文，确保 Rust 代码安全运行，并将控制权转移到 [`trap_handler()`]。
//!
//! 然后根据具体的异常调用不同的功能。例如，定时器中断触发任务抢占，系统调用则转到 [`syscall()`]。

mod context;

use crate::batch::run_next_app;
use crate::syscall::syscall;
use core::arch::global_asm;
use riscv::register::{
    mtvec::TrapMode,
    scause::{self, Exception, Trap},
    stval, stvec,
};

// 包含汇编代码 `trap.S`
global_asm!(include_str!("trap.S"));

/// 初始化 CSR `stvec` 为 `__alltraps` 的入口
pub fn init() {
    extern "C" {
        fn __alltraps();
    }
    unsafe {
        // 设置 stvec 寄存器为 __alltraps 的地址，使用直接模式
        stvec::write(__alltraps as usize, TrapMode::Direct);
    }
}

#[no_mangle]
/// 处理来自用户空间的中断、异常或系统调用
pub fn trap_handler(cx: &mut TrapContext) -> &mut TrapContext {
    debug!("Entering trap_handler");
    // 读取 trap 原因
    let scause = scause::read();
    // 读取 stval 寄存器的值
    let stval = stval::read();
    match scause.cause() {
        // 用户环境调用异常(也就是程序调用ecall来实现一些内核态才能做到的操作,可以说 不是异常, 而是调用)
        Trap::Exception(Exception::UserEnvCall) => {
            // 增加 sepc 寄存器的值以跳过 ecall 指令
            cx.sepc += 4;
            // 调用系统调用处理函数，并将返回值存储在 x[10] 寄存器中
            cx.x[10] = syscall(cx.x[17], [cx.x[10], cx.x[11], cx.x[12]]) as usize;
        }
        // 存储错误或存储页错误异常
        Trap::Exception(Exception::StoreFault) | Trap::Exception(Exception::StorePageFault) => {
            println!("[kernel] PageFault in application, kernel killed it.");
            // 运行下一个应用程序
            run_next_app();
        }
        // 非法指令异常
        Trap::Exception(Exception::IllegalInstruction) => {
            println!("[kernel] IllegalInstruction in application, kernel killed it.");
            // 运行下一个应用程序
            run_next_app();
        }
        // 其他不支持的 trap
        _ => {
            panic!(
                "Unsupported trap {:?}, stval = {:#x}!",
                scause.cause(),
                stval
            );
        }
    }
    cx
}

// 导出 TrapContext 模块
pub use context::TrapContext;
