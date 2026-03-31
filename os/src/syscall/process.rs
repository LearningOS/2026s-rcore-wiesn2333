//! Process management syscalls
use crate::{
    config::PAGE_SIZE,
    mm::{PageTable, VirtAddr, VirtPageNum},
    task::{
        change_program_brk, current_user_token, exit_current_and_run_next,
        suspend_current_and_run_next,
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
    let page_table = PageTable::from_token(current_user_token());

    let first_ppn = {
        if let Some(pte) = page_table.translate(va.floor()) {
            if pte.is_valid() && pte.writable() {
                pte.ppn()
            } else {
                return -1;
            }
        } else {
            return -1;
        }
    };

    let second_ppn = {
        if va.page_offset() + core::mem::size_of::<TimeVal>() > PAGE_SIZE {
            let next_vpn = VirtPageNum(va.floor().0 + 1);
            if let Some(pte) = page_table.translate(next_vpn) {
                if pte.is_valid() && pte.writable() {
                    Some(pte.ppn())
                } else {
                    return -1;
                }
            } else {
                return -1;
            }
        } else {
            None
        }
    };

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

    let first_len = time_val_bytes.len().min(PAGE_SIZE - va.page_offset());

    let first_offset = va.page_offset();
    let first_page_bytes = first_ppn.get_bytes_array();
    first_page_bytes[first_offset..first_offset + first_len]
        .copy_from_slice(&time_val_bytes[..first_len]);

    if let Some(second_ppn) = second_ppn {
        let rest_len = core::mem::size_of::<TimeVal>() - first_len;
        let second_page_bytes = second_ppn.get_bytes_array();
        second_page_bytes[0..rest_len].copy_from_slice(&time_val_bytes[first_len..]);
    }

    0
}

/// TODO: Finish sys_trace to pass testcases
/// HINT: You might reimplement it with virtual memory management.
pub fn sys_trace(_trace_request: usize, _id: usize, _data: usize) -> isize {
    trace!("kernel: sys_trace");
    -1
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap NOT IMPLEMENTED YET!");
    -1
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap NOT IMPLEMENTED YET!");
    -1
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
