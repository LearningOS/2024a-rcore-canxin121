//! batch 子系统

use crate::sync::UPSafeCell;
use crate::trap::TrapContext;
use core::arch::asm;
use lazy_static::*;

// 用户栈大小
const USER_STACK_SIZE: usize = 4096 * 2;
// 内核栈大小
const KERNEL_STACK_SIZE: usize = 4096 * 2;
// 最大应用程序数量
const MAX_APP_NUM: usize = 16;
// 应用程序基地址
const APP_BASE_ADDRESS: usize = 0x80400000;
// 应用程序大小限制
const APP_SIZE_LIMIT: usize = 0x20000;

// 内核栈结构体，按 4096 字节对齐
#[repr(align(4096))]
struct KernelStack {
    data: [u8; KERNEL_STACK_SIZE],
}

// 用户栈结构体，按 4096 字节对齐
#[repr(align(4096))]
struct UserStack {
    data: [u8; USER_STACK_SIZE],
}

// 静态内核栈实例
static KERNEL_STACK: KernelStack = KernelStack {
    data: [0; KERNEL_STACK_SIZE],
};
// 静态用户栈实例
static USER_STACK: UserStack = UserStack {
    data: [0; USER_STACK_SIZE],
};

impl KernelStack {
    // 获取内核栈顶指针
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + KERNEL_STACK_SIZE
    }
    // 将 TrapContext 压入内核栈
    pub fn push_context(&self, cx: TrapContext) -> &'static mut TrapContext {
        let cx_ptr = (self.get_sp() - core::mem::size_of::<TrapContext>()) as *mut TrapContext;
        unsafe {
            *cx_ptr = cx;
        }
        unsafe { cx_ptr.as_mut().unwrap() }
    }
}

impl UserStack {
    // 获取用户栈顶指针
    fn get_sp(&self) -> usize {
        self.data.as_ptr() as usize + USER_STACK_SIZE
    }
}

// 应用程序管理器结构体
struct AppManager {
    // 应用程序数量
    num_app: usize,
    // 当前应用程序索引
    current_app: usize,
    // 应用程序起始地址数组
    app_start: [usize; MAX_APP_NUM + 1],
}

impl AppManager {
    // 打印应用程序信息
    pub fn print_app_info(&self) {
        println!("[kernel] num_app = {}", self.num_app);
        for i in 0..self.num_app {
            println!(
                "[kernel] app_{} [{:#x}, {:#x})",
                i,
                self.app_start[i],
                self.app_start[i + 1]
            );
        }
    }

    // 加载指定应用程序
    // 所做的就是将应用程序的二进制数据复制到 APP_BASE_ADDRESS 处, 并且fence.i来清除指令缓存
    unsafe fn load_app(&self, app_index: usize) {
        if app_index >= self.num_app {
            println!("All apps have been loaded");
            use crate::board::QEMUExit;
            crate::board::QEMU_EXIT_HANDLE.exit_success();
        }
        println!("[kernel] load app_{}", app_index);
        // 清空应用程序区域
        // 应用程序需要存放在APP_BASE_ADDRESS到APP_BASE_ADDRESS+APP_SIZE_LIMIT的区域
        // 清空时直接将这个区域的所有字节都置为0

        unsafe {
            core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, APP_SIZE_LIMIT).fill(0)
        };
        // 加载应用程序
        // 首先读取应用程序的二进制数据，然后将其复制到APP_BASE_ADDRESS处
        let app_src = unsafe {
            core::slice::from_raw_parts(
                self.app_start[app_index] as *const u8,
                self.app_start[app_index + 1] - self.app_start[app_index],
            )
        };
        // 拿到应用程序区域的从 APP_BASE_ADDRESS到APP_BASE_ADDRESS+当前app所需大小 的可变切片
        // 这样后面可以直接使用safe的copy_from_slice方法，而不是使用unsafe的copy_from_nonoverlapping方法
        let app_dst =
            unsafe { core::slice::from_raw_parts_mut(APP_BASE_ADDRESS as *mut u8, app_src.len()) };
        app_dst.copy_from_slice(app_src);
        // 内存屏障，确保指令内存的写入被观察到
        // 这个指令会强制CPU将所有写缓冲区的数据刷新到内存中
        // 这样在这个指令之前的所有内存写入操作都会被观察到
        asm!("fence.i");
    }

    // 获取当前应用程序索引
    pub fn get_current_app(&self) -> usize {
        self.current_app
    }

    // 移动到下一个应用程序
    pub fn move_to_next_app(&mut self) {
        self.current_app += 1;
    }
}

// 使用 lazy_static 创建静态应用程序管理器实例
lazy_static! {
    static ref APP_MANAGER: UPSafeCell<AppManager> = unsafe {
        UPSafeCell::new({
            extern "C" {
                fn _num_app();
            }
            let num_app_ptr = _num_app as usize as *const usize;
            let num_app: usize = num_app_ptr.read_volatile();
            let mut app_start: [usize; MAX_APP_NUM + 1] = [0; MAX_APP_NUM + 1];
            let app_start_raw: &[usize] =
                core::slice::from_raw_parts(num_app_ptr.add(1), num_app + 1);
            app_start[..=num_app].copy_from_slice(app_start_raw);
            AppManager {
                num_app,
                current_app: 0,
                app_start,
            }
        })
    };
}

/// 初始化 batch 子系统
// 这个函数实际上1.初始化了全局的APP_MANAGER变量, 2.打印了应用程序信息
pub fn init() {
    print_app_info();
}

/// 打印应用程序信息
pub fn print_app_info() {
    APP_MANAGER.exclusive_access().print_app_info();
}

/// 运行下一个应用程序
pub fn run_next_app() -> ! {
    debug!("Entering run_next_app");
    let mut app_manager = APP_MANAGER.exclusive_access();
    let current_app = app_manager.get_current_app();
    unsafe {
        // 将当前应用程序加载到内存中
        app_manager.load_app(current_app);
    }
    app_manager.move_to_next_app();
    // 因为run_next_app不是在rust中结束调用的，而是在__restore中结束的，所以这里不会自动调用drop
    drop(app_manager);
    // 通过调用__restore来切换到用户态，以便运行刚刚load加载到内存的应用程序
    // 这里实际上是为了复用之前的代码，因为在trap.S中，当发生trap时，会调用__alltraps，然后调用trap_handler，最后调用__restore来切换到用户态
    // 而这里需要的是切换回用户态，直接调用__restore即可
    // __restore需要的参数是一个
    extern "C" {
        fn __restore(cx_addr: usize);
    }
    
    unsafe {
        __restore(KERNEL_STACK.push_context(TrapContext::app_init_context(
            APP_BASE_ADDRESS,
            USER_STACK.get_sp(),
        )) as *const _ as usize);
    }
    panic!("can't reach batch::run_current_app");
}
