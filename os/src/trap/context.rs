// context.rs
use riscv::register::sstatus::{self, Sstatus, SPP};

/// Trap Context
/// 陷阱上下文结构体
#[repr(C)]
pub struct TrapContext {
    /// general regs[0..31]
    /// 通用寄存器数组，包含32个usize类型的寄存器
    pub x: [usize; 32],
    /// CSR sstatus
    /// 控制状态寄存器 sstatus
    pub sstatus: Sstatus,
    /// CSR sepc
    /// 控制状态寄存器 sepc
    pub sepc: usize,
}

impl TrapContext {
    /// set stack pointer to x_2 reg (sp)
    /// 将栈指针设置到 x_2 寄存器 (sp)
    pub fn set_sp(&mut self, sp: usize) {
        self.x[2] = sp;
    }

    /// init app context
    /// 初始化应用程序上下文
    /// 这将设置 sstatus 寄存器的 spp 位为用户模式
    /// 并将 sepc 寄存器设置为应用程序的入口地址
    /// sepc 寄存器是一个特殊的寄存器，用于存储异常返回地址
    /// 当异常结束时，处理器将从 sepc 寄存器中读取地址并跳转到该地址
    /// 也就是说，异常结束之后处理器将从sepc开始执行新的指令
    /// 
    /// 作用上来说，这其实是一个初始化操作，将所有的寄存器都初始化为0，将sstatus寄存器设置为用户模式，
    /// 将sepc寄存器设置为应用程序的入口地址，将栈指针设置为应用程序的栈顶地址
    /// 这样做了之后，cpu就可以从应用程序的入口地址开始执行应用程序了
    pub fn app_init_context(entry: usize, sp: usize) -> Self {
        // 读取当前的 sstatus 寄存器值
        let mut sstatus = sstatus::read();
        // 设置之前的特权模式为用户模式
        sstatus.set_spp(SPP::User);
        // 创建一个新的 TrapContext 实例
        let mut cx = Self {
            // 初始化通用寄存器数组为全零
            x: [0; 32],
            // 设置 sstatus 寄存器
            sstatus,
            // 设置应用程序的入口点
            sepc: entry,
        };
        // 设置应用程序的用户栈指针
        cx.set_sp(sp);
        // 返回初始化后的 TrapContext 实例
        cx
    }
}
