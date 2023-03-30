use std::alloc::Layout;

use im_util::buffer::AutoBuffer;

pub enum DecodeStatus {
    /// 解码成功
    Ok,
    /// 解码失败
    Fail,
    /// 数据包不满足解码数据要求,等待下一次解码
    Continue,
}

#[derive(Debug, Clone)]
pub struct CmdId(pub u32);
#[derive(Debug, Clone)]
pub struct TaskId(pub u32);

pub const IdentifyCheckerTaskId: TaskId = TaskId(0xFFFFFFFE);

pub trait Codec: Send {
    /// 编码
    /// cmd_id: 命令id
    /// task_id: 任务id
    /// in_buffer: 输入数据
    /// extend_buffer: 额外的输入数据
    /// out_buffer: 输出buffer
    fn encode(
        &self,
        cmd_id: &CmdId,
        task_id: &TaskId,
        in_buffer: &AutoBuffer,
        extend_buffer: &AutoBuffer,
        out_buffer: &mut AutoBuffer,
    );

    /// 解码
    /// in_buffer: 待解码内容
    /// body_buffer: 解码后内容
    /// extend_buffer: 解码后扩展内容
    fn decode(
        &self,
        in_buffer: &AutoBuffer,
        body_buffer: &mut AutoBuffer,
        extend_buffer: &mut AutoBuffer,
    ) -> (DecodeStatus, CmdId, TaskId, usize);
}

#[repr(C, align(1))]
pub struct MsgHeader {
    pub version: u32,
    pub cmd_id: u32,
    pub task_id: u32,
    pub body_len: u16,
}

impl Default for MsgHeader {
    fn default() -> Self {
        MsgHeader {
            version: DEFAULT_VERSION,
            cmd_id: 0,
            task_id: 0,
            body_len: 0,
        }
    }
}

pub const DEFAULT_VERSION: u32 = 100;
pub const DEFAULT_HEADER_LEN: usize = 4 + 4 + 4 + 2;

pub struct DefaultLongLinkCodec {
    header_len: usize,
    version: u32,
}

impl DefaultLongLinkCodec {
    pub fn new(version: u32) -> Self {
        DefaultLongLinkCodec {
            header_len: DEFAULT_HEADER_LEN,
            version: version,
        }
    }
}

impl Codec for DefaultLongLinkCodec {
    fn encode(
        &self,
        cmd_id: &CmdId,
        task_id: &TaskId,
        in_buffer: &AutoBuffer,
        extend_buffer: &AutoBuffer,
        out_buffer: &mut AutoBuffer,
    ) {
        let mut header = MsgHeader::default();
        header.version = self.version;
        header.cmd_id = cmd_id.0;
        header.task_id = task_id.0;
        header.body_len = (in_buffer.len()) as u16;

        out_buffer.write(&header.version.to_be_bytes()[..]);
        out_buffer.write(&header.cmd_id.to_be_bytes()[..]);
        out_buffer.write(&header.task_id.to_be_bytes()[..]);
        out_buffer.write(&header.body_len.to_be_bytes()[..]);
        out_buffer.write_from(in_buffer);
    }

    fn decode(
        &self,
        in_buffer: &AutoBuffer,
        body_buffer: &mut AutoBuffer,
        extend_buffer: &mut AutoBuffer,
    ) -> (DecodeStatus, CmdId, TaskId, usize) {
        if in_buffer.len() < self.header_len {
            return (DecodeStatus::Continue, CmdId(0), TaskId(0), 0);
        }
        let bytes = in_buffer.as_slice(0);
        let header: MsgHeader = unsafe {
            let src = &bytes[..DEFAULT_HEADER_LEN];
            std::mem::transmute(src)
        };

        if header.version != self.version {
            return (DecodeStatus::Fail, CmdId(0), TaskId(0), 0);
        }

        let pack_len = DEFAULT_HEADER_LEN + (header.body_len as usize);
        if in_buffer.len() < pack_len {
            return (DecodeStatus::Continue, CmdId(0), TaskId(0), 0);
        }

        (
            DecodeStatus::Ok,
            CmdId(header.cmd_id),
            TaskId(header.task_id),
            header.body_len as usize,
        )
    }
}
