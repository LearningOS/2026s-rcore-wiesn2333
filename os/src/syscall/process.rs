//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{translated_byte_buffer, MapPermission, PTEFlags, PageTable, VirtAddr},
    task::{
        change_program_brk, current_memory_set, current_user_token, exit_current_and_run_next,
        get_syscall_stat, suspend_current_and_run_next,
    },
    timer::get_time_us,
};
#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// task exits and submit an exit code
pub fn sys_exit(_exit_code: i32) -> ! {
    trace!("kernel: sys_exit");
    exit_current_and_run_next();
    panic!("Unreachable in sys_exit!");
}

/// current task gives up resources for other tasks
pub fn sys_yield() -> isize {
    trace!("kernel: sys_yield");
    suspend_current_and_run_next();
    0
}

/// YOUR JOB: get time with second and microsecond
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TimeVal`] is splitted by two pages ?
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    if ts.is_null() {
        return -1;
    }

    let va = VirtAddr::from(ts as usize);

    let time_val_bytes = {
        let us = get_time_us();
        let time_val = TimeVal {
            sec: us / 1_000_000,
            usec: us % 1_000_000,
        };
        unsafe {
            core::slice::from_raw_parts(
                &time_val as *const _ as *const u8,
                core::mem::size_of::<TimeVal>(),
            )
        }
    };

    let buffers = translated_byte_buffer(
        current_user_token(),
        va.0 as *const u8,
        core::mem::size_of::<TimeVal>(),
    );
    {
        let mut i = 0;
        for buffer in buffers {
            for byte in time_val_bytes.iter() {
                buffer[i] = *byte;
                i += 1;
            }
        }
    }
    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(trace_request: usize, id: usize, data: usize) -> isize {
    trace!("kernel: sys_trace");
    match trace_request {
        0 => {
            let va = VirtAddr::from(id);
            let page_table = PageTable::from_token(current_user_token());
            let pte = page_table.translate(va.floor());
            if let Some(pte) = pte {
                let flags = pte.flags();
                if flags.contains(PTEFlags::U | PTEFlags::R) {
                    let byte = pte.ppn().get_bytes_array()[va.page_offset()] as usize;
                    byte as isize
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        1 => {
            let va = VirtAddr::from(id);
            let page_table = PageTable::from_token(current_user_token());
            let pte = page_table.translate(va.floor());
            if let Some(pte) = pte {
                let flags = pte.flags();
                if flags.contains(PTEFlags::U | PTEFlags::W) {
                    pte.ppn().get_bytes_array()[va.page_offset()] = data as u8;
                    0
                } else {
                    -1
                }
            } else {
                -1
            }
        }
        2 => get_syscall_stat(id) as isize,
        _ => -1,
    }
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, prot: usize) -> isize {
    trace!(
        "sys_mmap: start={:#x}, len={:#x}, prot={:#x} (binary:{:b})",
        start,
        len,
        prot,
        prot
    );

    if start % PAGE_SIZE != 0 {
        return -1;
    }
    if prot > 0x7 {
        return -1;
    }
    if prot == 0 {
        return -1;
    }

    if len == 0 {
        return 0;
    }

    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let mapped_len = page_count * PAGE_SIZE;
    let end = match start.checked_add(mapped_len) {
        Some(val) => val,
        None => {
            return -1;
        }
    };

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(end);

    let mut map_perm = MapPermission::U;
    if prot & 0x1 != 0 {
        map_perm |= MapPermission::R;
    }
    if prot & 0x2 != 0 {
        map_perm |= MapPermission::W;
    }
    if prot & 0x4 != 0 {
        map_perm |= MapPermission::X;
    }

    let memory_set = current_memory_set();
    let result = memory_set.map(start_va, end_va, map_perm);
    result
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    if start % PAGE_SIZE != 0 {
        trace!("sys_munmap: start address not page aligned");
        return -1;
    }
    if len == 0 {
        trace!("sys_munmap: zero length unmapping");
        return 0;
    }

    let page_count = (len + PAGE_SIZE - 1) / PAGE_SIZE;
    let unmapped_len = page_count * PAGE_SIZE;
    let end = start + unmapped_len;

    let start_va = VirtAddr::from(start);
    let end_va = VirtAddr::from(end);
    let memory_set = current_memory_set();
    let result = memory_set.unmap(start_va, end_va);
    result
}
/// change data segment size
pub fn sys_sbrk(size: i32) -> isize {
    trace!("kernel: sys_sbrk");
    if let Some(old_brk) = change_program_brk(size) {
        old_brk as isize
    } else {
        -1
    }
}
