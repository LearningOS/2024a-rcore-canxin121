总结:

1. 修改 TaskControlBlock 来存储系统调用桶计数(syscall_times: [u32; MAX_SYSCALL_NUM])和第一次调用的时间戳(start_time: usize)
2. 添加一个 add_syscall_times(syscall_id: usize)->()函数，用于增加当前任务的相应系统调用次数，并且在 syscall 函数中首先运行 add_syscall_times。
3. 在每个任务首次调用时更新该任务的 start_time 4. 增加 get_current_task_info(\_ti: \*mut TaskInfo)函数，并在 sys_task_info 的调用中使用

问答:
一. RustSBI-QEMU version 0.2.0-alpha.2 下报错如下:

```plaintext
[ERROR] [kernel] PageFault in application, bad addr = 0x0, bad instruction = 0x804003a4, kernel killed it.
[ERROR] [kernel] IllegalInstruction in application, kernel killed it.
[ERROR] [kernel] IllegalInstruction in application, kernel killed it.
```

二.

1.  a0 的值代表的是内核栈上保存的 TrapContext 的地址
    两种使用场景，一是在异常中断结束后恢复到用户栈以继续运行程序，二是复用到应用程序运行前的准备工作
2.  这几行代码恢复了进入中断时保存的 sstatus、sepc 和 sscratch 寄存器的值：
    - sstatus：恢复中断使能和全局中断状态，确保 CPU 能按正确的中断控制设置恢复。
    - sepc：恢复中断前的程序指针，确保程序从中断前的指令位置继续执行。
    - sscratch：恢复用户栈指针，确保从内核返回时，用户态的栈状态不被破坏。
3.  在这段代码中，跳过了 x2 和 x4 寄存器的恢复，原因如下：

    - x2 (stack pointer)：x2 寄存器保存的是栈指针，恢复时通常不需要操作，因为它在进入中断时已经通过 csrrw 指令将 sp 和 sscratch 交换，确保栈指针已被正确保存并在恢复时自动管理。跳过 x2 寄存器的恢复是因为它已经在内核态和用户态切换时被正确处理。

    - x4 (thread pointer, tp)：x4 寄存器在这段代码中被认为是应用程序不使用的寄存器。通常，tp 用于存储线程指针，但如果上下文中没有涉及多线程或线程指针的操作，可以安全地跳过它的保存和恢复。

4.

- `sp` (栈指针)：在这条指令执行后，`sp` 的值会变为用户态栈的地址，因为 `sscratch` 保存了原本的用户栈指针。
- `sscratch`：执行后，`sscratch` 会保存原本的内核栈指针（即 `sp` 的原始值），这样内核栈指针得以保存，方便之后恢复。

5. sret
   执行 `sret` 后，CPU 会从内核态切换回用户态。它通过恢复 `sepc`（程序计数器）和 `sstatus`（状态寄存器）中的值，设置处理器模式为用户模式，并跳转到用户程序中断前的执行位置。
6.

- `sp` (栈指针)：在这条指令执行后，`sp` 的值会变为用户态栈的地址，因为 `sscratch` 保存了原本的用户栈指针。
- `sscratch`：执行后，`sscratch` 会保存原本的内核栈指针（即 `sp` 的原始值），这样内核栈指针得以保存，方便之后恢复。

7.  应用程序调用 ecall 之后发生异常中断，cpu 会硬件将状态从 u 态切换到 s 态

三. 荣誉准则
1. 在完成本次实验的过程（含此前学习的过程）中，我曾分别与 以下各位 就（与本次实验相关的）以下方面做过交流，还在代码中对应的位置以注释形式记录了具体的交流对象及内容：
无
2. 此外，我也参考了 以下资料 ，还在代码中对应的位置以注释形式记录了具体的参考来源及内容：
无
3. 我独立完成了本次实验除以上方面之外的所有工作，包括代码与文档。 我清楚地知道，从以上方面获得的信息在一定程度上降低了实验难度，可能会影响起评分。
4. 我从未使用过他人的代码，不管是原封不动地复制，还是经过了某些等价转换。 我未曾也不会向他人（含此后各届同学）复制或公开我的实验代码，我有义务妥善保管好它们。 我提交至本实验的评测系统的代码，均无意于破坏或妨碍任何计算机系统的正常运转。 我清楚地知道，以上情况均为本课程纪律所禁止，若违反，对应的实验成绩将按“-100”分计。
