//! File and filesystem-related syscalls

const FD_STDOUT: usize = 1;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel: sys_write");
    match fd {
        FD_STDOUT => {
            // 将一个原始指针和长度转换为一个切片。
            //
            // # Safety
            //
            // 这个函数是 `unsafe` 的，因为它假设传入的指针 `buf` 是有效的，并且指向的内存区域
            // 在长度 `len` 范围内是有效的。调用者必须确保这些条件成立，否则可能会导致未定义行为。
            //
            // # 参数
            //
            // - `buf`: 指向内存区域的原始指针。
            // - `len`: 切片的长度。
            //
            // # 返回值
            //
            // 返回一个切片，该切片包含从 `buf` 开始的 `len` 个元素。
            //
            // # 示例
            //
            // ```rust
            // let ptr: *const u8 = 0x1000 as *const u8;
            // let len = 10;
            // let slice = unsafe { core::slice::from_raw_parts(ptr, len) };
            // ```
            let slice = unsafe { core::slice::from_raw_parts(buf, len) };
            let str = core::str::from_utf8(slice).unwrap();
            print!("{}", str);
            len as isize
        }
        _ => {
            panic!("Unsupported fd in sys_write!");
        }
    }
}
