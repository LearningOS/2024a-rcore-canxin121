//! The main module and entrypoint
//!
//! Various facilities of the kernels are implemented as submodules. The most
//! important ones are:
//!
//! - [`trap`]: Handles all cases of switching from userspace to the kernel
//! - [`syscall`]: System call handling and implementation
//!
//! The operating system also starts in this module. Kernel code starts
//! executing from `entry.asm`, after which [`rust_main()`] is called to
//! initialize various pieces of functionality. (See its source code for
//! details.)
//!
//! We then call [`batch::run_next_app()`] and for the first time go to
//! userspace.

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]
#![no_main]
#![feature(panic_info_message)]
#[macro_use]
extern crate log;

use core::arch::global_asm;
#[path = "boards/qemu.rs"]
mod board;
use log::*;
#[macro_use]
mod console;
pub mod batch;
pub mod lang_items;
pub mod logging;
pub mod sbi;
pub mod sync;
pub mod syscall;
pub mod trap;

// 在entry.asm中定义了__start，并且在__start中调用了rust_main
global_asm!(include_str!("entry.asm"));
// 在link_app.S中定义并且引入了一些app和app的信息(数量，地址)
global_asm!(include_str!("link_app.S"));

/// clear BSS segment
fn clear_bss() {
    extern "C" {
        fn sbss();
        fn ebss();
    }
    // 将 sbss 到 ebss 之间的内存清零
    unsafe {
        core::slice::from_raw_parts_mut(sbss as usize as *mut u8, ebss as usize - sbss as usize)
            .fill(0);
    }
}

/// 操作系统的 Rust 入口点
#[no_mangle]
pub fn rust_main() -> ! {
    extern "C" {
        fn stext(); // 代码段起始地址
        fn etext(); // 代码段结束地址
        fn srodata(); // 只读数据段起始地址
        fn erodata(); // 只读数据段结束地址
        fn sdata(); // 数据段起始地址
        fn edata(); // 数据段结束地址
        fn sbss(); // BSS 段起始地址
        fn ebss(); // BSS 段结束地址
        fn boot_stack_lower_bound(); // 栈的下边界
        fn boot_stack_top(); // 栈顶
    }
    clear_bss();
    logging::init();
    println!("[kernel] Hello, world!");
    trace!(
        "[kernel] .text [{:#x}, {:#x})",
        stext as usize,
        etext as usize
    );
    debug!(
        "[kernel] .rodata [{:#x}, {:#x})",
        srodata as usize, erodata as usize
    );
    info!(
        "[kernel] .data [{:#x}, {:#x})",
        sdata as usize, edata as usize
    );
    warn!(
        "[kernel] boot_stack top=bottom={:#x}, lower_bound={:#x}",
        boot_stack_top as usize, boot_stack_lower_bound as usize
    );
    error!("[kernel] .bss [{:#x}, {:#x})", sbss as usize, ebss as usize);
    // 初始化 trap 模块, 这个模块会 在trap.S中创建__alltraps和__restore,并在rust代码中将stvec指向__alltraps
    // 使得后续的中断和异常都会进入trap_handler,并且在trap_handler后进入__restore(这是因为在trap.S中__alltraps调用了trap_handler,调用结束后,下一个程序是__restore)
    trap::init();
    // 初始化了一个全局的AppManager，并且打印了应用程序信息
    // 在初始化时，会读取link_app.S中的app数量信息，app的起始地址信息，并且保存在AppManager中
    // 同时AppManager会记录当前运行的app索引，同时提供切换到下一个app继续运行的功能
    batch::init();
    // 运行下一个应用程序
    batch::run_next_app();
}
