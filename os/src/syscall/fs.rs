//! File and filesystem-related syscalls

use crate::task::increase_syscall_trace;

const FD_STDOUT: usize = 1;
/// write syscall
const SYSCALL_WRITE: usize = 64;

/// write buf of length `len`  to a file with `fd`
pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    increase_syscall_trace(SYSCALL_WRITE);
    trace!("kernel: sys_write");
    match fd {
        FD_STDOUT => {
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
