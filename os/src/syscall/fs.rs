//! File and filesystem-related syscalls
use crate::fs::{open_file, OpenFlags, Stat,ROOT_INODE };
use crate::mm::{translated_byte_buffer, translated_refmut, translated_str, UserBuffer};
use crate::task::{current_task, current_user_token};

pub fn sys_write(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_write", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        if !file.writable() {
            return -1;
        }
        let file = file.clone();
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        file.write(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_read(fd: usize, buf: *const u8, len: usize) -> isize {
    trace!("kernel:pid[{}] sys_read", current_task().unwrap().pid.0);
    let token = current_user_token();
    let task = current_task().unwrap();
    let inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if let Some(file) = &inner.fd_table[fd] {
        let file = file.clone();
        if !file.readable() {
            return -1;
        }
        // release current task TCB manually to avoid multi-borrow
        drop(inner);
        trace!("kernel: sys_read .. file.read");
        file.read(UserBuffer::new(translated_byte_buffer(token, buf, len))) as isize
    } else {
        -1
    }
}

pub fn sys_open(path: *const u8, flags: u32) -> isize {
    trace!("kernel:pid[{}] sys_open", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let token = current_user_token();
    let path = translated_str(token, path);
    if let Some(inode) = open_file(path.as_str(), OpenFlags::from_bits(flags).unwrap()) {
        let mut inner = task.inner_exclusive_access();
        let fd = inner.alloc_fd();
        inner.fd_table[fd] = Some(inode);
        fd as isize
    } else {
        -1
    }
}

pub fn sys_close(fd: usize) -> isize {
    trace!("kernel:pid[{}] sys_close", current_task().unwrap().pid.0);
    let task = current_task().unwrap();
    let mut inner = task.inner_exclusive_access();
    if fd >= inner.fd_table.len() {
        return -1;
    }
    if inner.fd_table[fd].is_none() {
        return -1;
    }
    inner.fd_table[fd].take();
    0
}

/// YOUR JOB: Implement fstat.
pub fn sys_fstat(fd: usize, st: *mut Stat) -> isize {
    trace!(
        "kernel:pid[{}] sys_fstat NOT IMPLEMENTED",
        current_task().unwrap().pid.0
    );
    let task = current_task().unwrap();
    // 获取内部可变引用
    let inner = task.inner_exclusive_access();
    
    if fd >= inner.fd_table.len() {
        return -1;
    }
    
    let ret = if let Some(file) = &inner.fd_table[fd] {
        // 克隆file的引用计数
        let file = file.clone();
        // 先释放inner的锁
        drop(inner);
        
        // 获取文件状态
        if let Some(stat) = file.get_stat() {
            let token = current_user_token();
            *translated_refmut(token, st) = stat;
            0
        } else {
            -1
        }
    } else {
        -1
    };
    
    ret
}

/// YOUR JOB: Implement linkat.
pub fn sys_linkat(old_name: *const u8, new_name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_linkat",
        current_task().unwrap().pid.0
    );
    
    let token = current_user_token();
    
    // 转换路径名
    let old_path = translated_str(token, old_name);
    let new_path = translated_str(token, new_name);
    
    // 检查源文件是否存在
    if let Some(old_inode) = ROOT_INODE.find(&old_path) {
        // 检查是否是同名文件
        if old_path == new_path {
            return -1;
        }
        
        // 检查新路径是否已存在
        if ROOT_INODE.find(&new_path).is_some() {
            return -1;
        }
        
        // 创建新的目录项
        let fs = ROOT_INODE.create_link(&new_path, &old_inode);
        if fs.is_some() {
            // 增加硬链接计数
            0
        } else {
            -1
        }
    } else {
        -1
    }
}

/// YOUR JOB: Implement unlinkat.
pub fn sys_unlinkat(name: *const u8) -> isize {
    trace!(
        "kernel:pid[{}] sys_unlinkat ",
        current_task().unwrap().pid.0
    );
    let token = current_user_token();
    let path = translated_str(token, name);
    
    let path = path.trim_matches('\0');
    // 获取文件的inode
    if let Some(inode) = ROOT_INODE.find(&path) {
        // 从目录中移除该项
        if ROOT_INODE.remove_dirent(&path) {
            // 减少硬链接计数
            if inode.decrease_nlink() {
                // 如果是最后一个链接，可以选择清除文件内容ch5
                inode.clear();
            }
            return 0;
        }
    }
    -1 
}