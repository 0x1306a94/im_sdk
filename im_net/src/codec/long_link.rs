use im_util::buffer::AutoBuffer;

pub enum DecodeStatus {
    /// 解码成功
    Ok,
    /// 解码失败
    Fail,
    /// 数据包不满足解码数据要求,等待下一次解码
    Continue,
}

pub struct CmdId(u32);
pub struct TaskId(u32);

pub trait Codec {
    /// 设置版本号
    fn set_version(version: u32);

    /// 编码
    /// cmd_id: 命令id
    /// task_id: 任务id
    /// out_buffer: 输出buffer
    /// extend_buffer: 输出扩展buffer
    fn encode(
        cmd_id: CmdId,
        task_id: TaskId,
        out_buffer: &mut AutoBuffer,
        extend_buffer: &mut AutoBuffer,
    );

    /// 解码
    /// in_buffer: 待解码内容
    /// body_buffer: 解码后内容
    /// extend_buffer: 解码后扩展内容
    fn decode(
        in_buffer: &AutoBuffer,
        body_buffer: &mut AutoBuffer,
        extend_buffer: &AutoBuffer,
    ) -> (DecodeStatus, CmdId, TaskId, usize);
}
