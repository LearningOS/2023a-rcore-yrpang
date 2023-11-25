//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM,
    task::{
        change_program_brk, exit_current_and_run_next, suspend_current_and_run_next, TaskStatus, get_counter, get_first_start_time, 
        current_user_token, map_new_page, umap_page
    },
    mm::{
        translated_byte_buffer
    },
    timer::get_time_us,
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
pub struct TaskInfo {
    /// Task status in it's life cycle
    status: TaskStatus,
    /// The numbers of syscall called by task
    syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Total running time of task
    time: usize,
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
pub fn sys_get_time(_ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let us = get_time_us();
    let ts_tmp = TimeVal {
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };

    let len = core::mem::size_of::<TimeVal>();
    let buffers = translated_byte_buffer(current_user_token(), _ts as usize as *const u8, len);
    let ts_tmp_bytes = unsafe {
        core::slice::from_raw_parts(&ts_tmp as *const _ as *const u8, len)
    };
    let mut i = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = ts_tmp_bytes[i];
            i = i + 1;
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(_ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let time_now = get_time_us();
    let task_tmp = TaskInfo {
        status: TaskStatus::Running,
        syscall_times: get_counter(),
        time: (time_now - get_first_start_time())/1000,
    };
    let len = core::mem::size_of::<TaskInfo>();
    let buffers = translated_byte_buffer(current_user_token(), _ti as usize as *const u8, len);
    let task_tmp_bytes = unsafe {
        core::slice::from_raw_parts(&task_tmp as *const _ as *const u8, len)
    };

    let mut i = 0;
    for buffer in buffers {
        for byte in buffer {
            *byte = task_tmp_bytes[i];
            i = i + 1;
        }
    }
    
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(_start: usize, _len: usize, _port: usize) -> isize {
    trace!("kernel: sys_mmap");

    if _start & 0xfff != 0 {
        return -1;
    }  else if _port & !0x7 != 0 || _port & 0x7 == 0 {
        return -1;
    } else if _len == 0 {
        return 0;
    }

    map_new_page(_start, _len, _port)
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(_start: usize, _len: usize) -> isize {
    trace!("kernel: sys_munmap");
    if _start & 0xfff != 0 {
        return -1;
    }  else if _len == 0 {
        return 0;
    }

    umap_page(_start, _len)
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
