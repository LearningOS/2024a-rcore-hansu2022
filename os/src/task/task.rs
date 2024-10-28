//! Types related to task management

use crate::config::MAX_SYSCALL_NUM;

use super::TaskContext;

/// The task control block (TCB) of a task.
#[derive(Copy, Clone)]
pub struct TaskControlBlock {
    /// The task status in it's lifecycle
    pub task_status: TaskStatus,
    /// The task context
    pub task_cx: TaskContext,
    
    /// Statistics about syscalls that have been called
    pub syscall_times: [u32; MAX_SYSCALL_NUM],
    /// Time when the task was first scheduled
    pub first_schedule_time: Option<usize>,
}

/// The status of a task
#[derive(Copy, Clone, PartialEq)]
pub enum TaskStatus {
    /// uninitialized
    UnInit,
    /// ready to run
    Ready,
    /// running
    Running,
    /// exited
    Exited,
}

impl TaskControlBlock {
     /// Create a new TaskControlBlock with default values
    pub fn new() -> Self{
        Self{
            task_cx : TaskContext::zero_init(),
            task_status : TaskStatus::UnInit,
            syscall_times : [0; MAX_SYSCALL_NUM],
            first_schedule_time : None,
        }
    }
}