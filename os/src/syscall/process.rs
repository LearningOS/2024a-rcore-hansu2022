//! Process management syscalls
use crate::{
    config::MAX_SYSCALL_NUM, mm::{translated_byte_buffer,MapPermission, VirtAddr}, task::{
        change_program_brk, current_user_token, exit_current_and_run_next, suspend_current_and_run_next,get_current_task_info,mmap_handler,munmap_handler, TaskStatus
    }, timer::get_time_us
};

#[repr(C)]
#[derive(Debug)]
pub struct TimeVal {
    pub sec: usize,
    pub usec: usize,
}

/// Task information
#[allow(dead_code)]
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
pub fn sys_get_time(ts: *mut TimeVal, _tz: usize) -> isize {
    trace!("kernel: sys_get_time");
    let token = current_user_token();
    

    let us = get_time_us();
    let ts_val = TimeVal{
        sec: us / 1_000_000,
        usec: us % 1_000_000,
    };
    let byte_buffers = translated_byte_buffer(token, ts as *const u8, core::mem::size_of::<TimeVal>());
    if byte_buffers.is_empty(){
        return -1;
    }
    let mut written_btyes = 0;

    let ts_bytes =unsafe{
        core::slice::from_raw_parts(
            &ts_val as *const TimeVal as *const u8,
            core::mem::size_of::<TimeVal>()
        )
    };

    for buffer in byte_buffers{
        let len = buffer.len().min(core::mem::size_of::<TimeVal>() - written_btyes);
        buffer[..len].copy_from_slice(&ts_bytes[written_btyes..written_btyes + len]);
        written_btyes += len;
        if written_btyes >= core::mem::size_of::<TimeVal>(){
            break;
        }
    }
    0
}

/// YOUR JOB: Finish sys_task_info to pass testcases
/// HINT: You might reimplement it with virtual memory management.
/// HINT: What if [`TaskInfo`] is splitted by two pages ?
pub fn sys_task_info(ti: *mut TaskInfo) -> isize {
    trace!("kernel: sys_task_info");
    let token = current_user_token();



    let (status,syscall_times,start_time) = get_current_task_info();

    let task_info = TaskInfo{
        status,
        syscall_times,
        time:(get_time_us() - start_time) / 1_000,
    };

    let byte_buffers = translated_byte_buffer(
        token,
        ti as *const u8,
        core::mem::size_of::<TaskInfo>()
    );

    if byte_buffers.is_empty(){
        return -1;
    }

    let mut written_bytes = 0;

    let ti_bytes = unsafe{
        core::slice::from_raw_parts(
            &task_info as *const TaskInfo as *const u8,
            core::mem::size_of::<TaskInfo>()
        )
    };

    for buffer in byte_buffers{
        let len = buffer.len().min(core::mem::size_of::<TaskInfo>() - written_bytes);
        buffer[..len].copy_from_slice(&ti_bytes[written_bytes..written_bytes + len]);
        written_bytes += len;
        if written_bytes >= core::mem::size_of::<TaskInfo>(){
            break;
        }
    }
    0
}

// YOUR JOB: Implement mmap.
pub fn sys_mmap(start: usize, len: usize, port: usize) -> isize {
    trace!("kernel: sys_mmap");
    let start_va = VirtAddr::from(start);
    if !start_va.aligned(){
        return -1;
    }

    if len == 0{
        return -1;
    }

    if port & !0x7 != 0 || port & 0x7 == 0{
        return -1;
    }

    if start.checked_add(len).is_none(){
        return -1;
    }

    let mut permission = MapPermission::U;
    if port & 0x1 != 0{
        permission |= MapPermission::R;
    }
    if port & 0x2 != 0{
        permission |= MapPermission::W;
    }
    if port & 0x4 != 0{
        permission |= MapPermission::X;
    }

    match mmap_handler(start_va, len, permission){
        true => 0,
        false => -1,
    }
}

// YOUR JOB: Implement munmap.
pub fn sys_munmap(start: usize, len: usize) -> isize {
    trace!("kernel: sys_munmap");
    // 地址必须按页对齐
    let start_va = VirtAddr::from(start);
    if !start_va.aligned() {
        return -1;
    }
    
    // 检查地址范围溢出
    if start.checked_add(len).is_none() {
        return -1;
    }
    
    // 尝试解除映射
    match munmap_handler(start_va, len) {
        true => 0,
        false => -1,
    }

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
