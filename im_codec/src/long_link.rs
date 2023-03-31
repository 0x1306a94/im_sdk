use std::alloc::Layout;

use im_util::buffer::{auto_buffer::Seek, AutoBuffer};

pub enum DecodeStatus {
    /// 解码成功
    Ok,
    /// 解码失败
    Fail,
    /// 数据包不满足解码数据要求,等待下一次解码
    Continue,
}

pub type CmdId = u32;
pub type TaskId = u32;

pub const IDENTIFY_CHECKER_TASK_ID: TaskId = 0xFFFFFFFE;
pub const DEFAULT_VERSION: u32 = 100;
pub const DEFAULT_HEADER_LEN: usize = 4 + 4 + 4 + 2;

pub trait Codec: Send {
    /// 编码
    /// cmd_id: 命令id
    /// task_id: 任务id
    /// in_buffer: 输入数据
    /// extend_buffer: 额外的输入数据
    /// out_buffer: 输出buffer
    fn encode(
        &self,
        cmd_id: CmdId,
        task_id: TaskId,
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
    pub cmd_id: CmdId,
    pub task_id: TaskId,
    pub body_len: u16,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ParseHeaderError {
    Underlength,
    VersionMismatch,
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

impl TryFrom<&[u8]> for MsgHeader {
    type Error = ParseHeaderError;
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < DEFAULT_HEADER_LEN {
            return Err(ParseHeaderError::Underlength);
        }

        let header: MsgHeader = unsafe {
            let src = &bytes[..DEFAULT_HEADER_LEN];
            std::mem::transmute(src)
        };

        if header.version != DEFAULT_VERSION {
            return Err(ParseHeaderError::VersionMismatch);
        }

        Ok(header)
    }
}

pub struct DefaultCodec {
    header_len: usize,
    version: u32,
}

impl DefaultCodec {
    pub fn new(version: u32) -> Self {
        DefaultCodec {
            header_len: DEFAULT_HEADER_LEN,
            version: version,
        }
    }
}

impl Codec for DefaultCodec {
    fn encode(
        &self,
        cmd_id: CmdId,
        task_id: TaskId,
        in_buffer: &AutoBuffer,
        extend_buffer: &AutoBuffer,
        out_buffer: &mut AutoBuffer,
    ) {
        let mut header = MsgHeader::default();
        header.version = self.version;
        header.cmd_id = cmd_id;
        header.task_id = task_id;
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
        let bytes = in_buffer.as_slice(0);

        let header;
        match MsgHeader::try_from(bytes) {
            Ok(v) => header = v,
            Err(e) => match e {
                ParseHeaderError::Underlength => {
                    return (DecodeStatus::Continue, 0, 0, 0);
                }
                ParseHeaderError::VersionMismatch => {
                    return (DecodeStatus::Fail, 0, 0, 0);
                }
            },
        }

        let pack_len = DEFAULT_HEADER_LEN + (header.body_len as usize);
        if in_buffer.len() < pack_len {
            return (DecodeStatus::Continue, 0, 0, 0);
        }

        let satrt = pack_len - (header.body_len as usize);
        let end = satrt + (header.body_len as usize);

        body_buffer.write_at_seek(Seek::Cur, &bytes[satrt..end]);

        (
            DecodeStatus::Ok,
            header.cmd_id,
            header.task_id,
            header.body_len as usize,
        )
    }
}
