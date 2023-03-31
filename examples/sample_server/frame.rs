use bytes::{Buf, Bytes};
use im_codec::long_link::{CmdId, TaskId};

#[derive(Debug)]
pub struct Frame {
    cmd_id: CmdId,
    task_id: TaskId,
    bytes: Bytes,
}

impl Frame {
    pub fn new(cmd_id: CmdId, task_id: TaskId, bytes: Bytes) -> Self {
        Frame {
            cmd_id,
            task_id,
            bytes,
        }
    }
}
